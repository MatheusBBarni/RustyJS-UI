use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

#[derive(Debug, Clone, Default, Serialize)]
pub struct DurationMetric {
    pub count: u64,
    pub total_micros: u64,
    pub max_micros: u64,
}

impl DurationMetric {
    pub fn record(&mut self, duration: Duration) {
        let micros = duration.as_micros().min(u64::MAX as u128) as u64;
        self.count = self.count.saturating_add(1);
        self.total_micros = self.total_micros.saturating_add(micros);
        self.max_micros = self.max_micros.max(micros);
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RuntimeMetrics {
    pub eval_script: DurationMetric,
    pub eval_module: DurationMetric,
    pub drain_payloads: DurationMetric,
    pub trigger_callback: DurationMetric,
    pub poll_async: DurationMetric,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BridgeMetrics {
    pub typed_tree_conversion: DurationMetric,
    pub update_vdom_received: u64,
    pub update_vdom_applied: u64,
    pub update_vdom_skipped: u64,
    pub update_vdom_coalesced: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UiMetrics {
    pub render_root: DurationMetric,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PerfSnapshot {
    pub runtime: RuntimeMetrics,
    pub bridge: BridgeMetrics,
    pub ui: UiMetrics,
}

fn metrics() -> &'static Mutex<PerfSnapshot> {
    static METRICS: OnceLock<Mutex<PerfSnapshot>> = OnceLock::new();
    METRICS.get_or_init(|| Mutex::new(PerfSnapshot::default()))
}

fn with_metrics(f: impl FnOnce(&mut PerfSnapshot)) {
    if let Ok(mut metrics) = metrics().lock() {
        f(&mut metrics);
    }
}

pub fn reset_metrics() {
    with_metrics(|metrics| *metrics = PerfSnapshot::default());
}

pub fn snapshot_metrics() -> PerfSnapshot {
    metrics()
        .lock()
        .map(|metrics| metrics.clone())
        .unwrap_or_default()
}

pub fn record_eval_script(duration: Duration) {
    with_metrics(|metrics| metrics.runtime.eval_script.record(duration));
}

pub fn record_eval_module(duration: Duration) {
    with_metrics(|metrics| metrics.runtime.eval_module.record(duration));
}

pub fn record_drain_payloads(duration: Duration) {
    with_metrics(|metrics| metrics.runtime.drain_payloads.record(duration));
}

pub fn record_trigger_callback(duration: Duration) {
    with_metrics(|metrics| metrics.runtime.trigger_callback.record(duration));
}

pub fn record_poll_async(duration: Duration) {
    with_metrics(|metrics| metrics.runtime.poll_async.record(duration));
}

pub fn record_typed_tree_conversion(duration: Duration) {
    with_metrics(|metrics| metrics.bridge.typed_tree_conversion.record(duration));
}

pub fn record_render_root(duration: Duration) {
    with_metrics(|metrics| metrics.ui.render_root.record(duration));
}

pub fn record_update_vdom_received() {
    with_metrics(|metrics| {
        metrics.bridge.update_vdom_received =
            metrics.bridge.update_vdom_received.saturating_add(1);
    });
}

pub fn record_update_vdom_applied() {
    with_metrics(|metrics| {
        metrics.bridge.update_vdom_applied =
            metrics.bridge.update_vdom_applied.saturating_add(1);
    });
}

pub fn record_update_vdom_skipped() {
    with_metrics(|metrics| {
        metrics.bridge.update_vdom_skipped =
            metrics.bridge.update_vdom_skipped.saturating_add(1);
    });
}

pub fn record_update_vdom_coalesced(skipped_intermediate_updates: u64) {
    if skipped_intermediate_updates == 0 {
        return;
    }

    with_metrics(|metrics| {
        metrics.bridge.update_vdom_coalesced = metrics
            .bridge
            .update_vdom_coalesced
            .saturating_add(skipped_intermediate_updates);
    });
}

