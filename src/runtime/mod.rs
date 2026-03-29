mod fetch;
mod jsengine;
mod modules;
#[path = "storage.rs"]
mod storage;
#[path = "timer.rs"]
mod timer;

use crate::bridge::{coalesce_payloads, BridgePayload};
use crate::perf;
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
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};
use storage::{StorageCompletion, StorageRequest, StorageTransport};
use timer::{TimerCompletion, TimerTransport};

thread_local! {
    static OUTBOUND_QUEUE: RefCell<VecDeque<String>> = const { RefCell::new(VecDeque::new()) };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWarning {
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeDiagnostics {
    pub eval_script_calls: u64,
    pub eval_module_calls: u64,
    pub callback_calls: u64,
    pub poll_async_calls: u64,
    pub drain_payload_calls: u64,
    pub bridge_messages_seen: u64,
    pub bridge_parse_time: Duration,
    pub eval_script_time: Duration,
    pub eval_module_time: Duration,
    pub callback_time: Duration,
    pub poll_async_time: Duration,
    pub drain_payload_time: Duration,
    pub coalesced_update_batches: u64,
    pub warnings: Vec<RuntimeWarning>,
}

impl RuntimeDiagnostics {
    fn push_warning(&mut self, warning: RuntimeWarning) {
        self.warnings.push(warning);
    }
}

#[derive(Debug)]
pub struct JsRuntime {
    context: BoaContext<'static>,
    module_loader: Rc<AppModuleLoader>,
    fetch_transport: FetchTransport,
    storage_transport: StorageTransport,
    timer_transport: TimerTransport,
    pending_fetches: HashMap<u64, String>,
    pending_storage_requests: HashMap<u64, String>,
    pending_timer_requests: HashMap<u64, String>,
    diagnostics: RuntimeDiagnostics,
}

impl JsRuntime {
    pub fn new() -> Result<Self> {
        let storage_path = std::env::temp_dir().join("rustyjs-ui-storage.json");
        Self::new_with_storage_path(storage_path)
    }

    pub fn new_with_storage_path(storage_path: impl Into<PathBuf>) -> Result<Self> {
        let module_loader = Rc::new(AppModuleLoader::new());
        let loader: Rc<dyn ModuleLoader> = module_loader.clone();
        let mut context = BoaContext::builder()
            .module_loader(loader)
            .build()
            .map_err(|err| anyhow!("failed to build JS runtime context: {err}"))?;
        let fetch_transport = FetchTransport::new()?;
        let storage_transport = StorageTransport::new_with_path(storage_path.into())?;
        let timer_transport = TimerTransport::new()?;
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
            storage_transport,
            timer_transport,
            pending_fetches: HashMap::new(),
            pending_storage_requests: HashMap::new(),
            pending_timer_requests: HashMap::new(),
            diagnostics: RuntimeDiagnostics::default(),
        })
    }

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

    pub fn startup_with_app_source(app_source: &str) -> Result<(Self, Vec<BridgePayload>)> {
        let mut runtime = Self::new()?;
        let bootstrap = jsengine::bootstrap();
        let mut initial_payloads =
            runtime.eval_script_with_path(bootstrap.source, Some(bootstrap.path))?;
        initial_payloads.extend(runtime.eval_script(app_source)?);
        Ok((runtime, initial_payloads))
    }

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

    pub fn eval_script(&mut self, source: &str) -> Result<Vec<BridgePayload>> {
        self.eval_script_with_path(source, None)
    }

    pub fn eval_module_entry(&mut self, entry_path: &Path) -> Result<Vec<BridgePayload>> {
        let (canonical_entry, module) = self
            .module_loader
            .prepare_entry_module(entry_path, &mut self.context)?;
        self.eval_module(canonical_entry.as_path(), &module)
    }

    pub fn diagnostics(&self) -> &RuntimeDiagnostics {
        &self.diagnostics
    }

    pub fn reset_diagnostics(&mut self) {
        self.diagnostics = RuntimeDiagnostics::default();
    }

    fn eval_script_with_path(
        &mut self,
        source: &str,
        source_path: Option<&str>,
    ) -> Result<Vec<BridgePayload>> {
        let started_at = Instant::now();
        self.context
            .eval(Source::from_reader(
                source.as_bytes(),
                source_path.map(Path::new),
            ))
            .map_err(|err| anyhow!("failed to evaluate JS source: {err}"))?;
        self.context.run_jobs();
        let payloads = self.drain_payloads();
        let elapsed = started_at.elapsed();
        self.diagnostics.eval_script_calls += 1;
        self.diagnostics.eval_script_time += elapsed;
        perf::record_eval_script(elapsed);
        payloads
    }

    fn eval_module(&mut self, entry_path: &Path, module: &Module) -> Result<Vec<BridgePayload>> {
        let started_at = Instant::now();
        let promise = module
            .load_link_evaluate(&mut self.context)
            .map_err(|err| anyhow!("failed to load JS module `{}`: {err}", entry_path.display()))?;
        self.context.run_jobs();

        let result = match promise.state().map_err(|err| {
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
        };

        let elapsed = started_at.elapsed();
        self.diagnostics.eval_module_calls += 1;
        self.diagnostics.eval_module_time += elapsed;
        perf::record_eval_module(elapsed);
        result
    }

    pub fn drain_payloads(&mut self) -> Result<Vec<BridgePayload>> {
        let started_at = Instant::now();
        let mut payloads = Vec::new();

        loop {
            let pending = take_outbound_messages();
            if pending.is_empty() {
                break;
            }

            for message in pending {
                self.diagnostics.bridge_messages_seen += 1;
                let parse_started_at = Instant::now();
                let outbound = parse_outbound_message(&message)?;
                self.diagnostics.bridge_parse_time += parse_started_at.elapsed();

                match outbound {
                    OutboundMessage::Payload(payload) => payloads.push(payload),
                    OutboundMessage::FetchRequest(request) => self.submit_fetch_request(request)?,
                    OutboundMessage::StorageRequest(request) => {
                        self.submit_storage_request(request)?
                    }
                    OutboundMessage::TimerRequest(request) => self.submit_timer_request(request)?,
                    OutboundMessage::DevWarning(warning) => {
                        let warning = RuntimeWarning {
                            message: warning.message,
                            details: warning.details,
                        };
                        eprintln!("RustyJS-UI warning: {}", warning.message);
                        self.diagnostics.push_warning(warning);
                    }
                }
            }
        }

        let original_len = payloads.len();
        let payloads = coalesce_payloads(payloads);
        if original_len > payloads.len() {
            self.diagnostics.coalesced_update_batches += 1;
        }

        let elapsed = started_at.elapsed();
        self.diagnostics.drain_payload_calls += 1;
        self.diagnostics.drain_payload_time += elapsed;
        perf::record_drain_payloads(elapsed);
        Ok(payloads)
    }

    pub fn trigger_callback(
        &mut self,
        callback_id: &str,
        payload: Value,
    ) -> Result<Vec<BridgePayload>> {
        let started_at = Instant::now();
        let callback_id =
            serde_json::to_string(callback_id).context("failed to encode callback id")?;
        let payload =
            serde_json::to_string(&payload).context("failed to encode callback payload")?;
        let script = format!("globalThis.RustBridge.trigger({callback_id}, {payload});");
        self.context
            .eval(Source::from_bytes(script.as_str()))
            .map_err(|err| anyhow!("failed to trigger JS callback: {err}"))?;
        self.context.run_jobs();
        let payloads = self.drain_payloads();
        let elapsed = started_at.elapsed();
        self.diagnostics.callback_calls += 1;
        self.diagnostics.callback_time += elapsed;
        perf::record_trigger_callback(elapsed);
        payloads
    }

    pub fn poll_async(&mut self) -> Result<Vec<BridgePayload>> {
        let started_at = Instant::now();

        if !self.has_pending_async_work() {
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

        for completion in self.storage_transport.drain_completions() {
            let Some(request_id) = self.pending_storage_requests.remove(&completion.request_id())
            else {
                continue;
            };

            match completion {
                StorageCompletion::Response(response) => {
                    self.resolve_storage(&request_id, response.value)?
                }
                StorageCompletion::Error(error) => {
                    self.reject_storage(&request_id, &error.message)?
                }
            }

            payloads.extend(self.drain_payloads()?);
        }

        for completion in self.timer_transport.drain_completions() {
            let Some(request_id) = self.pending_timer_requests.remove(&completion.request_id())
            else {
                continue;
            };

            match completion {
                TimerCompletion::Response(_) => self.resolve_timer(&request_id)?,
                TimerCompletion::Error(error) => self.reject_timer(&request_id, &error.message)?,
            }

            payloads.extend(self.drain_payloads()?);
        }

        let elapsed = started_at.elapsed();
        self.diagnostics.poll_async_calls += 1;
        self.diagnostics.poll_async_time += elapsed;
        perf::record_poll_async(elapsed);
        Ok(payloads)
    }

    pub fn has_pending_fetches(&self) -> bool {
        self.has_pending_async_work()
    }

    pub fn has_pending_async_work(&self) -> bool {
        !self.pending_fetches.is_empty()
            || !self.pending_storage_requests.is_empty()
            || !self.pending_timer_requests.is_empty()
    }

    pub fn bootstrap_source() -> &'static str {
        jsengine::bootstrap().source
    }

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
                Err(error) => self.reject_fetch(&request_id, &error.to_string())?,
            },
            Err(error) => self.reject_fetch(&request_id, &error.to_string())?,
        }

        Ok(())
    }

    fn submit_storage_request(&mut self, request: OutboundStorageRequest) -> Result<()> {
        let request_id = request.request_id.clone();
        let transport_request = request
            .to_transport_request()
            .with_context(|| format!("failed to prepare storage request: {request_id}"));

        match transport_request {
            Ok(transport_request) => match self.storage_transport.submit(transport_request) {
                Ok(internal_id) => {
                    self.pending_storage_requests.insert(internal_id, request_id);
                }
                Err(error) => self.reject_storage(&request_id, &error.to_string())?,
            },
            Err(error) => self.reject_storage(&request_id, &error.to_string())?,
        }

        Ok(())
    }

    fn submit_timer_request(&mut self, request: OutboundTimerRequest) -> Result<()> {
        let request_id = request.request_id.clone();
        match self
            .timer_transport
            .submit(Duration::from_millis(request.delay_ms))
        {
            Ok(internal_id) => {
                self.pending_timer_requests.insert(internal_id, request_id);
            }
            Err(error) => self.reject_timer(&request_id, &error.to_string())?,
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

    fn resolve_storage(&mut self, request_id: &str, value: Option<String>) -> Result<()> {
        let request_id =
            serde_json::to_string(request_id).context("failed to encode storage request id")?;
        let value = serde_json::to_string(&value).context("failed to encode storage value")?;
        let script = format!("globalThis.RustBridge.resolveStorage({request_id}, {value});");
        self.context
            .eval(Source::from_bytes(script.as_str()))
            .map_err(|err| anyhow!("failed to resolve storage promise: {err}"))?;
        self.context.run_jobs();
        Ok(())
    }

    fn reject_storage(&mut self, request_id: &str, message: &str) -> Result<()> {
        let request_id =
            serde_json::to_string(request_id).context("failed to encode storage request id")?;
        let message = serde_json::to_string(message).context("failed to encode storage error")?;
        let script = format!("globalThis.RustBridge.rejectStorage({request_id}, {message});");
        self.context
            .eval(Source::from_bytes(script.as_str()))
            .map_err(|err| anyhow!("failed to reject storage promise: {err}"))?;
        self.context.run_jobs();
        Ok(())
    }

    fn resolve_timer(&mut self, request_id: &str) -> Result<()> {
        let request_id =
            serde_json::to_string(request_id).context("failed to encode timer request id")?;
        let script = format!("globalThis.RustBridge.resolveTimer({request_id});");
        self.context
            .eval(Source::from_bytes(script.as_str()))
            .map_err(|err| anyhow!("failed to resolve timer promise: {err}"))?;
        self.context.run_jobs();
        Ok(())
    }

    fn reject_timer(&mut self, request_id: &str, message: &str) -> Result<()> {
        let request_id =
            serde_json::to_string(request_id).context("failed to encode timer request id")?;
        let message = serde_json::to_string(message).context("failed to encode timer error")?;
        let script = format!("globalThis.RustBridge.rejectTimer({request_id}, {message});");
        self.context
            .eval(Source::from_bytes(script.as_str()))
            .map_err(|err| anyhow!("failed to reject timer promise: {err}"))?;
        self.context.run_jobs();
        Ok(())
    }
}

#[derive(Debug)]
enum OutboundMessage {
    Payload(BridgePayload),
    FetchRequest(OutboundFetchRequest),
    StorageRequest(OutboundStorageRequest),
    TimerRequest(OutboundTimerRequest),
    DevWarning(OutboundDevWarning),
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

#[derive(Debug, Deserialize)]
struct OutboundStorageRequest {
    #[serde(rename = "requestId")]
    request_id: String,
    operation: String,
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OutboundTimerRequest {
    #[serde(rename = "requestId")]
    request_id: String,
    #[serde(rename = "delayMs")]
    delay_ms: u64,
}

#[derive(Debug, Deserialize)]
struct OutboundDevWarning {
    message: String,
    #[serde(default)]
    details: Option<String>,
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

impl OutboundStorageRequest {
    fn to_transport_request(&self) -> Result<StorageRequest> {
        match self.operation.trim().to_ascii_uppercase().as_str() {
            "GET" => Ok(StorageRequest::get(
                self.key
                    .clone()
                    .ok_or_else(|| anyhow!("storage get requests require a key"))?,
            )),
            "SET" => Ok(StorageRequest::set(
                self.key
                    .clone()
                    .ok_or_else(|| anyhow!("storage set requests require a key"))?,
                self.value.clone().unwrap_or_default(),
            )),
            "REMOVE" => Ok(StorageRequest::remove(
                self.key
                    .clone()
                    .ok_or_else(|| anyhow!("storage remove requests require a key"))?,
            )),
            "CLEAR" => Ok(StorageRequest::clear()),
            other => Err(anyhow!("unsupported storage operation: {other}")),
        }
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

    match action {
        "FETCH_REQUEST" => serde_json::from_value(value)
            .map(OutboundMessage::FetchRequest)
            .with_context(|| format!("failed to parse fetch request payload: {input}")),
        "STORAGE_REQUEST" => serde_json::from_value(value)
            .map(OutboundMessage::StorageRequest)
            .with_context(|| format!("failed to parse storage request payload: {input}")),
        "TIMER_REQUEST" => serde_json::from_value(value)
            .map(OutboundMessage::TimerRequest)
            .with_context(|| format!("failed to parse timer request payload: {input}")),
        "DEV_WARNING" => serde_json::from_value(value)
            .map(OutboundMessage::DevWarning)
            .with_context(|| format!("failed to parse dev warning payload: {input}")),
        _ => BridgePayload::parse_json(value)
            .map(OutboundMessage::Payload)
            .with_context(|| format!("failed to parse bridge payload: {input}")),
    }
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
