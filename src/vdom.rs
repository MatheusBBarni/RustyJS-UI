use crate::style::Style;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallbackRef {
    pub id: String,
}

impl CallbackRef {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Serialize for CallbackRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id)
    }
}

impl<'de> Deserialize<'de> for CallbackRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum CallbackRefRepr {
            Id(String),
            Object { id: String },
        }

        match CallbackRefRepr::deserialize(deserializer)? {
            CallbackRefRepr::Id(id) | CallbackRefRepr::Object { id } => Ok(Self { id }),
        }
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
    pub options: Vec<SelectOption>,
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
    FlatList(FlatListNode),
    Text(TextNode),
    Button(ButtonNode),
    TextInput(TextInputNode),
    SelectInput(SelectInputNode),
}

impl UiNode {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::View(_) => "View",
            Self::FlatList(_) => "FlatList",
            Self::Text(_) => "Text",
            Self::Button(_) => "Button",
            Self::TextInput(_) => "TextInput",
            Self::SelectInput(_) => "SelectInput",
        }
    }

    pub fn style(&self) -> &Style {
        match self {
            Self::View(node) => &node.style,
            Self::FlatList(node) => &node.style,
            Self::Text(node) => &node.style,
            Self::Button(node) => &node.style,
            Self::TextInput(node) => &node.style,
            Self::SelectInput(node) => &node.style,
        }
    }

    pub fn children(&self) -> &[UiNode] {
        match self {
            Self::View(node) => &node.children,
            Self::FlatList(node) => &node.children,
            Self::Text(_) => &[],
            Self::Button(_) => &[],
            Self::TextInput(_) => &[],
            Self::SelectInput(_) => &[],
        }
    }

    pub fn into_children(self) -> Vec<UiNode> {
        match self {
            Self::View(node) => node.children,
            Self::FlatList(node) => node.children,
            Self::Text(_) | Self::Button(_) | Self::TextInput(_) | Self::SelectInput(_) => {
                Vec::new()
            }
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
            "FlatList" => Ok(Self::FlatList(FlatListNode {
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
            "SelectInput" => Ok(Self::SelectInput(SelectInputNode {
                value: node.props.value.unwrap_or_default(),
                placeholder: node.props.placeholder,
                options: node.props.options,
                on_change: node.props.on_change,
                style: node.props.style,
                disabled: node.props.disabled,
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
pub struct FlatListNode {
    pub style: Style,
    pub children: Vec<UiNode>,
}

impl FlatListNode {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
}

impl SelectOption {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

impl fmt::Display for SelectOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

impl<'de> Deserialize<'de> for SelectOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum SelectOptionRepr {
            Value(String),
            Object {
                #[serde(default)]
                label: Option<String>,
                #[serde(default)]
                value: Option<String>,
            },
        }

        match SelectOptionRepr::deserialize(deserializer)? {
            SelectOptionRepr::Value(value) => Ok(Self::new(value.clone(), value)),
            SelectOptionRepr::Object { label, value } => match (label, value) {
                (Some(label), Some(value)) => Ok(Self::new(label, value)),
                (Some(label), None) => Ok(Self::new(label.clone(), label)),
                (None, Some(value)) => Ok(Self::new(value.clone(), value)),
                (None, None) => Err(serde::de::Error::custom(
                    "select option must provide a label or a value",
                )),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectInputNode {
    pub value: String,
    pub placeholder: Option<String>,
    #[serde(default)]
    pub options: Vec<SelectOption>,
    pub on_change: Option<CallbackRef>,
    pub style: Style,
    pub disabled: bool,
}

impl SelectInputNode {
    pub fn new(value: impl Into<String>, options: Vec<SelectOption>, style: Style) -> Self {
        Self {
            value: value.into(),
            placeholder: None,
            options,
            on_change: None,
            style,
            disabled: false,
        }
    }

    pub fn selected_option(&self) -> Option<&SelectOption> {
        self.options
            .iter()
            .find(|option| option.value == self.value)
    }
}

impl From<ViewNode> for UiNode {
    fn from(node: ViewNode) -> Self {
        Self::View(node)
    }
}

impl From<FlatListNode> for UiNode {
    fn from(node: FlatListNode) -> Self {
        Self::FlatList(node)
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

impl From<SelectInputNode> for UiNode {
    fn from(node: SelectInputNode) -> Self {
        Self::SelectInput(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn callback_ref_deserializes_from_string_id() {
        let callback: CallbackRef = serde_json::from_value(json!("cb_1")).unwrap();

        assert_eq!(callback, CallbackRef::new("cb_1"));
    }

    #[test]
    fn callback_ref_deserializes_from_object_form() {
        let callback: CallbackRef = serde_json::from_value(json!({ "id": "cb_2" })).unwrap();

        assert_eq!(callback, CallbackRef::new("cb_2"));
    }

    #[test]
    fn callback_ref_serializes_as_string_id() {
        let callback = CallbackRef::new("cb_3");

        assert_eq!(serde_json::to_value(callback).unwrap(), json!("cb_3"));
    }

    #[test]
    fn prd_style_button_node_deserializes_into_typed_ui_node() {
        let value = json!({
            "type": "Button",
            "props": {
                "text": "Increment",
                "onClick": "cb_1",
                "style": {
                    "padding": 10,
                    "backgroundColor": "#007AFF"
                }
            }
        });

        let node = UiNode::try_from(value).unwrap();

        match node {
            UiNode::Button(button) => {
                assert_eq!(button.text, "Increment");
                assert_eq!(button.on_click, Some(CallbackRef::new("cb_1")));
                assert!(!button.disabled);
            }
            other => panic!("expected Button node, got {other:?}"),
        }
    }

    #[test]
    fn select_input_node_deserializes_into_typed_ui_node() {
        let value = json!({
            "type": "SelectInput",
            "props": {
                "value": "typescript",
                "placeholder": "Choose a language",
                "options": [
                    "rust",
                    { "label": "TypeScript", "value": "typescript" }
                ],
                "onChange": "cb_7",
                "style": {
                    "width": 240,
                    "borderWidth": 1,
                    "borderRadius": 8
                }
            }
        });

        let node = UiNode::try_from(value).unwrap();

        match node {
            UiNode::SelectInput(select) => {
                assert_eq!(select.value, "typescript");
                assert_eq!(select.placeholder.as_deref(), Some("Choose a language"));
                assert_eq!(select.options.len(), 2);
                assert_eq!(select.options[0], SelectOption::new("rust", "rust"));
                assert_eq!(
                    select.options[1],
                    SelectOption::new("TypeScript", "typescript")
                );
                assert_eq!(select.on_change, Some(CallbackRef::new("cb_7")));
                assert!(!select.disabled);
                assert_eq!(
                    select.selected_option(),
                    Some(&SelectOption::new("TypeScript", "typescript"))
                );
            }
            other => panic!("expected SelectInput node, got {other:?}"),
        }
    }
}
