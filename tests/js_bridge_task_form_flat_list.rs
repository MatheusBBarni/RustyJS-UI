use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::style::SizeValue;
use rustyjs_ui::vdom::{ButtonNode, FlatListNode, TextInputNode, TextNode, UiNode};
use serde_json::Value;

const TASK_FORM_APP: &str = include_str!("../examples/task_form_flat_list.js");

#[test]
fn task_form_flat_list_example_renders_expected_initial_state() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(TASK_FORM_APP).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 760,
            height: 560
        } if title == "Task Form FlatList Example"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);
    let input = find_text_input(&tree, "Task name").expect("expected task name input");
    let add_button = find_button(&tree, "Add Task").expect("expected Add Task button");
    let flat_list = find_flat_list(&tree).expect("expected FlatList node");

    assert_eq!(input.value, "");
    assert!(input.on_change.is_some());
    assert!(add_button.on_click.is_some());
    assert_eq!(flat_list.style.layout.width, SizeValue::Fill);
    assert_eq!(flat_list.style.layout.height, SizeValue::Fill);
    assert!(texts.iter().any(|text| *text == "Task Form FlatList"));
    assert!(texts.iter().any(|text| *text == "Tasks: 2"));
    assert!(texts.iter().any(|text| *text == "1. Ship FlatList"));
    assert!(texts.iter().any(|text| *text == "Status: Pending"));
    assert!(texts
        .iter()
        .any(|text| *text == "2. Write integration tests"));
    assert!(texts.iter().any(|text| *text == "Status: Completed"));
}

#[test]
fn task_form_flat_list_add_task_re_renders_example() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(TASK_FORM_APP).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let input_callback = find_text_input(&initial_tree, "Task name")
        .and_then(|input| input.on_change.as_ref())
        .expect("expected task input callback")
        .id
        .clone();

    let payloads = runtime
        .trigger_callback(
            &input_callback,
            Value::String("Review release checklist".to_string()),
        )
        .unwrap();
    let updated_tree = payloads[0].typed_tree().unwrap().unwrap();
    let add_callback = find_button(&updated_tree, "Add Task")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Add Task callback")
        .id
        .clone();

    let payloads = runtime
        .trigger_callback(&add_callback, Value::Null)
        .unwrap();
    let added_tree = payloads[0].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&added_tree);
    let input = find_text_input(&added_tree, "Task name").expect("expected task name input");

    assert_eq!(input.value, "");
    assert!(texts.iter().any(|text| *text == "Tasks: 3"));
    assert!(texts
        .iter()
        .any(|text| *text == "3. Review release checklist"));
    assert!(find_button(&added_tree, "Complete 3").is_some());
    assert!(find_button(&added_tree, "Delete 3").is_some());
}

#[test]
fn task_form_flat_list_complete_and_delete_re_render_example() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(TASK_FORM_APP).unwrap();
    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let complete_callback = find_button(&initial_tree, "Complete 1")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Complete 1 callback")
        .id
        .clone();

    let payloads = runtime
        .trigger_callback(&complete_callback, Value::Null)
        .unwrap();
    let toggled_tree = payloads[0].typed_tree().unwrap().unwrap();
    let delete_callback = find_button(&toggled_tree, "Delete 2")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Delete 2 callback")
        .id
        .clone();

    assert!(find_button(&toggled_tree, "Undo 1").is_some());

    let payloads = runtime
        .trigger_callback(&delete_callback, Value::Null)
        .unwrap();
    let deleted_tree = payloads[0].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&deleted_tree);

    assert!(texts.iter().any(|text| *text == "Tasks: 1"));
    assert!(texts.iter().any(|text| *text == "1. Ship FlatList"));
    assert!(!texts
        .iter()
        .any(|text| *text == "2. Write integration tests"));
    assert!(find_button(&deleted_tree, "Undo 1").is_some());
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

fn find_flat_list(node: &UiNode) -> Option<&FlatListNode> {
    match node {
        UiNode::FlatList(list) => Some(list),
        UiNode::Text(_) | UiNode::Button(_) | UiNode::TextInput(_) | UiNode::SelectInput(_) => None,
        _ => node.children().iter().find_map(find_flat_list),
    }
}
