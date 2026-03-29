use anyhow::Result;
use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::perf::{reset_metrics, snapshot_metrics, PerfSnapshot};
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, TextNode, UiNode};
use serde::Serialize;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
struct PerfReport {
    generated_at_epoch_ms: u128,
    scenarios: Vec<ScenarioResult>,
}

#[derive(Debug, Serialize)]
struct ScenarioResult {
    name: String,
    elapsed_micros: u128,
    metrics: PerfSnapshot,
}

fn main() -> Result<()> {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run_harness)?
        .join()
        .map_err(|_| anyhow::anyhow!("perf harness thread panicked"))?
}

fn run_harness() -> Result<()> {
    let report = PerfReport {
        generated_at_epoch_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis(),
        scenarios: vec![
            scenario("startup_counter", || {
                let _ = JsRuntime::startup()?;
                Ok(())
            })?,
            scenario("route_transition", || {
                let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("examples/router_demo/main.js");
                let (mut runtime, payloads) = JsRuntime::startup_with_app_entry(&fixture)?;
                let tree = payloads[1].typed_tree()?.unwrap();
                let callback_id = find_button(&tree, "Open project alpha")
                    .and_then(|button| button.on_click.as_ref())
                    .expect("expected Open project alpha callback")
                    .id
                    .clone();
                let _ = runtime.trigger_callback(&callback_id, Value::Null)?;
                Ok(())
            })?,
            scenario("text_input_burst", || {
                let mut runtime = boot_runtime();
                let payloads = runtime.eval_script(
                    r#"
let value = '';

function handleChange(nextValue) {
    value = nextValue;
    App.requestRender();
}

function AppLayout() {
    return TextInput({
        value,
        onChange: handleChange
    });
}

App.run({
    title: 'Perf Text Input',
    render: AppLayout
});
"#,
                )?;
                let tree = payloads[1].typed_tree()?.unwrap();
                let callback_id = match tree {
                    UiNode::TextInput(input) => input.on_change.unwrap().id,
                    other => panic!("expected TextInput, got {other:?}"),
                };

                for index in 0..32 {
                    let _ = runtime.trigger_callback(
                        &callback_id,
                        Value::String("a".repeat(index + 1)),
                    )?;
                }

                Ok(())
            })?,
            scenario("modal_open_close", || {
                let mut runtime = boot_runtime();
                let payloads = runtime.eval_script(include_str!("../../examples/modal.js"))?;
                let tree = payloads[1].typed_tree()?.unwrap();
                let open_callback_id = find_button(&tree, "Open modal")
                    .and_then(|button| button.on_click.as_ref())
                    .expect("expected Open modal callback")
                    .id
                    .clone();
                let payloads = runtime.trigger_callback(&open_callback_id, Value::Null)?;
                let tree = payloads[0].typed_tree()?.unwrap();
                let close_callback_id = find_button(&tree, "Close")
                    .and_then(|button| button.on_click.as_ref())
                    .expect("expected Close callback")
                    .id
                    .clone();
                let _ = runtime.trigger_callback(&close_callback_id, Value::Null)?;
                Ok(())
            })?,
            scenario("large_list_render", || {
                let mut runtime = boot_runtime();
                let data = (0..1000)
                    .map(|index| format!("'Row {index}'"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let app_source = format!(
                    r#"
const rows = [{data}];

function AppLayout() {{
    return NativeList({{
        data: rows,
        renderItem: ({{ item }}) => Text({{ text: item }})
    }});
}}

App.run({{
    title: 'Large List',
    render: AppLayout
}});
"#
                );
                let _ = runtime.eval_script(&app_source)?;
                Ok(())
            })?,
            scenario("fetch_round_trip", || {
                let server = spawn_server("HTTP/1.1 200 OK", "perf");
                let mut runtime = boot_runtime();
                let app_source = format!(
                    r#"
async function load() {{
    await fetch('{url}/ping');
    App.requestRender();
}}

function AppLayout() {{
    return Button({{
        text: 'Load',
        onClick: load
    }});
}}

App.run({{
    title: 'Perf Fetch',
    render: AppLayout
}});
"#,
                    url = server.base_url
                );
                let payloads = runtime.eval_script(&app_source)?;
                let tree = payloads[1].typed_tree()?.unwrap();
                let callback_id = find_button(&tree, "Load")
                    .and_then(|button| button.on_click.as_ref())
                    .expect("expected Load callback")
                    .id
                    .clone();
                let _ = runtime.trigger_callback(&callback_id, Value::Null)?;
                let _ = server.request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                let _ = wait_for_async_payloads(&mut runtime)?;
                Ok(())
            })?,
            scenario("storage_round_trip", || {
                let storage_file = std::env::temp_dir().join("rustyjs-ui-perf-storage.json");
                let mut runtime = JsRuntime::new_with_storage_path(storage_file.clone())?;
                assert!(runtime.eval_script(JsRuntime::bootstrap_source())?.is_empty());
                let payloads = runtime.eval_script(
                    r#"
async function save() {
    await Storage.set('theme', 'dark');
    await Storage.get('theme');
    App.requestRender();
}

function AppLayout() {
    return Button({
        text: 'Save',
        onClick: save
    });
}

App.run({
    title: 'Perf Storage',
    render: AppLayout
});
"#,
                )?;
                let tree = payloads[1].typed_tree()?.unwrap();
                let callback_id = find_button(&tree, "Save")
                    .and_then(|button| button.on_click.as_ref())
                    .expect("expected Save callback")
                    .id
                    .clone();
                let _ = runtime.trigger_callback(&callback_id, Value::Null)?;
                let _ = wait_for_async_payloads(&mut runtime)?;
                let _ = std::fs::remove_file(storage_file);
                Ok(())
            })?,
        ],
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn scenario(name: &str, run: impl FnOnce() -> Result<()>) -> Result<ScenarioResult> {
    reset_metrics();
    let started_at = Instant::now();
    run()?;

    Ok(ScenarioResult {
        name: name.to_string(),
        elapsed_micros: started_at.elapsed().as_micros(),
        metrics: snapshot_metrics(),
    })
}

fn boot_runtime() -> JsRuntime {
    let mut runtime = JsRuntime::new().unwrap();
    assert!(runtime
        .eval_script(JsRuntime::bootstrap_source())
        .unwrap()
        .is_empty());
    runtime
}

fn wait_for_async_payloads(runtime: &mut JsRuntime) -> Result<Vec<BridgePayload>> {
    for _ in 0..200 {
        let payloads = runtime.poll_async()?;
        if !payloads.is_empty() {
            return Ok(payloads);
        }

        thread::sleep(Duration::from_millis(10));
    }

    panic!("timed out waiting for async payloads");
}

fn find_button<'a>(node: &'a UiNode, label: &str) -> Option<&'a ButtonNode> {
    match node {
        UiNode::Button(button) if button.text == label => Some(button),
        UiNode::Button(_) => None,
        UiNode::Text(_) | UiNode::TextInput(_) | UiNode::SelectInput(_) => None,
        _ => node
            .children()
            .iter()
            .find_map(|child| find_button(child, label)),
    }
}

#[allow(dead_code)]
fn collect_texts(node: &UiNode) -> Vec<&str> {
    let mut texts = Vec::new();
    collect_texts_into(node, &mut texts);
    texts
}

fn collect_texts_into<'a>(node: &'a UiNode, texts: &mut Vec<&'a str>) {
    match node {
        UiNode::Text(TextNode { text, .. }) => texts.push(text.as_str()),
        UiNode::Button(_) | UiNode::TextInput(_) | UiNode::SelectInput(_) => {}
        _ => {
            for child in node.children() {
                collect_texts_into(child, texts);
            }
        }
    }
}

struct ServerFixture {
    base_url: String,
    request_rx: mpsc::Receiver<String>,
}

fn spawn_server(status_line: &'static str, body: &'static str) -> ServerFixture {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (request_tx, request_rx) = mpsc::channel();
    let body = body.to_string();
    let status_line = status_line.to_string();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_http_request(&mut stream).unwrap();
        let _ = request_tx.send(request);

        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        let _ = stream.shutdown(Shutdown::Both);
    });

    ServerFixture {
        base_url: format!("http://{addr}"),
        request_rx,
    }
}

fn read_http_request(stream: &mut TcpStream) -> std::io::Result<String> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;

    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];

    loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);

        if let Some(header_end) = find_header_end(&buffer) {
            let content_length = parse_content_length(&buffer[..header_end])?;
            let total_len = header_end + 4 + content_length;

            while buffer.len() < total_len {
                let read = stream.read(&mut chunk)?;
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }

            break;
        }
    }

    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(headers: &[u8]) -> std::io::Result<usize> {
    let headers = String::from_utf8_lossy(headers);

    for line in headers.lines() {
        let mut parts = line.splitn(2, ':');
        let name = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();

        if name.eq_ignore_ascii_case("content-length") {
            return value.parse::<usize>().map_err(|error| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
            });
        }
    }

    Ok(0)
}
