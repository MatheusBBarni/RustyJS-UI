use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, TextNode, UiNode};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn fetch_resolves_and_re_renders_app_state() {
    let server = spawn_server("HTTP/1.1 200 OK", "fetched from rust");
    let app_source = format!(
        r#"
let state = {{
    loading: false,
    data: 'waiting',
    error: null
}};

async function loadData() {{
    state.loading = true;
    state.error = null;
    App.requestRender();

    try {{
        state.data = await fetch('{url}/items', {{
            method: 'GET',
            headers: {{ 'X-Test-Token': 'bridge' }}
        }});
    }} catch (error) {{
        state.error = error.message;
    }} finally {{
        state.loading = false;
        App.requestRender();
    }}
}}

function AppLayout() {{
    return View({{
        children: [
            Text({{ text: state.loading ? 'Loading...' : 'Idle' }}),
            Text({{ text: `Data: ${{state.data}}` }}),
            Text({{ text: state.error ? `Error: ${{state.error}}` : 'Error: none' }}),
            Button({{ text: 'Load', onClick: loadData }})
        ]
    }});
}}

App.run({{
    title: 'Fetch Bridge Test',
    render: AppLayout
}});
"#,
        url = server.base_url
    );

    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(&app_source).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let callback_id = find_button(&initial_tree, "Load")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Load callback")
        .id
        .clone();

    let loading_payloads = runtime.trigger_callback(&callback_id, Value::Null).unwrap();
    let loading_tree = loading_payloads[0].typed_tree().unwrap().unwrap();
    let loading_texts = collect_texts(&loading_tree);

    assert!(loading_texts.iter().any(|text| *text == "Loading..."));

    let request = server
        .request_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    let request_lower = request.to_ascii_lowercase();
    assert!(request_lower.starts_with("get /items http/1.1"));
    assert!(request_lower.contains("x-test-token: bridge"));

    let completed_payloads = wait_for_async_payloads(&mut runtime);
    let completed_tree = completed_payloads[0].typed_tree().unwrap().unwrap();
    let completed_texts = collect_texts(&completed_tree);

    assert!(completed_texts.iter().any(|text| *text == "Idle"));
    assert!(completed_texts
        .iter()
        .any(|text| *text == "Data: fetched from rust"));
    assert!(completed_texts.iter().any(|text| *text == "Error: none"));
}

#[test]
fn fetch_rejects_http_errors_and_re_renders_error_state() {
    let server = spawn_server("HTTP/1.1 500 Internal Server Error", "boom");
    let app_source = format!(
        r#"
let state = {{
    loading: false,
    error: null
}};

async function loadData() {{
    state.loading = true;
    App.requestRender();

    try {{
        await fetch('{url}/fail');
    }} catch (error) {{
        state.error = error.message;
    }} finally {{
        state.loading = false;
        App.requestRender();
    }}
}}

function AppLayout() {{
    return View({{
        children: [
            Text({{ text: state.loading ? 'Loading...' : 'Idle' }}),
            Text({{ text: state.error ? `Error: ${{state.error}}` : 'Error: none' }}),
            Button({{ text: 'Load', onClick: loadData }})
        ]
    }});
}}

App.run({{
    title: 'Fetch Error Test',
    render: AppLayout
}});
"#,
        url = server.base_url
    );

    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(&app_source).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let callback_id = find_button(&initial_tree, "Load")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Load callback")
        .id
        .clone();

    runtime.trigger_callback(&callback_id, Value::Null).unwrap();
    let _ = server
        .request_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap();

    let completed_payloads = wait_for_async_payloads(&mut runtime);
    let completed_tree = completed_payloads[0].typed_tree().unwrap().unwrap();
    let completed_texts = collect_texts(&completed_tree);

    assert!(completed_texts.iter().any(|text| *text == "Idle"));
    assert!(completed_texts
        .iter()
        .any(|text| *text == "Error: HTTP 500 Internal Server Error"));
}

fn boot_runtime() -> JsRuntime {
    let mut runtime = JsRuntime::new().unwrap();
    assert!(runtime
        .eval_script(JsRuntime::bootstrap_source())
        .unwrap()
        .is_empty());
    runtime
}

fn wait_for_async_payloads(runtime: &mut JsRuntime) -> Vec<rustyjs_ui::bridge::BridgePayload> {
    for _ in 0..200 {
        let payloads = runtime.poll_async().unwrap();
        if !payloads.is_empty() {
            return payloads;
        }

        thread::sleep(Duration::from_millis(10));
    }

    panic!("timed out waiting for async payloads");
}

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
