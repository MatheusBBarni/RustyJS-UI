use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, FlatListNode, SelectInputNode, TextNode, UiNode};
use serde_json::Value;
use std::thread;
use std::time::Duration;

#[test]
fn native_aliases_render_existing_nodes() {
    let mut runtime = boot_runtime();
    let payloads = runtime
        .eval_script(
            r#"
let value = '';

function AppLayout() {
    return View({
        children: [
            NativeSelect({
                value,
                options: ['a', 'b'],
                onChange: (nextValue) => {
                    value = nextValue;
                    App.requestRender();
                }
            }),
            NativeList({
                data: ['x', 'y'],
                renderItem: ({ item }) => Text({ text: item })
            })
        ]
    });
}

App.run({
    title: 'Alias Test',
    render: AppLayout
});
"#,
        )
        .unwrap();

    let tree = payloads[1].typed_tree().unwrap().unwrap();

    assert!(find_select_input(&tree).is_some());
    assert!(find_flat_list(&tree).is_some());
}

#[test]
fn tabs_helper_switches_selected_content() {
    let mut runtime = boot_runtime();
    let payloads = runtime
        .eval_script(
            r#"
let tab = 'overview';

function AppLayout() {
    return Tabs({
        value: tab,
        onChange: (nextValue) => {
            tab = nextValue;
            App.requestRender();
        },
        tabs: [
            {
                label: 'Overview',
                value: 'overview',
                content: Text({ text: 'Overview panel' })
            },
            {
                label: 'Settings',
                value: 'settings',
                content: Text({ text: 'Settings panel' })
            }
        ]
    });
}

App.run({
    title: 'Tabs Test',
    render: AppLayout
});
"#,
        )
        .unwrap();

    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    assert!(collect_texts(&initial_tree)
        .iter()
        .any(|text| *text == "Overview panel"));

    let callback_id = find_button(&initial_tree, "Settings")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Settings callback")
        .id
        .clone();

    let payloads = runtime.trigger_callback(&callback_id, Value::Null).unwrap();
    let updated_tree = payloads[0].typed_tree().unwrap().unwrap();
    assert!(collect_texts(&updated_tree)
        .iter()
        .any(|text| *text == "Settings panel"));
}

#[test]
fn navigation_helper_and_dev_warnings_are_available() {
    let mut runtime = boot_runtime();
    let payloads = runtime
        .eval_script(
            r#"
function AppLayout() {
    const matched = Navigation.matchRoute('/users/:id', '/users/7');

    return View({
        children: [
            Text({ text: `Path ${Navigation.normalizePath('/users//7/?tab=activity#hash')}` }),
            Text({ text: `User ${matched.id}` }),
            TextInput({
                onChange: () => {}
            })
        ]
    });
}

App.run({
    title: 'Navigation Test',
    render: AppLayout
});
"#,
        )
        .unwrap();

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);

    assert!(texts.iter().any(|text| *text == "Path /users/7?tab=activity"));
    assert!(texts.iter().any(|text| *text == "User 7"));
    assert!(runtime
        .diagnostics()
        .warnings
        .iter()
        .any(|warning| warning.message.contains("controlled-only")));
}

#[test]
fn toast_auto_dismisses_after_timer() {
    let mut runtime = boot_runtime();
    let payloads = runtime
        .eval_script(
            r#"
function showToast() {
    Toast.show({
        message: 'Saved successfully',
        durationMs: 1
    });
}

function AppLayout() {
    return View({
        children: [
            Text({ text: 'Toast Demo' }),
            Button({ text: 'Notify', onClick: showToast })
        ]
    });
}

App.run({
    title: 'Toast Test',
    render: AppLayout
});
"#,
        )
        .unwrap();

    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let callback_id = find_button(&initial_tree, "Notify")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Notify callback")
        .id
        .clone();

    let payloads = runtime.trigger_callback(&callback_id, Value::Null).unwrap();
    let toast_tree = payloads[0].typed_tree().unwrap().unwrap();
    assert!(collect_texts(&toast_tree)
        .iter()
        .any(|text| *text == "Saved successfully"));

    let updated_payloads = wait_for_async_payloads(&mut runtime);
    let updated_tree = updated_payloads[0].typed_tree().unwrap().unwrap();
    assert!(!collect_texts(&updated_tree)
        .iter()
        .any(|text| *text == "Saved successfully"));
}

fn boot_runtime() -> JsRuntime {
    let mut runtime = JsRuntime::new().unwrap();
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

fn find_select_input(node: &UiNode) -> Option<&SelectInputNode> {
    match node {
        UiNode::SelectInput(select) => Some(select),
        UiNode::Text(_) | UiNode::Button(_) | UiNode::TextInput(_) => None,
        _ => node.children().iter().find_map(find_select_input),
    }
}

fn find_flat_list(node: &UiNode) -> Option<&FlatListNode> {
    match node {
        UiNode::FlatList(list) => Some(list),
        UiNode::Text(_) | UiNode::Button(_) | UiNode::TextInput(_) | UiNode::SelectInput(_) => None,
        _ => node.children().iter().find_map(find_flat_list),
    }
}
