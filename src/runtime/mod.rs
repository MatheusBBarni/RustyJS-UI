mod fetch;
mod jsengine;
mod modules;

use crate::bridge::BridgePayload;
use anyhow::{anyhow, Context as AnyhowContext, Result};
use boa_engine::{
    builtins::promise::PromiseState,
    module::{Module, ModuleLoader},
    Context as BoaContext, JsValue, NativeFunction, Source,
};
use fetch::{FetchCompletion, FetchMethod, FetchRequest, FetchTransport};
use modules::AppModuleLoader;
use serde::Deserialize;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

thread_local! {
    static OUTBOUND_QUEUE: RefCell<VecDeque<String>> = const { RefCell::new(VecDeque::new()) };
}

/// Boa runtime host for the RustyJS-UI bridge.
///
/// The runtime evaluates bundled JS files that expose the JS-side helpers
/// (`App`, `View`, `Text`, `Button`, `TextInput`, `SelectInput`, and `__SEND_TO_RUST__`)
/// plus a sample counter app used for the MVP.
#[derive(Debug)]
pub struct JsRuntime {
    context: BoaContext<'static>,
    module_loader: Rc<AppModuleLoader>,
    fetch_transport: FetchTransport,
    pending_fetches: HashMap<u64, String>,
}

impl JsRuntime {
    /// Creates a new Boa context and installs the host bridge helpers.
    pub fn new() -> Result<Self> {
        let module_loader = Rc::new(AppModuleLoader::new());
        let loader: Rc<dyn ModuleLoader> = module_loader.clone();
        let mut context = BoaContext::builder()
            .module_loader(loader)
            .build()
            .map_err(|err| anyhow!("failed to build JS runtime context: {err}"))?;
        let fetch_transport = FetchTransport::new()?;
        context
            .register_global_callable(
                "__RUSTYJS_NATIVE_CAPTURE__",
                1,
                NativeFunction::from_fn_ptr(native_capture),
            )
            .map_err(|err| anyhow!("failed to register native capture callback: {err}"))?;

        Ok(Self {
            context,
            module_loader,
            fetch_transport,
            pending_fetches: HashMap::new(),
        })
    }

    /// Boots the bundled runtime scripts and loads the sample counter app.
    ///
    /// Returns the runtime plus the payloads produced during initialization.
    pub fn startup() -> Result<(Self, Vec<BridgePayload>)> {
        let bootstrap = jsengine::bootstrap();
        let sample_app = jsengine::counter_app();
        let mut runtime = Self::new()?;
        let mut initial_payloads =
            runtime.eval_script_with_path(bootstrap.source, Some(bootstrap.path))?;
        initial_payloads
            .extend(runtime.eval_script_with_path(sample_app.source, Some(sample_app.path))?);
        Ok((runtime, initial_payloads))
    }

    /// Boots the embedded runtime and loads the provided app source.
    ///
    /// Returns the runtime plus the payloads produced during initialization.
    pub fn startup_with_app_source(app_source: &str) -> Result<(Self, Vec<BridgePayload>)> {
        let mut runtime = Self::new()?;
        let bootstrap = jsengine::bootstrap();
        let mut initial_payloads =
            runtime.eval_script_with_path(bootstrap.source, Some(bootstrap.path))?;
        initial_payloads.extend(runtime.eval_script(app_source)?);
        Ok((runtime, initial_payloads))
    }

    /// Boots the embedded runtime and loads the provided app entry as an ECMAScript module.
    ///
    /// Returns the runtime plus the payloads produced during initialization.
    pub fn startup_with_app_entry(entry_path: &Path) -> Result<(Self, Vec<BridgePayload>)> {
        let mut runtime = Self::new()?;
        let bootstrap = jsengine::bootstrap();
        let mut initial_payloads =
            runtime.eval_script_with_path(bootstrap.source, Some(bootstrap.path))?;
        let canonical_entry = entry_path
            .canonicalize()
            .with_context(|| format!("failed to read app entry `{}`", entry_path.display()))?;
        let entry_source = fs::read_to_string(&canonical_entry).with_context(|| {
            format!(
                "failed to load app entry source `{}`",
                canonical_entry.display()
            )
        })?;

        if uses_static_esm_syntax(&entry_source) {
            initial_payloads.extend(runtime.eval_module_entry(canonical_entry.as_path())?);
        } else {
            let entry_path = canonical_entry.to_string_lossy().into_owned();
            initial_payloads
                .extend(runtime.eval_script_with_path(&entry_source, Some(entry_path.as_str()))?);
        }

        Ok((runtime, initial_payloads))
    }

    /// Evaluates additional JS source and returns any payloads emitted by it.
    pub fn eval_script(&mut self, source: &str) -> Result<Vec<BridgePayload>> {
        self.eval_script_with_path(source, None)
    }

    /// Evaluates an app entry file as an ECMAScript module and returns any emitted payloads.
    pub fn eval_module_entry(&mut self, entry_path: &Path) -> Result<Vec<BridgePayload>> {
        let (canonical_entry, module) = self
            .module_loader
            .prepare_entry_module(entry_path, &mut self.context)?;
        self.eval_module(canonical_entry.as_path(), &module)
    }

    fn eval_script_with_path(
        &mut self,
        source: &str,
        source_path: Option<&str>,
    ) -> Result<Vec<BridgePayload>> {
        self.context
            .eval(Source::from_reader(
                source.as_bytes(),
                source_path.map(Path::new),
            ))
            .map_err(|err| anyhow!("failed to evaluate JS source: {err}"))?;
        self.context.run_jobs();
        self.drain_payloads()
    }

    fn eval_module(&mut self, entry_path: &Path, module: &Module) -> Result<Vec<BridgePayload>> {
        let promise = module
            .load_link_evaluate(&mut self.context)
            .map_err(|err| anyhow!("failed to load JS module `{}`: {err}", entry_path.display()))?;
        self.context.run_jobs();

        match promise.state().map_err(|err| {
            anyhow!(
                "failed to inspect JS module `{}`: {err}",
                entry_path.display()
            )
        })? {
            PromiseState::Fulfilled(_) => self.drain_payloads(),
            PromiseState::Rejected(reason) => Err(anyhow!(
                "failed to evaluate JS module `{}`: {}",
                entry_path.display(),
                reason.display()
            )),
            PromiseState::Pending => Err(anyhow!(
                "JS module `{}` is still pending after startup; top-level await is not supported",
                entry_path.display()
            )),
        }
    }

    /// Drains outbound bridge payloads that the JS runtime queued.
    ///
    /// Fetch requests are intercepted and dispatched to the async transport.
    pub fn drain_payloads(&mut self) -> Result<Vec<BridgePayload>> {
        let mut payloads = Vec::new();

        loop {
            let pending = take_outbound_messages();
            if pending.is_empty() {
                break;
            }

            for message in pending {
                match parse_outbound_message(&message)? {
                    OutboundMessage::Payload(payload) => payloads.push(payload),
                    OutboundMessage::FetchRequest(request) => {
                        self.submit_fetch_request(request)?;
                    }
                }
            }
        }

        Ok(payloads)
    }

    /// Triggers a registered JS callback and returns any payloads emitted by it.
    pub fn trigger_callback(
        &mut self,
        callback_id: &str,
        payload: Value,
    ) -> Result<Vec<BridgePayload>> {
        let callback_id =
            serde_json::to_string(callback_id).context("failed to encode callback id")?;
        let payload =
            serde_json::to_string(&payload).context("failed to encode callback payload")?;
        let script = format!("globalThis.RustBridge.trigger({callback_id}, {payload});");
        self.context
            .eval(Source::from_bytes(script.as_str()))
            .map_err(|err| anyhow!("failed to trigger JS callback: {err}"))?;
        self.context.run_jobs();
        self.drain_payloads()
    }

    /// Polls for completed async fetch requests and resolves any pending JS promises.
    pub fn poll_async(&mut self) -> Result<Vec<BridgePayload>> {
        if self.pending_fetches.is_empty() {
            return Ok(Vec::new());
        }

        let mut payloads = Vec::new();

        for completion in self.fetch_transport.drain_completions() {
            let Some(request_id) = self.pending_fetches.remove(&completion.request_id()) else {
                continue;
            };

            match completion {
                FetchCompletion::Response(response) if (200..400).contains(&response.status) => {
                    self.resolve_fetch(&request_id, &response.body)?;
                }
                FetchCompletion::Response(response) => {
                    self.reject_fetch(&request_id, &format!("HTTP {}", response.status_text))?;
                }
                FetchCompletion::Error(error) => {
                    self.reject_fetch(&request_id, &error.message)?;
                }
            }

            payloads.extend(self.drain_payloads()?);
        }

        Ok(payloads)
    }

    pub fn has_pending_fetches(&self) -> bool {
        !self.pending_fetches.is_empty()
    }

    /// Returns the bundled bootstrap script.
    pub fn bootstrap_source() -> &'static str {
        jsengine::bootstrap().source
    }

    /// Returns the bundled sample counter app.
    pub fn sample_counter_app_source() -> &'static str {
        jsengine::counter_app().source
    }

    fn submit_fetch_request(&mut self, request: OutboundFetchRequest) -> Result<()> {
        let request_id = request.request_id.clone();
        let transport_request = request
            .to_transport_request()
            .with_context(|| format!("failed to prepare fetch request: {request_id}"));

        match transport_request {
            Ok(transport_request) => match self.fetch_transport.submit(transport_request) {
                Ok(internal_id) => {
                    self.pending_fetches.insert(internal_id, request_id);
                }
                Err(error) => {
                    self.reject_fetch(&request_id, &error.to_string())?;
                }
            },
            Err(error) => {
                self.reject_fetch(&request_id, &error.to_string())?;
            }
        }

        Ok(())
    }

    fn resolve_fetch(&mut self, request_id: &str, body: &str) -> Result<()> {
        let request_id =
            serde_json::to_string(request_id).context("failed to encode fetch request id")?;
        let body = serde_json::to_string(body).context("failed to encode fetch body")?;
        let script = format!("globalThis.RustBridge.resolveFetch({request_id}, {body});");
        self.context
            .eval(Source::from_bytes(script.as_str()))
            .map_err(|err| anyhow!("failed to resolve fetch promise: {err}"))?;
        self.context.run_jobs();
        Ok(())
    }

    fn reject_fetch(&mut self, request_id: &str, message: &str) -> Result<()> {
        let request_id =
            serde_json::to_string(request_id).context("failed to encode fetch request id")?;
        let message = serde_json::to_string(message).context("failed to encode fetch error")?;
        let script = format!("globalThis.RustBridge.rejectFetch({request_id}, {message});");
        self.context
            .eval(Source::from_bytes(script.as_str()))
            .map_err(|err| anyhow!("failed to reject fetch promise: {err}"))?;
        self.context.run_jobs();
        Ok(())
    }
}

#[derive(Debug)]
enum OutboundMessage {
    Payload(BridgePayload),
    FetchRequest(OutboundFetchRequest),
}

#[derive(Debug, Deserialize)]
struct OutboundFetchRequest {
    #[serde(rename = "requestId")]
    request_id: String,
    url: String,
    #[serde(default = "default_fetch_method")]
    method: String,
    #[serde(default)]
    headers: BTreeMap<String, String>,
    #[serde(default)]
    body: Option<String>,
}

impl OutboundFetchRequest {
    fn to_transport_request(&self) -> Result<FetchRequest> {
        let method = match self.method.trim().to_ascii_uppercase().as_str() {
            "GET" => FetchMethod::Get,
            "POST" => FetchMethod::Post,
            "PUT" => FetchMethod::Put,
            "DELETE" => FetchMethod::Delete,
            other => return Err(anyhow!("unsupported fetch method: {other}")),
        };

        let mut request = FetchRequest::new(self.url.clone())
            .with_method(method)
            .with_timeout(Duration::from_secs(30));

        for (name, value) in &self.headers {
            request = request.with_header(name.clone(), value.clone());
        }

        if let Some(body) = &self.body {
            request = request.with_body(body.clone());
        }

        Ok(request)
    }
}

fn default_fetch_method() -> String {
    "GET".to_string()
}

fn uses_static_esm_syntax(source: &str) -> bool {
    source.lines().map(str::trim_start).any(|line| {
        line.starts_with("import ")
            || line.starts_with("import{")
            || line.starts_with("import*")
            || line.starts_with("import\"")
            || line.starts_with("import'")
            || line.starts_with("export ")
            || line.starts_with("export{")
            || line.starts_with("export*")
    })
}

fn take_outbound_messages() -> Vec<String> {
    OUTBOUND_QUEUE.with(|queue| queue.borrow_mut().drain(..).collect::<Vec<_>>())
}

fn parse_outbound_message(input: &str) -> Result<OutboundMessage> {
    let value: Value = serde_json::from_str(input)
        .with_context(|| format!("failed to parse outbound bridge message: {input}"))?;
    let action = value
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("outbound bridge message missing action: {input}"))?;

    if action == "FETCH_REQUEST" {
        return serde_json::from_value(value)
            .map(OutboundMessage::FetchRequest)
            .with_context(|| format!("failed to parse fetch request payload: {input}"));
    }

    BridgePayload::parse_json(value)
        .map(OutboundMessage::Payload)
        .with_context(|| format!("failed to parse bridge payload: {input}"))
}

fn native_capture(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut BoaContext<'_>,
) -> boa_engine::JsResult<JsValue> {
    let payload = args
        .first()
        .cloned()
        .unwrap_or_else(JsValue::undefined)
        .to_string(context)?
        .to_std_string_escaped();
    OUTBOUND_QUEUE.with(|queue| queue.borrow_mut().push_back(payload));
    Ok(JsValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{AlignItems, FlexDirection, JustifyContent, SizeValue};
    use crate::vdom::UiNode;
    use serde_json::Value;

    #[test]
    fn eval_script_emits_prd_style_bridge_payloads() {
        let mut runtime = JsRuntime::new().unwrap();
        let bootstrap = jsengine::bootstrap();

        assert!(runtime
            .eval_script_with_path(bootstrap.source, Some(bootstrap.path))
            .unwrap()
            .is_empty());

        let payloads = runtime
            .eval_script(
                r#"
let counter = 0;

function increment() {
    counter += 1;
    App.requestRender();
}

function AppLayout() {
    return Button({
        text: `Count is: ${counter}`,
        onClick: increment,
        style: { padding: 10, backgroundColor: '#007AFF' }
    });
}

App.run({
    title: 'Bridge Test',
    windowSize: { width: 320, height: 200 },
    render: AppLayout
});
"#,
            )
            .unwrap();

        assert_eq!(payloads.len(), 2);
        assert!(matches!(
            &payloads[0],
            BridgePayload::InitWindow {
                title,
                width: 320,
                height: 200
            } if title == "Bridge Test"
        ));

        match payloads[1].typed_tree().unwrap() {
            Some(UiNode::Button(button)) => {
                assert_eq!(button.text, "Count is: 0");
                assert_eq!(
                    button
                        .on_click
                        .as_ref()
                        .map(|callback| callback.id.as_str()),
                    Some("cb_1")
                );
            }
            other => panic!("expected button tree payload, got {other:?}"),
        }
    }

    #[test]
    fn trigger_callback_re_renders_with_updated_vdom() {
        let mut runtime = JsRuntime::new().unwrap();
        let bootstrap = jsengine::bootstrap();

        assert!(runtime
            .eval_script_with_path(bootstrap.source, Some(bootstrap.path))
            .unwrap()
            .is_empty());
        runtime
            .eval_script(
                r#"
let counter = 0;

function increment() {
    counter += 1;
    App.requestRender();
}

function AppLayout() {
    return View({
        children: [
            Text({ text: `Count is: ${counter}` }),
            Button({ text: 'Increment', onClick: increment })
        ]
    });
}

App.run({
    title: 'Counter Test',
    render: AppLayout
});
"#,
            )
            .unwrap();

        let payloads = runtime.trigger_callback("cb_1", Value::Null).unwrap();

        assert_eq!(payloads.len(), 1);

        match payloads[0].typed_tree().unwrap() {
            Some(UiNode::View(view)) => match view.children.first() {
                Some(UiNode::Text(text)) => assert_eq!(text.text, "Count is: 1"),
                other => panic!("expected first child text node, got {other:?}"),
            },
            other => panic!("expected view tree payload, got {other:?}"),
        }
    }

    #[test]
    fn previous_generation_text_input_callback_remains_valid() {
        let mut runtime = JsRuntime::new().unwrap();
        let bootstrap = jsengine::bootstrap();

        assert!(runtime
            .eval_script_with_path(bootstrap.source, Some(bootstrap.path))
            .unwrap()
            .is_empty());

        let initial_payloads = runtime
            .eval_script(
                r#"
let value = '';

function handleChange(nextValue) {
    value = nextValue;
    App.requestRender();
}

function AppLayout() {
    return TextInput({
        value,
        placeholder: 'Type here',
        onChange: handleChange
    });
}

App.run({
    title: 'Input Callback Test',
    render: AppLayout
});
"#,
            )
            .unwrap();

        let initial_callback_id = match initial_payloads[1].typed_tree().unwrap() {
            Some(UiNode::TextInput(input)) => input
                .on_change
                .as_ref()
                .map(|callback| callback.id.clone())
                .expect("expected text input callback"),
            other => panic!("expected text input tree payload, got {other:?}"),
        };

        let first_update = runtime
            .trigger_callback(&initial_callback_id, Value::String("a".to_string()))
            .unwrap();

        match first_update[0].typed_tree().unwrap() {
            Some(UiNode::TextInput(input)) => assert_eq!(input.value, "a"),
            other => panic!("expected text input tree payload, got {other:?}"),
        }

        let second_update = runtime
            .trigger_callback(&initial_callback_id, Value::String("ab".to_string()))
            .unwrap();

        match second_update[0].typed_tree().unwrap() {
            Some(UiNode::TextInput(input)) => assert_eq!(input.value, "ab"),
            other => panic!("expected text input tree payload, got {other:?}"),
        }
    }

    #[test]
    fn bootstrap_normalizes_web_flex_style_aliases() {
        let mut runtime = JsRuntime::new().unwrap();
        let bootstrap = jsengine::bootstrap();

        assert!(runtime
            .eval_script_with_path(bootstrap.source, Some(bootstrap.path))
            .unwrap()
            .is_empty());

        let payloads = runtime
            .eval_script(
                r#"
function AppLayout() {
    return View({
        style: {
            flexDirection: 'column',
            gap: 12,
            justifyContent: 'space-between',
            alignItems: 'flex-end',
            width: 'fill',
            height: 'fill'
        },
        children: [
            Text({ text: 'Top' }),
            Text({ text: 'Bottom' })
        ]
    });
}

App.run({
    title: 'Flex Alias Test',
    render: AppLayout
});
"#,
            )
            .unwrap();

        assert_eq!(payloads.len(), 2);

        match payloads[1].typed_tree().unwrap() {
            Some(UiNode::View(view)) => {
                assert_eq!(view.style.layout.flex_direction, FlexDirection::Column);
                assert_eq!(view.style.layout.spacing, 12.0);
                assert_eq!(
                    view.style.layout.justify_content,
                    JustifyContent::SpaceBetween
                );
                assert_eq!(view.style.layout.align_items, AlignItems::End);
                assert_eq!(view.style.layout.width, SizeValue::Fill);
                assert_eq!(view.style.layout.height, SizeValue::Fill);
            }
            other => panic!("expected view tree payload, got {other:?}"),
        }
    }

    #[test]
    fn esm_detection_only_flags_static_imports_and_exports() {
        assert!(!uses_static_esm_syntax(
            "function AppLayout() {\n  return View({ children: [] });\n}\n"
        ));
        assert!(uses_static_esm_syntax(
            "import { SaveButton } from './save_button.js';"
        ));
        assert!(uses_static_esm_syntax("export function SaveButton() {}"));
    }
}
