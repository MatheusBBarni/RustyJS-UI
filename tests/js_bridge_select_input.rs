use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{SelectInputNode, TextNode, UiNode};
use serde_json::Value;

const SELECT_INPUT_APP: &str = include_str!("../examples/select_input_echo.js");

#[test]
fn select_input_example_renders_expected_initial_state() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(SELECT_INPUT_APP).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 520,
            height: 260
        } if title == "SelectInput Example"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let select = find_select_input(&tree).expect("expected SelectInput node");
    let texts = collect_texts(&tree);

    assert_eq!(select.value, "");
    assert_eq!(select.placeholder.as_deref(), Some("Choose a language"));
    assert_eq!(
        select
            .on_change
            .as_ref()
            .map(|callback| callback.id.as_str()),
        Some("cb_1")
    );
    assert_eq!(
        select
            .options
            .iter()
            .map(|option| (option.label.as_str(), option.value.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("Rust", "rust"),
            ("JavaScript", "javascript"),
            ("TypeScript", "typescript")
        ]
    );
    assert!(texts.iter().any(|text| *text == "SelectInput Example"));
    assert!(texts
        .iter()
        .any(|text| *text == "Selected label: Nothing selected"));
}

#[test]
fn select_input_change_re_renders_example() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(SELECT_INPUT_APP).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let select = find_select_input(&initial_tree).expect("expected SelectInput node");
    let callback_id = select.on_change.as_ref().unwrap().id.clone();

    let payloads = runtime
        .trigger_callback(&callback_id, Value::String("typescript".to_string()))
        .unwrap();

    assert_eq!(payloads.len(), 1);

    let updated_tree = payloads[0].typed_tree().unwrap().unwrap();
    let updated_select = find_select_input(&updated_tree).expect("expected SelectInput node");
    let texts = collect_texts(&updated_tree);

    assert_eq!(updated_select.value, "typescript");
    assert!(texts
        .iter()
        .any(|text| *text == "Selected label: TypeScript"));
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

fn find_select_input(node: &UiNode) -> Option<&SelectInputNode> {
    match node {
        UiNode::SelectInput(select) => Some(select),
        UiNode::View(view) => view.children.iter().find_map(find_select_input),
        UiNode::Text(_) | UiNode::Button(_) | UiNode::TextInput(_) => None,
    }
}
