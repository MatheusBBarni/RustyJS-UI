use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::style::SizeValue;
use rustyjs_ui::vdom::{ButtonNode, FlatListNode, TextNode, UiNode};
use serde_json::Value;

const FLAT_LIST_APP: &str = include_str!("../examples/flat_list.js");

#[test]
fn flat_list_example_renders_values_from_data() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(FLAT_LIST_APP).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 720,
            height: 520
        } if title == "FlatList Example"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);
    let second_button = find_button(&tree, "Select 2").expect("expected Select 2 button");
    let flat_list = find_flat_list(&tree).expect("expected FlatList node");

    assert!(texts.iter().any(|text| *text == "FlatList Example"));
    assert!(texts
        .iter()
        .any(|text| *text == "Selected task: Nothing selected"));
    assert!(texts.iter().any(|text| *text == "1. Ship FlatList"));
    assert!(texts.iter().any(|text| *text == "Owner: Ada"));
    assert!(texts.iter().any(|text| *text == "2. Review JS bridge"));
    assert!(texts.iter().any(|text| *text == "Owner: Grace"));
    assert!(texts.iter().any(|text| *text == "3. Polish renderer"));
    assert_eq!(
        second_button
            .on_click
            .as_ref()
            .map(|callback| callback.id.as_str()),
        Some("cb_2")
    );
    assert_eq!(flat_list.style.layout.width, SizeValue::Fill);
    assert_eq!(flat_list.style.layout.height, SizeValue::Fill);
}

#[test]
fn flat_list_item_callback_re_renders_example() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(FLAT_LIST_APP).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let callback_id = find_button(&initial_tree, "Select 2")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Select 2 button callback")
        .id
        .clone();

    let payloads = runtime.trigger_callback(&callback_id, Value::Null).unwrap();

    assert_eq!(payloads.len(), 1);

    let updated_tree = payloads[0].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&updated_tree);

    assert!(texts
        .iter()
        .any(|text| *text == "Selected task: Review JS bridge"));
}

fn boot_runtime() -> JsRuntime {
    let mut runtime = JsRuntime::new().unwrap();
    assert!(runtime
        .eval_script(JsRuntime::bootstrap_source())
        .unwrap()
        .is_empty());
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

fn find_flat_list(node: &UiNode) -> Option<&FlatListNode> {
    match node {
        UiNode::FlatList(list) => Some(list),
        UiNode::Text(_) | UiNode::Button(_) | UiNode::TextInput(_) | UiNode::SelectInput(_) => None,
        _ => node.children().iter().find_map(find_flat_list),
    }
}
