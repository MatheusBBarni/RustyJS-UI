use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{TextInputNode, TextNode, UiNode};
use serde_json::Value;

const TEXT_INPUT_APP: &str = include_str!("../examples/text_input_echo.js");

#[test]
fn text_input_example_renders_expected_initial_state() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(TEXT_INPUT_APP).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 520,
            height: 260
        } if title == "TextInput Example"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let input = find_text_input(&tree).expect("expected TextInput node");
    let texts = collect_texts(&tree);

    assert_eq!(input.value, "");
    assert_eq!(input.placeholder.as_deref(), Some("Type something"));
    assert_eq!(
        input
            .on_change
            .as_ref()
            .map(|callback| callback.id.as_str()),
        Some("cb_1")
    );
    assert!(texts.iter().any(|text| *text == "TextInput Example"));
    assert!(texts.iter().any(|text| *text == "Current value: "));
}

#[test]
fn text_input_change_re_renders_example() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(TEXT_INPUT_APP).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let input = find_text_input(&initial_tree).expect("expected TextInput node");
    let callback_id = input.on_change.as_ref().unwrap().id.clone();

    let payloads = runtime
        .trigger_callback(&callback_id, Value::String("RustyJS".to_string()))
        .unwrap();

    assert_eq!(payloads.len(), 1);

    let updated_tree = payloads[0].typed_tree().unwrap().unwrap();
    let updated_input = find_text_input(&updated_tree).expect("expected TextInput node");
    let texts = collect_texts(&updated_tree);

    assert_eq!(updated_input.value, "RustyJS");
    assert!(texts.iter().any(|text| *text == "Current value: RustyJS"));
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
        UiNode::View(view) => {
            for child in &view.children {
                collect_texts_into(child, texts);
            }
        }
        UiNode::Button(_) | UiNode::TextInput(_) | UiNode::SelectInput(_) => {}
    }
}

fn find_text_input(node: &UiNode) -> Option<&TextInputNode> {
    match node {
        UiNode::TextInput(input) => Some(input),
        UiNode::View(view) => view.children.iter().find_map(find_text_input),
        UiNode::Text(_) | UiNode::Button(_) | UiNode::SelectInput(_) => None,
    }
}
