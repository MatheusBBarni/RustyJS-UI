use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::style::{AlignItems, JustifyContent, SizeValue};
use rustyjs_ui::vdom::{ButtonNode, SelectInputNode, TextInputNode, TextNode, UiNode};
use serde_json::Value;

const FLEX_FORM_APP: &str = include_str!("../examples/flex_form.js");

#[test]
fn flex_form_example_renders_expected_initial_state() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(FLEX_FORM_APP).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 760,
            height: 560
        } if title == "Flex Form Example"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);
    let email_input = find_text_input(&tree, "Email").expect("expected Email input");
    let name_input = find_text_input(&tree, "Name").expect("expected Name input");
    let role_input = find_select_input(&tree, "Role").expect("expected Role select");
    let save_button = find_button(&tree, "Save").expect("expected Save button");

    match &tree {
        UiNode::View(root) => {
            assert_eq!(root.style.layout.width, SizeValue::Fill);
            assert_eq!(root.style.layout.height, SizeValue::Fill);
            assert_eq!(root.style.layout.justify_content, JustifyContent::Center);
            assert_eq!(root.style.layout.align_items, AlignItems::Center);

            match root.children.first() {
                Some(UiNode::View(card)) => {
                    assert_eq!(card.style.layout.spacing, 14.0);
                    assert_eq!(card.style.layout.justify_content, JustifyContent::Start);
                    assert_eq!(card.style.layout.align_items, AlignItems::Center);
                }
                other => panic!("expected inner card view, got {other:?}"),
            }
        }
        other => panic!("expected root view, got {other:?}"),
    }

    assert_eq!(email_input.value, "");
    assert_eq!(email_input.placeholder.as_deref(), Some("Email"));
    assert_eq!(name_input.value, "");
    assert_eq!(name_input.placeholder.as_deref(), Some("Name"));
    assert_eq!(role_input.value, "");
    assert_eq!(role_input.placeholder.as_deref(), Some("Role"));
    assert_eq!(role_input.options.len(), 3);
    assert!(save_button.on_click.is_some());
    assert!(texts.iter().any(|text| *text == "Profile Form"));
    assert!(texts
        .iter()
        .any(|text| *text == "Saved profile will appear here."));
}

#[test]
fn flex_form_save_click_re_renders_example() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(FLEX_FORM_APP).unwrap();
    let mut tree = payloads[1].typed_tree().unwrap().unwrap();

    let email_callback = find_text_input(&tree, "Email")
        .and_then(|input| input.on_change.as_ref())
        .expect("expected Email input callback")
        .id
        .clone();
    let payloads = runtime
        .trigger_callback(
            &email_callback,
            Value::String("ada@example.com".to_string()),
        )
        .unwrap();
    tree = payloads[0].typed_tree().unwrap().unwrap();

    let name_callback = find_text_input(&tree, "Name")
        .and_then(|input| input.on_change.as_ref())
        .expect("expected Name input callback")
        .id
        .clone();
    let payloads = runtime
        .trigger_callback(&name_callback, Value::String("Ada Lovelace".to_string()))
        .unwrap();
    tree = payloads[0].typed_tree().unwrap().unwrap();

    let role_callback = find_select_input(&tree, "Role")
        .and_then(|input| input.on_change.as_ref())
        .expect("expected Role select callback")
        .id
        .clone();
    let payloads = runtime
        .trigger_callback(&role_callback, Value::String("engineer".to_string()))
        .unwrap();
    tree = payloads[0].typed_tree().unwrap().unwrap();

    let save_callback = find_button(&tree, "Save")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Save button callback")
        .id
        .clone();
    let payloads = runtime
        .trigger_callback(&save_callback, Value::Null)
        .unwrap();

    assert_eq!(payloads.len(), 1);

    let updated_tree = payloads[0].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&updated_tree);

    assert!(texts
        .iter()
        .any(|text| { *text == "Saved Ada Lovelace (ada@example.com) as Engineer." }));
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

fn find_select_input<'a>(node: &'a UiNode, placeholder: &str) -> Option<&'a SelectInputNode> {
    match node {
        UiNode::SelectInput(input) if input.placeholder.as_deref() == Some(placeholder) => {
            Some(input)
        }
        UiNode::SelectInput(_) => None,
        UiNode::Text(_) | UiNode::Button(_) | UiNode::TextInput(_) => None,
        _ => node
            .children()
            .iter()
            .find_map(|child| find_select_input(child, placeholder)),
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
