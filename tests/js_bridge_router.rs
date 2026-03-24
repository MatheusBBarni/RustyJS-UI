use rustyjs_ui::bridge::BridgePayload;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::vdom::{ButtonNode, TextNode, UiNode};
use serde_json::Value;
use std::path::{Path, PathBuf};

const ROUTER_APP_TEMPLATE: &str = r#"
const router = App.createRouter({
    initialPath: "__INITIAL_PATH__",
    routes: [
        {
            path: "/",
            render: ({ navigate }) =>
                View({
                    style: {
                        direction: "column",
                        gap: 8
                    },
                    children: [
                        Text({ text: "Home" }),
                        Button({
                            text: "Go settings",
                            onClick: () => navigate("/settings")
                        })
                    ]
                })
        },
        {
            path: "/settings",
            render: ({ navigate, back, forward }) =>
                View({
                    style: {
                        direction: "column",
                        gap: 8
                    },
                    children: [
                        Text({ text: "Settings" }),
                        Button({
                            text: "Go user 42",
                            onClick: () => navigate("/users/42?tab=activity")
                        }),
                        Button({
                            text: "Back",
                            onClick: back
                        }),
                        Button({
                            text: "Forward",
                            onClick: forward
                        })
                    ]
                })
        },
        {
            path: "/users/:id",
            render: ({ path, params, query, navigate, replace, back, forward }) =>
                View({
                    style: {
                        direction: "column",
                        gap: 8
                    },
                    children: [
                        Text({ text: `User ${params.id}` }),
                        Text({ text: `Path ${path}` }),
                        Text({ text: `Tab ${query.tab || ""}` }),
                        Button({
                            text: "Go settings",
                            onClick: () => navigate("/settings")
                        }),
                        Button({
                            text: "Replace user 99",
                            onClick: () => replace("/users/99?tab=overview")
                        }),
                        Button({
                            text: "Back",
                            onClick: back
                        }),
                        Button({
                            text: "Forward",
                            onClick: forward
                        })
                    ]
                })
        }
    ],
    notFound: ({ path }) =>
        View({
            style: {
                direction: "column",
                gap: 8
            },
            children: [
                Text({ text: "Route not found" }),
                Text({ text: `Missing ${path}` })
            ]
        })
});

function AppLayout() {
    return router.render();
}

App.run({
    title: "Router Example",
    windowSize: { width: 720, height: 420 },
    render: AppLayout
});
"#;

#[test]
fn router_initial_route_renders_expected_screen() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(&router_app_source("/settings")).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 720,
            height: 420
        } if title == "Router Example"
    ));
    assert!(matches!(&payloads[1], BridgePayload::UpdateVdom { .. }));

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);

    assert!(texts.iter().any(|text| *text == "Settings"));
    assert!(find_button(&tree, "Go user 42").is_some());
}

#[test]
fn router_params_and_query_are_available_to_route_render() {
    let mut runtime = boot_runtime();
    let payloads = runtime
        .eval_script(&router_app_source("/users/42?tab=activity"))
        .unwrap();

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);

    assert!(texts.iter().any(|text| *text == "User 42"));
    assert!(texts
        .iter()
        .any(|text| *text == "Path /users/42?tab=activity"));
    assert!(texts.iter().any(|text| *text == "Tab activity"));
}

#[test]
fn router_not_found_renders_fallback() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(&router_app_source("/missing")).unwrap();

    let tree = payloads[1].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&tree);

    assert!(texts.iter().any(|text| *text == "Route not found"));
    assert!(texts.iter().any(|text| *text == "Missing /missing"));
}

#[test]
fn router_navigation_replace_back_and_forward_emit_update_vdom() {
    let mut runtime = boot_runtime();
    let payloads = runtime.eval_script(&router_app_source("/")).unwrap();
    let home_tree = payloads[1].typed_tree().unwrap().unwrap();
    let go_settings = find_button(&home_tree, "Go settings")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Go settings callback")
        .id
        .clone();

    let payloads = runtime.trigger_callback(&go_settings, Value::Null).unwrap();
    assert_eq!(payloads.len(), 1);
    assert!(matches!(&payloads[0], BridgePayload::UpdateVdom { .. }));

    let settings_tree = payloads[0].typed_tree().unwrap().unwrap();
    let go_user_42 = find_button(&settings_tree, "Go user 42")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Go user 42 callback")
        .id
        .clone();

    let payloads = runtime.trigger_callback(&go_user_42, Value::Null).unwrap();
    assert_eq!(payloads.len(), 1);
    assert!(matches!(&payloads[0], BridgePayload::UpdateVdom { .. }));

    let user_tree = payloads[0].typed_tree().unwrap().unwrap();
    let replace_user_99 = find_button(&user_tree, "Replace user 99")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Replace user 99 callback")
        .id
        .clone();

    let payloads = runtime
        .trigger_callback(&replace_user_99, Value::Null)
        .unwrap();
    assert_eq!(payloads.len(), 1);
    assert!(matches!(&payloads[0], BridgePayload::UpdateVdom { .. }));

    let replaced_tree = payloads[0].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&replaced_tree);
    assert!(texts.iter().any(|text| *text == "User 99"));
    assert!(texts
        .iter()
        .any(|text| *text == "Path /users/99?tab=overview"));
    assert!(texts.iter().any(|text| *text == "Tab overview"));

    let back = find_button(&replaced_tree, "Back")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Back callback")
        .id
        .clone();

    let payloads = runtime.trigger_callback(&back, Value::Null).unwrap();
    assert_eq!(payloads.len(), 1);
    assert!(matches!(&payloads[0], BridgePayload::UpdateVdom { .. }));

    let settings_again = payloads[0].typed_tree().unwrap().unwrap();
    let forward = find_button(&settings_again, "Forward")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Forward callback")
        .id
        .clone();

    let payloads = runtime.trigger_callback(&forward, Value::Null).unwrap();
    assert_eq!(payloads.len(), 1);
    assert!(matches!(&payloads[0], BridgePayload::UpdateVdom { .. }));

    let forward_tree = payloads[0].typed_tree().unwrap().unwrap();
    let texts = collect_texts(&forward_tree);

    assert!(texts.iter().any(|text| *text == "User 99"));
    assert!(texts
        .iter()
        .any(|text| *text == "Path /users/99?tab=overview"));
}

#[test]
fn router_example_entry_works_with_multi_file_module_loader() {
    let (mut runtime, payloads) =
        JsRuntime::startup_with_app_entry(&fixture_path("examples/router_demo/main.js")).unwrap();

    assert_eq!(payloads.len(), 2);
    assert!(matches!(
        &payloads[0],
        BridgePayload::InitWindow {
            title,
            width: 840,
            height: 620
        } if title == "Router Demo Example"
    ));

    let initial_tree = payloads[1].typed_tree().unwrap().unwrap();
    let initial_texts = collect_texts(&initial_tree);

    assert!(initial_texts
        .iter()
        .any(|text| *text == "Router state path: /"));
    assert!(initial_texts.iter().any(|text| *text == "Router Demo"));

    let open_project_alpha = find_button(&initial_tree, "Open project alpha")
        .and_then(|button| button.on_click.as_ref())
        .expect("expected Open project alpha callback")
        .id
        .clone();

    let payloads = runtime
        .trigger_callback(&open_project_alpha, Value::Null)
        .unwrap();

    assert_eq!(payloads.len(), 1);
    assert!(matches!(&payloads[0], BridgePayload::UpdateVdom { .. }));

    let project_tree = payloads[0].typed_tree().unwrap().unwrap();
    let project_texts = collect_texts(&project_tree);

    assert!(project_texts
        .iter()
        .any(|text| *text == "Router state path: /projects/alpha?tab=overview"));
    assert!(project_texts
        .iter()
        .any(|text| *text == "Project alpha"));
    assert!(project_texts
        .iter()
        .any(|text| *text == "Query tab = overview"));
}

fn boot_runtime() -> JsRuntime {
    let mut runtime = JsRuntime::new().unwrap();
    assert!(runtime
        .eval_script(JsRuntime::bootstrap_source())
        .unwrap()
        .is_empty());
    runtime
}

fn router_app_source(initial_path: &str) -> String {
    ROUTER_APP_TEMPLATE.replace("__INITIAL_PATH__", initial_path)
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
