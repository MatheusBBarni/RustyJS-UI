use crate::style::Style;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallbackRef {
    pub id: String,
}

impl CallbackRef {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WireProps {
    #[serde(default)]
    pub style: Style,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default, alias = "onClick")]
    pub on_click: Option<CallbackRef>,
    #[serde(default, alias = "onChange")]
    pub on_change: Option<CallbackRef>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub multiline: bool,
}

impl WireProps {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Self::default()
        }
    }

    pub fn callback_id(&self) -> Option<&str> {
        self.on_click
            .as_ref()
            .or(self.on_change.as_ref())
            .map(|callback| callback.id.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireNode {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub props: WireProps,
    #[serde(default)]
    pub children: Vec<WireNode>,
}

impl WireNode {
    pub fn new(kind: impl Into<String>, props: WireProps, children: Vec<WireNode>) -> Self {
        Self {
            kind: kind.into(),
            props,
            children,
        }
    }

    pub fn parse(value: Value) -> Result<Self> {
        serde_json::from_value(value).map_err(Into::into)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiNode {
    View(ViewNode),
    Text(TextNode),
    Button(ButtonNode),
    TextInput(TextInputNode),
}

impl UiNode {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::View(_) => "View",
            Self::Text(_) => "Text",
            Self::Button(_) => "Button",
            Self::TextInput(_) => "TextInput",
        }
    }

    pub fn style(&self) -> &Style {
        match self {
            Self::View(node) => &node.style,
            Self::Text(node) => &node.style,
            Self::Button(node) => &node.style,
            Self::TextInput(node) => &node.style,
        }
    }

    pub fn children(&self) -> &[UiNode] {
        match self {
            Self::View(node) => &node.children,
            Self::Text(_) => &[],
            Self::Button(_) => &[],
            Self::TextInput(_) => &[],
        }
    }

    pub fn into_children(self) -> Vec<UiNode> {
        match self {
            Self::View(node) => node.children,
            Self::Text(_) | Self::Button(_) | Self::TextInput(_) => Vec::new(),
        }
    }
}

impl TryFrom<WireNode> for UiNode {
    type Error = anyhow::Error;

    fn try_from(node: WireNode) -> Result<Self> {
        match node.kind.as_str() {
            "View" => Ok(Self::View(ViewNode {
                style: node.props.style,
                children: node
                    .children
                    .into_iter()
                    .map(UiNode::try_from)
                    .collect::<Result<Vec<_>>>()?,
            })),
            "Text" => Ok(Self::Text(TextNode {
                text: node.props.text.or(node.props.value).unwrap_or_default(),
                style: node.props.style,
            })),
            "Button" => Ok(Self::Button(ButtonNode {
                text: node.props.text.or(node.props.value).unwrap_or_default(),
                on_click: node.props.on_click,
                style: node.props.style,
                disabled: node.props.disabled,
            })),
            "TextInput" => Ok(Self::TextInput(TextInputNode {
                value: node.props.value.unwrap_or_default(),
                placeholder: node.props.placeholder,
                on_change: node.props.on_change,
                style: node.props.style,
                disabled: node.props.disabled,
                multiline: node.props.multiline,
            })),
            other => Err(anyhow!("unsupported node type: {other}")),
        }
    }
}

impl TryFrom<Value> for UiNode {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self> {
        WireNode::parse(value)?.try_into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewNode {
    pub style: Style,
    pub children: Vec<UiNode>,
}

impl ViewNode {
    pub fn new(style: Style, children: Vec<UiNode>) -> Self {
        Self { style, children }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextNode {
    pub text: String,
    pub style: Style,
}

impl TextNode {
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonNode {
    pub text: String,
    pub on_click: Option<CallbackRef>,
    pub style: Style,
    pub disabled: bool,
}

impl ButtonNode {
    pub fn new(text: impl Into<String>, on_click: Option<CallbackRef>, style: Style) -> Self {
        Self {
            text: text.into(),
            on_click,
            style,
            disabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextInputNode {
    pub value: String,
    pub placeholder: Option<String>,
    pub on_change: Option<CallbackRef>,
    pub style: Style,
    pub disabled: bool,
    pub multiline: bool,
}

impl TextInputNode {
    pub fn new(value: impl Into<String>, style: Style) -> Self {
        Self {
            value: value.into(),
            placeholder: None,
            on_change: None,
            style,
            disabled: false,
            multiline: false,
        }
    }
}

impl From<ViewNode> for UiNode {
    fn from(node: ViewNode) -> Self {
        Self::View(node)
    }
}

impl From<TextNode> for UiNode {
    fn from(node: TextNode) -> Self {
        Self::Text(node)
    }
}

impl From<ButtonNode> for UiNode {
    fn from(node: ButtonNode) -> Self {
        Self::Button(node)
    }
}

impl From<TextInputNode> for UiNode {
    fn from(node: TextInputNode) -> Self {
        Self::TextInput(node)
    }
}
