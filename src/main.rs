use anyhow::Result;
use iced::executor;
use iced::widget::text;
use iced::{time, Application, Command, Element, Settings, Subscription, Theme};
use rustyjs_ui::bridge::{BridgePayload, EventPayload, WindowConfig};
use rustyjs_ui::perf;
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::ui;
use rustyjs_ui::vdom::UiNode;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn main() -> iced::Result {
    let script_path = application_script_path();
    let flags = bootstrap_flags(script_path.as_deref()).map_err(|error| {
        iced::Error::WindowCreationFailed(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            error.to_string(),
        )))
    })?;
    let mut settings = Settings::with_flags(flags);
    settings.window = iced::window::Settings {
        size: (settings.flags.window.width, settings.flags.window.height),
        ..iced::window::Settings::default()
    };

    RustyJsApp::run(settings)
}

fn application_script_path() -> Option<PathBuf> {
    std::env::args_os().nth(1).map(PathBuf::from)
}

fn bootstrap_flags(script_path: Option<&Path>) -> Result<AppFlags> {
    let (runtime, payloads) = match script_path {
        Some(path) => JsRuntime::startup_with_app_entry(path)?,
        None => JsRuntime::startup()?,
    };
    let mut window = WindowConfig::default();
    let mut tree = None;

    for payload in payloads {
        apply_payload(&mut window, &mut tree, payload)?;
    }

    Ok(AppFlags {
        runtime,
        window,
        tree,
        error: None,
    })
}

struct AppFlags {
    runtime: JsRuntime,
    window: WindowConfig,
    tree: Option<UiNode>,
    error: Option<String>,
}

struct RustyJsApp {
    runtime: JsRuntime,
    window: WindowConfig,
    tree: Option<UiNode>,
    error: Option<String>,
    pending_text_inputs: PendingTextInputs,
}

#[derive(Default)]
struct PendingTextInputs {
    events: Vec<EventPayload>,
}

impl PendingTextInputs {
    fn push(&mut self, event: EventPayload) {
        if let Some(pending) = self
            .events
            .iter_mut()
            .find(|pending| pending.callback_id == event.callback_id)
        {
            pending.data = event.data;
            return;
        }

        self.events.push(event);
    }

    fn take(&mut self) -> Vec<EventPayload> {
        std::mem::take(&mut self.events)
    }

    fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[derive(Debug, Clone)]
enum Message {
    UiEvent(EventPayload),
    TextInputEvent(EventPayload),
    FrameTick,
}

impl Application for RustyJsApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                runtime: flags.runtime,
                window: flags.window,
                tree: flags.tree,
                error: flags.error,
                pending_text_inputs: PendingTextInputs::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        self.window.title.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::UiEvent(event) => {
                self.flush_pending_text_inputs();
                self.process_ui_event(event);
            }
            Message::TextInputEvent(event) => self.queue_text_input(event),
            Message::FrameTick => {
                self.flush_pending_text_inputs();
                self.poll_async();
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.runtime.has_pending_async_work() || !self.pending_text_inputs.is_empty() {
            return time::every(Duration::from_millis(16)).map(|_| Message::FrameTick);
        }

        Subscription::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if let Some(tree) = &self.tree {
            let started_at = Instant::now();
            let element = ui::render_root(tree, Message::UiEvent, Message::TextInputEvent);
            perf::record_render_root(started_at.elapsed());
            return element;
        }

        let message = self.error.as_deref().unwrap_or("Booting RustyJS-UI...");

        text(message).size(20).into()
    }
}

impl RustyJsApp {
    fn queue_text_input(&mut self, event: EventPayload) {
        self.pending_text_inputs.push(event);
    }

    fn flush_pending_text_inputs(&mut self) {
        let pending = self.pending_text_inputs.take();

        for event in pending {
            self.process_ui_event(event);
        }
    }

    fn process_ui_event(&mut self, event: EventPayload) {
        match self
            .runtime
            .trigger_callback(&event.callback_id, event.data)
        {
            Ok(payloads) => {
                self.error = None;
                if let Err(error) = apply_payloads(&mut self.window, &mut self.tree, payloads) {
                    self.error = Some(error.to_string());
                }
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }

    fn poll_async(&mut self) {
        match self.runtime.poll_async() {
            Ok(payloads) if !payloads.is_empty() => {
                if let Err(error) = apply_payloads(&mut self.window, &mut self.tree, payloads) {
                    self.error = Some(error.to_string());
                } else {
                    self.error = None;
                }
            }
            Ok(_) => {}
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }
}

fn apply_payload(
    window: &mut WindowConfig,
    tree: &mut Option<UiNode>,
    payload: BridgePayload,
) -> Result<()> {
    match payload {
        BridgePayload::InitWindow {
            title,
            width,
            height,
        } => {
            *window = WindowConfig {
                title: if title.is_empty() {
                    WindowConfig::default().title
                } else {
                    title
                },
                width,
                height,
            };
        }
        BridgePayload::UpdateVdom { tree: wire_tree } => {
            perf::record_update_vdom_received();
            let started_at = Instant::now();
            let next_tree = UiNode::try_from(wire_tree)?;
            perf::record_typed_tree_conversion(started_at.elapsed());

            if tree.as_ref() == Some(&next_tree) {
                perf::record_update_vdom_skipped();
            } else {
                *tree = Some(next_tree);
                perf::record_update_vdom_applied();
            }
        }
    }

    Ok(())
}

fn apply_payloads(
    window: &mut WindowConfig,
    tree: &mut Option<UiNode>,
    payloads: Vec<BridgePayload>,
) -> Result<()> {
    let mut pending_tree = None;
    let mut coalesced_updates = 0_u64;

    for payload in payloads {
        match payload {
            BridgePayload::UpdateVdom { tree: wire_tree } => {
                perf::record_update_vdom_received();
                if pending_tree.replace(wire_tree).is_some() {
                    coalesced_updates = coalesced_updates.saturating_add(1);
                }
            }
            payload => apply_payload(window, tree, payload)?,
        }
    }

    perf::record_update_vdom_coalesced(coalesced_updates);

    if let Some(wire_tree) = pending_tree {
        let started_at = Instant::now();
        let next_tree = UiNode::try_from(wire_tree)?;
        perf::record_typed_tree_conversion(started_at.elapsed());

        if tree.as_ref() == Some(&next_tree) {
            perf::record_update_vdom_skipped();
        } else {
            *tree = Some(next_tree);
            perf::record_update_vdom_applied();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::PendingTextInputs;
    use rustyjs_ui::bridge::EventPayload;
    use serde_json::Value;

    #[test]
    fn pending_text_inputs_coalesce_by_callback_id() {
        let mut pending = PendingTextInputs::default();

        pending.push(EventPayload::new(
            "cb_email",
            Value::String("a".to_string()),
        ));
        pending.push(EventPayload::new(
            "cb_email",
            Value::String("ab".to_string()),
        ));
        pending.push(EventPayload::new(
            "cb_password",
            Value::String("secret".to_string()),
        ));

        let drained = pending.take();

        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].callback_id, "cb_email");
        assert_eq!(drained[0].data, Value::String("ab".to_string()));
        assert_eq!(drained[1].callback_id, "cb_password");
        assert_eq!(drained[1].data, Value::String("secret".to_string()));
        assert!(pending.is_empty());
    }
}
