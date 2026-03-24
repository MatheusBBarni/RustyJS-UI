use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, TextInputNode, TextNode, UiNode};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const POKEMON_FETCH_APP: &str = include_str!("../examples/pokemon_fetch.js");

#[test]
fn pokemon_fetch_example_renders_initial_state() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(POKEMON_FETCH_APP).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 760,
            height: 620
        } if title == "Pokemon Fetch Example"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);
    let input = find_text_input(&tree, "Pokemon name").expect("expected pokemon name input");
    let button = find_button(&tree, "Search").expect("expected search button");

    assert_eq!(input.value, "");
    assert!(button.on_click.is_some());
    assert!(texts.iter().any(|text| *text == "Pokemon Fetch Example"));
    assert!(texts.iter().any(|text| *text == "Pokédex card"));
}

#[test]
fn pokemon_fetch_example_loads_pokemon_details() {
    let server = spawn_server(
        "HTTP/1.1 200 OK",
        r#"{
  "id": 132,
  "name": "ditto",
  "base_experience": 101,
  "abilities": [
    { "ability": { "name": "limber" } },
    { "ability": { "name": "imposter" } }
  ],
  "types": [
    { "slot": 1, "type": { "name": "normal" } }
  ]
}"#,
    );
    let mut runtime = boot_runtime();
    let app_source = format!(
        "globalThis.__POKEAPI_BASE_URL__ = '{}';\n{}",
        server.base_url, POKEMON_FETCH_APP
    );
    let payloads = runtime.eval_script(&app_source).unwrap();
    let mut tree = payloads[1].typed_tree().unwrap().unwrap();

    let input_callback = find_text_input(&tree, "Pokemon name")
        .and_then(|input| input.on_change.as_ref())
        .expect("expected pokemon input callback")
        .id
        .clone();

    let payloads = runtime
        .trigger_callback(&input_callback, Value::String("ditto".to_string()))
        .unwrap();
    tree = payloads[0].typed_tree().unwrap().unwrap();

    let search_callback = find_button(&tree, "Search")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected search button callback")
        .id
        .clone();

    let payloads = runtime
        .trigger_callback(&search_callback, Value::Null)
        .unwrap();
    let loading_tree = payloads[0].typed_tree().unwrap().unwrap();
    let loading_texts = collect_texts(&loading_tree);

    assert!(loading_texts
        .iter()
        .any(|text| *text == "Searching PokeAPI..."));

    let request = server
        .request_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    let request_lower = request.to_ascii_lowercase();
    assert!(request_lower.starts_with("get /ditto http/1.1"));

    let payloads = wait_for_async_payloads(&mut runtime);
    let loaded_tree = payloads[0].typed_tree().unwrap().unwrap();
    let loaded_texts = collect_texts(&loaded_tree);

    assert!(loaded_texts.iter().any(|text| *text == "Ditto"));
    assert!(loaded_texts.iter().any(|text| *text == "Dex ID"));
    assert!(loaded_texts.iter().any(|text| *text == "#132"));
    assert!(loaded_texts.iter().any(|text| *text == "Types"));
    assert!(loaded_texts.iter().any(|text| *text == "Normal"));
    assert!(loaded_texts.iter().any(|text| *text == "Base Experience"));
    assert!(loaded_texts.iter().any(|text| *text == "101"));
    assert!(loaded_texts.iter().any(|text| *text == "Abilities"));
    assert!(loaded_texts.iter().any(|text| *text == "Limber, Imposter"));
}

fn boot_runtime() -> JsRuntime {
    let mut runtime = JsRuntime::new().unwrap();
    assert!(runtime
        .eval_script(JsRuntime::bootstrap_source())
        .unwrap()
        .is_empty());
    runtime
}

fn wait_for_async_payloads(runtime: &mut JsRuntime) -> Vec<BridgePayload> {
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

fn find_text_input<'a>(node: &'a UiNode, placeholder: &str) -> Option<&'a TextInputNode> {
    match node {
        UiNode::TextInput(input) if input.placeholder.as_deref() == Some(placeholder) => {
            Some(input)
        }
        UiNode::TextInput(_) => None,
        UiNode::Text(_) | UiNode::Button(_) | UiNode::SelectInput(_) => None,
        _ => node
            .children()
            .iter()
            .find_map(|child| find_text_input(child, placeholder)),
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
            "{status_line}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{body}",
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

        if find_header_end(&buffer).is_some() {
            break;
        }
    }

    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}
