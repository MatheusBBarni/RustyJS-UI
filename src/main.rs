use anyhow::Result;
use iced::executor;
use iced::widget::text;
use iced::{Application, Command, Element, Settings, Theme};
use rustyjs_ui::bridge::{BridgePayload, EventPayload, WindowConfig};
use rustyjs_ui::runtime::JsRuntime;
use rustyjs_ui::ui;
use rustyjs_ui::vdom::UiNode;

fn main() -> iced::Result {
    let flags = bootstrap_flags().map_err(|error| {
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

fn bootstrap_flags() -> Result<AppFlags> {
    let (runtime, payloads) = JsRuntime::startup()?;
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
}

#[derive(Debug, Clone)]
enum Message {
    UiEvent(EventPayload),
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
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        self.window.title.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::UiEvent(event) => match self
                .runtime
                .trigger_callback(&event.callback_id, event.data)
            {
                Ok(payloads) => {
                    self.error = None;

                    for payload in payloads {
                        if let Err(error) = apply_payload(&mut self.window, &mut self.tree, payload)
                        {
                            self.error = Some(error.to_string());
                            break;
                        }
                    }
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                }
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if let Some(tree) = &self.tree {
            return ui::render_root(tree, Message::UiEvent);
        }

        let message = self.error.as_deref().unwrap_or("Booting RustyJS-UI...");

        text(message).size(20).into()
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
            *tree = Some(UiNode::try_from(wire_tree)?);
        }
    }

    Ok(())
}
