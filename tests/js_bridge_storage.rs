use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, TextNode, UiNode};
use serde_json::Value;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn storage_round_trip_updates_app_state() {
    let storage_file = unique_storage_path("storage-bridge");
    let app_source = format!(
        r#"
let state = {{
    value: 'unset'
}};

async function saveTheme() {{
    await Storage.set('theme', 'dark');
    state.value = (await Storage.get('theme')) ?? 'missing';
    App.requestRender();
}}

function AppLayout() {{
    return View({{
        children: [
            Text({{ text: `Stored: ${{state.value}}` }}),
            Button({{ text: 'Save', onClick: saveTheme }})
        ]
    }});
}}

App.run({{
    title: 'Storage Test',
    render: AppLayout
}});
"#
    );

    let mut runtime = boot_runtime_with_storage(&storage_file);
    let payloads = runtime.eval_script(&app_source).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let callback_id = find_button(&initial_tree, "Save")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Save callback")
        .id
        .clone();

    assert!(runtime.trigger_callback(&callback_id, Value::Null).unwrap().is_empty());

    let completed_payloads = wait_for_async_payloads(&mut runtime);
    let completed_tree = completed_payloads[0].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&completed_tree);

    assert!(texts.iter().any(|text| *text == "Stored: dark"));

    let _ = std::fs::remove_file(storage_file);
}

fn boot_runtime_with_storage(storage_file: &std::path::Path) -> JsRuntime {
    let mut runtime = JsRuntime::new_with_storage_path(storage_file.to_path_buf()).unwrap();
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

fn unique_storage_path(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("rustyjs-ui-{label}-{suffix}.json"))
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
