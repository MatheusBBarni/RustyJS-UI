use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, TextNode, UiNode};
use std::path::{Path, PathBuf};

#[test]
fn module_entry_imports_local_component_file() {
    let (_runtime, payloads) =
        JsRuntime::startup_with_app_entry(&fixture_path("tests/fixtures/esm/save_button/main.js"))
            .unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 480,
            height: 320
        } if title == "Save Button Fixture"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);
    let button = find_button(&tree, "Save changes").expect("expected imported SaveButton");

    assert!(texts.iter().any(|text| *text == "Module entry"));
    assert_eq!(button.text, "Save changes");
}

#[test]
fn module_entry_resolves_nested_relative_imports() {
    let (_runtime, payloads) =
        JsRuntime::startup_with_app_entry(&fixture_path("tests/fixtures/esm/nested/main.js"))
            .unwrap();

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);

    assert!(texts.iter().any(|text| *text == "Nested modules"));
    assert!(find_button(&tree, "Save nested").is_some());
}

#[test]
fn repeated_imports_share_one_cached_module_instance() {
    let (_runtime, payloads) = JsRuntime::startup_with_app_entry(&fixture_path(
        "tests/fixtures/esm/repeated_imports/main.js",
    ))
    .unwrap();

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);

    assert!(texts.iter().any(|text| *text == "Shared count: 1"));
    assert!(find_button(&tree, "Save 1").is_some());
}

#[test]
fn module_entry_imports_rustyjs_ui_package() {
    let (_runtime, payloads) = JsRuntime::startup_with_app_entry(&fixture_path(
        "tests/fixtures/esm/package_import/main.js",
    ))
    .unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 480,
            height: 320
        } if title == "Package Import Fixture"
    ));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);

    assert!(texts.iter().any(|text| *text == "Package import fixture"));
    assert!(find_button(&tree, "Save package import").is_some());
}

#[test]
fn missing_import_reports_specifier_and_importer() {
    let error = JsRuntime::startup_with_app_entry(&fixture_path(
        "tests/fixtures/esm/errors/missing_import/main.js",
    ))
    .unwrap_err()
    .to_string();

    assert!(error.contains("failed to resolve import `./missing.js`"));
    assert!(error.contains("main.js"));
}

#[test]
fn non_relative_import_is_rejected() {
    let error = JsRuntime::startup_with_app_entry(&fixture_path(
        "tests/fixtures/esm/errors/non_relative/main.js",
    ))
    .unwrap_err()
    .to_string();

    assert!(error.contains("unsupported import `components/save_button.js`"));
    assert!(error.contains("only `./` and `../` specifiers are supported"));
}

#[test]
fn path_escape_import_is_rejected() {
    let error = JsRuntime::startup_with_app_entry(&fixture_path(
        "tests/fixtures/esm/errors/path_escape/root/main.js",
    ))
    .unwrap_err()
    .to_string();

    assert!(error.contains("import `../outside.js`"));
    assert!(error.contains("resolves outside the app root"));
}

fn fixture_path(relative: &str) -> PathBuf {
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
