use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlexDirection {
    Row,
    Column,
}

impl Default for FlexDirection {
    fn default() -> Self {
        Self::Column
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

impl Default for AlignItems {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizeValue {
    Auto,
    Fill,
    Shrink,
    Px(u32),
}

impl Default for SizeValue {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontWeight {
    Thin,
    Light,
    Normal,
    Medium,
    Semibold,
    Bold,
    Heavy,
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color {
    pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub const fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    pub fn from_hex(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);

        let parse_byte = |pair: &str| u8::from_str_radix(pair, 16).ok();

        match hex.len() {
            3 => {
                let r = parse_byte(&hex[0..1].repeat(2))?;
                let g = parse_byte(&hex[1..2].repeat(2))?;
                let b = parse_byte(&hex[2..3].repeat(2))?;
                Some(Self::from_rgb8(r, g, b))
            }
            6 => {
                let r = parse_byte(&hex[0..2])?;
                let g = parse_byte(&hex[2..4])?;
                let b = parse_byte(&hex[4..6])?;
                Some(Self::from_rgb8(r, g, b))
            }
            8 => {
                let r = parse_byte(&hex[0..2])?;
                let g = parse_byte(&hex[2..4])?;
                let b = parse_byte(&hex[4..6])?;
                let a = parse_byte(&hex[6..8])?;
                Some(Self::from_rgba8(r, g, b, a))
            }
            _ => None,
        }
    }

    pub const fn from_rgb8(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
            1.0,
        )
    }

    pub const fn from_rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self::rgba(
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
            alpha as f32 / 255.0,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutStyle {
    #[serde(default, alias = "direction")]
    pub flex_direction: FlexDirection,
    #[serde(default, deserialize_with = "deserialize_edge_insets")]
    pub padding: EdgeInsets,
    #[serde(default)]
    pub spacing: f32,
    #[serde(default, deserialize_with = "deserialize_size_value")]
    pub width: SizeValue,
    #[serde(default, deserialize_with = "deserialize_size_value")]
    pub height: SizeValue,
    #[serde(default, alias = "alignItems")]
    pub align_items: AlignItems,
    #[serde(default, alias = "justifyContent")]
    pub justify_content: JustifyContent,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            flex_direction: FlexDirection::default(),
            padding: EdgeInsets::default(),
            spacing: 0.0,
            width: SizeValue::Auto,
            height: SizeValue::Auto,
            align_items: AlignItems::default(),
            justify_content: JustifyContent::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceStyle {
    #[serde(
        default,
        alias = "backgroundColor",
        deserialize_with = "deserialize_option_color"
    )]
    pub background_color: Option<Color>,
    #[serde(
        default,
        alias = "borderColor",
        deserialize_with = "deserialize_option_color"
    )]
    pub border_color: Option<Color>,
    #[serde(default, alias = "borderWidth")]
    pub border_width: f32,
    #[serde(default, alias = "borderRadius")]
    pub border_radius: f32,
}

impl Default for AppearanceStyle {
    fn default() -> Self {
        Self {
            background_color: None,
            border_color: None,
            border_width: 0.0,
            border_radius: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    #[serde(
        default,
        alias = "color",
        deserialize_with = "deserialize_option_color"
    )]
    pub color: Option<Color>,
    #[serde(default = "TextStyle::default_font_size", alias = "fontSize")]
    pub font_size: f32,
    #[serde(default, alias = "fontWeight")]
    pub font_weight: FontWeight,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: None,
            font_size: Self::default_font_size(),
            font_weight: FontWeight::default(),
        }
    }
}

impl TextStyle {
    pub const fn default_font_size() -> f32 {
        16.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Style {
    #[serde(default, flatten)]
    pub layout: LayoutStyle,
    #[serde(default, flatten)]
    pub appearance: AppearanceStyle,
    #[serde(default, flatten)]
    pub text: TextStyle,
}

impl Style {
    pub fn parse(value: &Value) -> serde_json::Result<Self> {
        serde_json::from_value(value.clone())
    }

    pub fn merge(&self, override_style: &Self) -> Self {
        let mut merged = self.clone();

        if override_style.layout.flex_direction != FlexDirection::default() {
            merged.layout.flex_direction = override_style.layout.flex_direction;
        }
        if override_style.layout.padding != EdgeInsets::ZERO {
            merged.layout.padding = override_style.layout.padding;
        }
        if override_style.layout.spacing != 0.0 {
            merged.layout.spacing = override_style.layout.spacing;
        }
        if override_style.layout.width != SizeValue::Auto {
            merged.layout.width = override_style.layout.width;
        }
        if override_style.layout.height != SizeValue::Auto {
            merged.layout.height = override_style.layout.height;
        }
        if override_style.layout.align_items != AlignItems::default() {
            merged.layout.align_items = override_style.layout.align_items;
        }
        if override_style.layout.justify_content != JustifyContent::default() {
            merged.layout.justify_content = override_style.layout.justify_content;
        }

        if override_style.appearance.background_color.is_some() {
            merged.appearance.background_color = override_style.appearance.background_color;
        }
        if override_style.appearance.border_color.is_some() {
            merged.appearance.border_color = override_style.appearance.border_color;
        }
        if override_style.appearance.border_width != 0.0 {
            merged.appearance.border_width = override_style.appearance.border_width;
        }
        if override_style.appearance.border_radius != 0.0 {
            merged.appearance.border_radius = override_style.appearance.border_radius;
        }

        if override_style.text.color.is_some() {
            merged.text.color = override_style.text.color;
        }
        if override_style.text.font_size != TextStyle::default().font_size {
            merged.text.font_size = override_style.text.font_size;
        }
        if override_style.text.font_weight != FontWeight::default() {
            merged.text.font_weight = override_style.text.font_weight;
        }

        merged
    }
}

fn deserialize_edge_insets<'de, D>(deserializer: D) -> Result<EdgeInsets, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    parse_edge_insets(value).map_err(de::Error::custom)
}

fn deserialize_size_value<'de, D>(deserializer: D) -> Result<SizeValue, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    parse_size_value(value).map_err(de::Error::custom)
}

fn deserialize_option_color<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(raw) => parse_color(raw).map(Some).map_err(de::Error::custom),
    }
}

pub fn parse_edge_insets(value: Value) -> Result<EdgeInsets, String> {
    match value {
        Value::Null => Ok(EdgeInsets::default()),
        Value::Number(number) => {
            let amount = number
                .as_f64()
                .ok_or_else(|| "padding must be a finite number".to_string())?
                as f32;
            Ok(EdgeInsets::all(amount))
        }
        Value::Object(mut map) => {
            if let Some(all) = map.remove("all") {
                let amount = parse_number(all, "padding.all")?;
                return Ok(EdgeInsets::all(amount));
            }

            let horizontal = map
                .remove("x")
                .map(|v| parse_number(v, "padding.x"))
                .transpose()?
                .unwrap_or(0.0);
            let vertical = map
                .remove("y")
                .map(|v| parse_number(v, "padding.y"))
                .transpose()?
                .unwrap_or(0.0);

            Ok(EdgeInsets {
                top: map
                    .remove("top")
                    .map(|v| parse_number(v, "padding.top"))
                    .transpose()?
                    .unwrap_or(vertical),
                right: map
                    .remove("right")
                    .map(|v| parse_number(v, "padding.right"))
                    .transpose()?
                    .unwrap_or(horizontal),
                bottom: map
                    .remove("bottom")
                    .map(|v| parse_number(v, "padding.bottom"))
                    .transpose()?
                    .unwrap_or(vertical),
                left: map
                    .remove("left")
                    .map(|v| parse_number(v, "padding.left"))
                    .transpose()?
                    .unwrap_or(horizontal),
            })
        }
        other => Err(format!("unsupported padding value: {other}")),
    }
}

pub fn parse_size_value(value: Value) -> Result<SizeValue, String> {
    match value {
        Value::Null => Ok(SizeValue::Auto),
        Value::Number(number) => {
            let size = number
                .as_f64()
                .ok_or_else(|| "size must be finite".to_string())?;
            if size.is_sign_negative() {
                return Err("size must be non-negative".to_string());
            }
            Ok(SizeValue::Px(size.round() as u32))
        }
        Value::String(text) => match text.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(SizeValue::Auto),
            "fill" => Ok(SizeValue::Fill),
            "shrink" => Ok(SizeValue::Shrink),
            other => Err(format!("unsupported size value: {other}")),
        },
        other => Err(format!("unsupported size value: {other}")),
    }
}

pub fn parse_color(value: Value) -> Result<Color, String> {
    match value {
        Value::String(text) => {
            Color::from_hex(&text).ok_or_else(|| format!("invalid color string: {text}"))
        }
        Value::Object(mut map) => {
            let red = map
                .remove("red")
                .or_else(|| map.remove("r"))
                .map(|v| parse_number(v, "color.red"))
                .transpose()?
                .unwrap_or(0.0);
            let green = map
                .remove("green")
                .or_else(|| map.remove("g"))
                .map(|v| parse_number(v, "color.green"))
                .transpose()?
                .unwrap_or(0.0);
            let blue = map
                .remove("blue")
                .or_else(|| map.remove("b"))
                .map(|v| parse_number(v, "color.blue"))
                .transpose()?
                .unwrap_or(0.0);
            let alpha = map
                .remove("alpha")
                .or_else(|| map.remove("a"))
                .map(|v| parse_number(v, "color.alpha"))
                .transpose()?
                .unwrap_or(1.0);

            Ok(Color::rgba(red, green, blue, alpha))
        }
        other => Err(format!("unsupported color value: {other}")),
    }
}

fn parse_number(value: Value, field: &str) -> Result<f32, String> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .map(|v| v as f32)
            .ok_or_else(|| format!("{field} must be finite")),
        Value::String(text) => text
            .parse::<f32>()
            .map_err(|err| format!("{field} must be numeric: {err}")),
        other => Err(format!("{field} must be numeric, got {other}")),
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rgba({:.3}, {:.3}, {:.3}, {:.3})",
            self.red, self.green, self.blue, self.alpha
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn style_defaults_font_size_when_text_style_field_is_missing() {
        let style = Style::parse(&json!({
            "padding": 10,
            "backgroundColor": "#007AFF",
            "borderRadius": 8
        }))
        .unwrap();

        assert_eq!(style.text.font_size, TextStyle::default_font_size());
    }

    #[test]
    fn style_keeps_explicit_font_size() {
        let style = Style::parse(&json!({
            "fontSize": 28,
            "color": "#111111"
        }))
        .unwrap();

        assert_eq!(style.text.font_size, 28.0);
    }
}
