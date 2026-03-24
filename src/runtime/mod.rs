mod scripts;

use crate::bridge::BridgePayload;
use anyhow::{anyhow, Context as AnyhowContext, Result};
use boa_engine::{Context as BoaContext, JsValue, NativeFunction, Source};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::VecDeque;

thread_local! {
    static OUTBOUND_QUEUE: RefCell<VecDeque<String>> = const { RefCell::new(VecDeque::new()) };
}

/// Boa runtime host for the RustyJS-UI bridge.
///
/// The runtime evaluates an embedded bootstrap script that exposes the JS-side
/// helpers (`App`, `View`, `Text`, `Button`, `TextInput`, and
/// `__SEND_TO_RUST__`) plus a sample counter app used for the MVP.
#[derive(Debug)]
pub struct JsRuntime {
    context: BoaContext<'static>,
}

impl JsRuntime {
    /// Creates a new Boa context and installs the host bridge helpers.
    pub fn new() -> Result<Self> {
        let mut context = BoaContext::default();
        context
            .register_global_callable(
                "__RUSTYJS_NATIVE_CAPTURE__",
                1,
                NativeFunction::from_fn_ptr(native_capture),
            )
            .map_err(|err| anyhow!("failed to register native capture callback: {err}"))?;

        Ok(Self { context })
    }

    /// Boots the embedded runtime and loads the bundled counter app.
    ///
    /// Returns the runtime plus the payloads produced during initialization.
    pub fn startup() -> Result<(Self, Vec<BridgePayload>)> {
        let mut runtime = Self::new()?;
        let mut initial_payloads = runtime.eval_script(scripts::bootstrap())?;
        initial_payloads.extend(runtime.eval_script(scripts::counter_app())?);
        Ok((runtime, initial_payloads))
    }

    /// Evaluates additional JS source and returns any payloads emitted by it.
    pub fn eval_script(&mut self, source: &str) -> Result<Vec<BridgePayload>> {
        self.context
            .eval(Source::from_bytes(source))
            .map_err(|err| anyhow!("failed to evaluate JS source: {err}"))?;
        self.context.run_jobs();
        self.drain_payloads()
    }

    /// Drains outbound bridge payloads that the JS runtime queued.
    pub fn drain_payloads(&mut self) -> Result<Vec<BridgePayload>> {
        let pending = OUTBOUND_QUEUE.with(|queue| queue.borrow_mut().drain(..).collect::<Vec<_>>());

        pending
            .into_iter()
            .map(|payload| {
                BridgePayload::parse_str(&payload)
                    .with_context(|| format!("failed to parse bridge payload: {payload}"))
            })
            .collect()
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

    /// Returns the embedded bootstrap script.
    pub fn bootstrap_source() -> &'static str {
        scripts::bootstrap()
    }

    /// Returns the bundled sample counter app.
    pub fn sample_counter_app_source() -> &'static str {
        scripts::counter_app()
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
