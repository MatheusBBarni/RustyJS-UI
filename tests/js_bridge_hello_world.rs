use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, TextNode, UiNode};
use serde_json::Value;

const HELLO_WORLD_APP: &str = include_str!("hello_world_counter.js");

#[test]
fn hello_world_fixture_renders_expected_vdom() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(HELLO_WORLD_APP).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 480,
            height: 320
        } if title == "Hello World Test"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);
    let button = find_button(&tree, "Increment").expect("expected Increment button");

    assert!(texts.iter().any(|text| *text == "Hello world"));
    assert!(texts.iter().any(|text| *text == "Count is: 0"));
    assert_eq!(
        button.on_click.as_ref().map(|callback| callback.id.as_str()),
        Some("cb_1")
    );
}

#[test]
fn increment_button_callback_re_renders_fixture() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(HELLO_WORLD_APP).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let button = find_button(&initial_tree, "Increment").expect("expected Increment button");
    let callback_id = button.on_click.as_ref().unwrap().id.clone();

    let payloads = runtime.trigger_callback(&callback_id, Value::Null).unwrap();

    assert_eq!(payloads.len(), 1);

    let updated_tree = payloads[0].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&updated_tree);

    assert!(texts.iter().any(|text| *text == "Hello world"));
    assert!(texts.iter().any(|text| *text == "Count is: 1"));
}

fn boot_runtime() -> JsRuntime {
    let mut runtime = JsRuntime::new().unwrap();
    assert!(runtime.eval_script(JsRuntime::bootstrap_source()).unwrap().is_empty());
    runtime
}

fn collect_texts(node: &UiNode) -> Vec<&str> {
    let mut texts = Vec::new();
    collect_texts_into(node, &mut texts);
    texts
}

fn collect_texts_into<'a>(node: &'a UiNode, texts: &mut Vec<&'a str>) {
    match node {
        UiNode::Text(TextNode { text, .. }) => texts.push(text.as_str()),
        UiNode::View(view) => {
            for child in &view.children {
                collect_texts_into(child, texts);
            }
        }
        UiNode::Button(_) | UiNode::TextInput(_) => {}
    }
}

fn find_button<'a>(node: &'a UiNode, label: &str) -> Option<&'a ButtonNode> {
    match node {
        UiNode::Button(button) if button.text == label => Some(button),
        UiNode::Button(_) => None,
        UiNode::View(view) => view
            .children
            .iter()
            .find_map(|child| find_button(child, label)),
        UiNode::Text(_) | UiNode::TextInput(_) => None,
    }
}
