use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, ModalNode, TextNode, UiNode};
use serde_json::Value;
use std::path::Path;

const MODAL_APP: &str = include_str!("../examples/modal.js");

#[test]
fn modal_example_renders_hidden_modal_initially() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(MODAL_APP).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 760,
            height: 560
        } if title == "Modal Example"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);
    let modal = find_modal(&tree).expect("expected Modal node");
    let open_button = find_button(&tree, "Open modal").expect("expected Open modal button");

    assert!(texts.iter().any(|text| *text == "Modal Example"));
    assert!(texts
        .iter()
        .any(|text| *text == "Open the modal to see the overlay host in action."));
    assert!(!modal.visible);
    assert!(modal.on_request_close.is_some());
    assert!(open_button.on_click.is_some());
}

#[test]
fn modal_open_button_re_renders_visible_modal() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(MODAL_APP).unwrap();
    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let callback_id = find_button(&tree, "Open modal")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Open modal callback")
        .id
        .clone();

    let payloads = runtime.trigger_callback(&callback_id, Value::Null).unwrap();

    assert_eq!(payloads.len(), 1);

    let updated_tree = payloads[0].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&updated_tree);
    let modal = find_modal(&updated_tree).expect("expected Modal node");
    let close_button = find_button(&updated_tree, "Close").expect("expected Close button");

    assert!(modal.visible);
    assert!(texts.iter().any(|text| *text == "Confirm Changes"));
    assert!(close_button.on_click.is_some());
}

#[test]
fn modal_example_starts_via_app_entry_path() {
    let (_runtime, payloads) =
        JsRuntime::startup_with_app_entry(&fixture_path("examples/modal.js")).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 760,
            height: 560
        } if title == "Modal Example"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let modal = find_modal(&tree).expect("expected Modal node");

    assert!(!modal.visible);
}

fn boot_runtime() -> JsRuntime {
    let mut runtime = JsRuntime::new().unwrap();
    assert!(runtime
        .eval_script(JsRuntime::bootstrap_source())
        .unwrap()
        .is_empty());
    runtime
}

fn fixture_path(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
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

fn find_modal(node: &UiNode) -> Option<&ModalNode> {
    match node {
        UiNode::Modal(modal) => Some(modal),
        UiNode::Text(_) | UiNode::Button(_) | UiNode::TextInput(_) | UiNode::SelectInput(_) => None,
        _ => node.children().iter().find_map(find_modal),
    }
}
