use crate::vdom::{UiNode, WireNode};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default)]
    pub title: String,
    #[serde(default = "WindowConfig::default_width")]
    pub width: u32,
    #[serde(default = "WindowConfig::default_height")]
    pub height: u32,
}

impl WindowConfig {
    pub const fn default_width() -> u32 {
        800
    }

    pub const fn default_height() -> u32 {
        600
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "IcedJS App".to_string(),
            width: Self::default_width(),
            height: Self::default_height(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgePayload {
    InitWindow {
        #[serde(default)]
        title: String,
        #[serde(default = "WindowConfig::default_width")]
        width: u32,
        #[serde(default = "WindowConfig::default_height")]
        height: u32,
    },
    UpdateVdom {
        tree: WireNode,
    },
}

impl BridgePayload {
    pub fn parse_json(value: Value) -> Result<Self> {
        serde_json::from_value(value).map_err(Into::into)
    }

    pub fn parse_str(input: &str) -> Result<Self> {
        serde_json::from_str(input).map_err(Into::into)
    }

    pub fn to_window_config(&self) -> Option<WindowConfig> {
        match self {
            Self::InitWindow {
                title,
                width,
                height,
            } => Some(WindowConfig {
                title: if title.is_empty() {
                    WindowConfig::default().title
                } else {
                    title.clone()
                },
                width: *width,
                height: *height,
            }),
            Self::UpdateVdom { .. } => None,
        }
    }

    pub fn typed_tree(&self) -> Result<Option<UiNode>> {
        match self {
            Self::UpdateVdom { tree } => Ok(Some(tree.clone().try_into()?)),
            Self::InitWindow { .. } => Ok(None),
        }
    }
}

pub fn coalesce_payloads(payloads: Vec<BridgePayload>) -> Vec<BridgePayload> {
    let mut init_window = None;
    let mut last_tree = None;

    for payload in payloads {
        match payload {
            payload @ BridgePayload::InitWindow { .. } => init_window = Some(payload),
            payload @ BridgePayload::UpdateVdom { .. } => last_tree = Some(payload),
        }
    }

    let mut coalesced =
        Vec::with_capacity(init_window.is_some() as usize + last_tree.is_some() as usize);
    if let Some(payload) = init_window {
        coalesced.push(payload);
    }
    if let Some(payload) = last_tree {
        coalesced.push(payload);
    }

    coalesced
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventPayload {
    pub callback_id: String,
    #[serde(default)]
    pub data: Value,
}

impl EventPayload {
    pub fn new(callback_id: impl Into<String>, data: Value) -> Self {
        Self {
            callback_id: callback_id.into(),
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vdom::WireProps;

    #[test]
    fn coalesce_payloads_keeps_last_window_and_tree_payload() {
        let payloads = vec![
            BridgePayload::InitWindow {
                title: "One".to_string(),
                width: 320,
                height: 200,
            },
            BridgePayload::UpdateVdom {
                tree: WireNode::new("Text", WireProps::text("first"), Vec::new()),
            },
            BridgePayload::InitWindow {
                title: "Two".to_string(),
                width: 640,
                height: 480,
            },
            BridgePayload::UpdateVdom {
                tree: WireNode::new("Text", WireProps::text("second"), Vec::new()),
            },
        ];

        let coalesced = coalesce_payloads(payloads);

        assert_eq!(
            coalesced,
            vec![
                BridgePayload::InitWindow {
                    title: "Two".to_string(),
                    width: 640,
                    height: 480,
                },
                BridgePayload::UpdateVdom {
                    tree: WireNode::new("Text", WireProps::text("second"), Vec::new()),
                }
            ]
        );
    }
}
