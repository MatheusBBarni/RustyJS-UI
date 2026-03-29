use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, TextNode, UiNode};
use serde_json::Value;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const STORAGE_APP: &str = r#"
let status = 'Idle';

async function writeTheme() {
    status = 'Writing';
    App.requestRender();
    const value = await Storage.set('theme', 'dark');
    status = `Stored: ${value ?? 'null'}`;
    App.requestRender();
}

async function readTheme() {
    status = 'Reading';
    App.requestRender();
    const value = await Storage.get('theme');
    status = `Read: ${value ?? 'null'}`;
    App.requestRender();
}

async function removeTheme() {
    status = 'Removing';
    App.requestRender();
    const value = await Storage.remove('theme');
    status = `Removed: ${value ?? 'null'}`;
    App.requestRender();
}

async function clearStorage() {
    status = 'Clearing';
    App.requestRender();
    await Storage.clear();
    status = 'Cleared';
    App.requestRender();
}

function AppLayout() {
    return View({
        children: [
            Text({ text: status }),
            Button({ text: 'Write', onClick: writeTheme }),
            Button({ text: 'Read', onClick: readTheme }),
            Button({ text: 'Remove', onClick: removeTheme }),
            Button({ text: 'Clear', onClick: clearStorage })
        ]
    });
}

App.run({
    title: 'Storage Bridge Test',
    render: AppLayout
});
"#;

#[test]
fn storage_bridge_round_trips_values_and_persists_across_runtime_instances() {
    let storage_path = unique_storage_path();
    let mut runtime = boot_runtime(&storage_path);
    let payloads = runtime.eval_script(STORAGE_APP).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();

    let write_callback = find_button(&initial_tree, "Write")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Write callback")
        .id
        .clone();
    let read_callback = find_button(&initial_tree, "Read")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Read callback")
        .id
        .clone();
    let remove_callback = find_button(&initial_tree, "Remove")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Remove callback")
        .id
        .clone();
    let clear_callback = find_button(&initial_tree, "Clear")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Clear callback")
        .id
        .clone();

    let writing_payloads = runtime.trigger_callback(&write_callback, Value::Null).unwrap();
    assert!(collect_texts(&writing_payloads[0].typed_tree().unwrap().unwrap())
        .iter()
        .any(|text| *text == "Writing"));
    let stored_payloads = wait_for_async_payloads(&mut runtime);
    assert!(collect_texts(&stored_payloads[0].typed_tree().unwrap().unwrap())
        .iter()
        .any(|text| *text == "Stored: dark"));

    let mut second_runtime = boot_runtime(&storage_path);
    let second_payloads = second_runtime.eval_script(STORAGE_APP).unwrap();
    let second_tree = second_payloads[1].typed_tree().unwrap().unwrap();
    let second_read_callback = find_button(&second_tree, "Read")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Read callback")
        .id
        .clone();

    second_runtime
        .trigger_callback(&second_read_callback, Value::Null)
        .unwrap();
    let persisted_payloads = wait_for_async_payloads(&mut second_runtime);
    assert!(collect_texts(&persisted_payloads[0].typed_tree().unwrap().unwrap())
        .iter()
        .any(|text| *text == "Read: dark"));

    runtime.trigger_callback(&read_callback, Value::Null).unwrap();
    let read_payloads = wait_for_async_payloads(&mut runtime);
    assert!(collect_texts(&read_payloads[0].typed_tree().unwrap().unwrap())
        .iter()
        .any(|text| *text == "Read: dark"));

    runtime
        .trigger_callback(&remove_callback, Value::Null)
        .unwrap();
    let remove_payloads = wait_for_async_payloads(&mut runtime);
    assert!(collect_texts(&remove_payloads[0].typed_tree().unwrap().unwrap())
        .iter()
        .any(|text| *text == "Removed: dark"));

    runtime.trigger_callback(&clear_callback, Value::Null).unwrap();
    let clear_payloads = wait_for_async_payloads(&mut runtime);
    assert!(collect_texts(&clear_payloads[0].typed_tree().unwrap().unwrap())
        .iter()
        .any(|text| *text == "Cleared"));

    let _ = std::fs::remove_file(storage_path);
}

fn boot_runtime(storage_path: &PathBuf) -> JsRuntime {
    let mut runtime = JsRuntime::new_with_storage_path(storage_path.clone()).unwrap();
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

fn unique_storage_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("rustyjs-ui-storage-bridge-{nanos}.json"))
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

