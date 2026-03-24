use crate::bridge::EventPayload;
use crate::style::{
    AlignItems, AppearanceStyle, Color as StyleColor, EdgeInsets, FlexDirection, JustifyContent,
    SizeValue, Style,
};
use crate::vdom::{ButtonNode, TextInputNode, TextNode, UiNode, ViewNode};
use iced::theme;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Background, Color, Element, Length, Padding, Theme};
use serde_json::Value;

pub fn render_root<'a, Message>(
    node: &'a UiNode,
    on_event: impl Fn(EventPayload) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    render_node(node, on_event)
}

fn render_node<'a, Message>(
    node: &'a UiNode,
    on_event: impl Fn(EventPayload) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    match node {
        UiNode::View(view) => render_view(view, on_event),
        UiNode::Text(text_node) => render_text(text_node),
        UiNode::Button(button_node) => render_button(button_node, on_event),
        UiNode::TextInput(input_node) => render_text_input(input_node, on_event),
    }
}

fn render_view<'a, Message>(
    node: &'a ViewNode,
    on_event: impl Fn(EventPayload) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let children = node
        .children
        .iter()
        .map(|child| render_node(child, on_event))
        .collect::<Vec<_>>();
    let children = apply_justify_content(
        children,
        node.style.layout.flex_direction,
        node.style.layout.justify_content,
    );

    let content = match node.style.layout.flex_direction {
        FlexDirection::Row => row(children)
            .spacing(node.style.layout.spacing)
            .padding(to_padding(node.style.layout.padding))
            .align_items(to_alignment(node.style.layout.align_items))
            .width(to_length(node.style.layout.width))
            .height(to_length(node.style.layout.height))
            .into(),
        FlexDirection::Column => column(children)
            .spacing(node.style.layout.spacing)
            .padding(to_padding(node.style.layout.padding))
            .align_items(to_alignment(node.style.layout.align_items))
            .width(to_length(node.style.layout.width))
            .height(to_length(node.style.layout.height))
            .into(),
    };

    wrap_with_container(content, &node.style, false, false)
}

fn render_text<'a, Message>(node: &'a TextNode) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let mut content = text(node.text.as_str()).size(node.style.text.font_size);

    if let Some(color) = node.style.text.color {
        content = content.style(Color::from(color));
    }

    wrap_with_container(content.into(), &node.style, true, true)
}

fn render_button<'a, Message>(
    node: &'a ButtonNode,
    on_event: impl Fn(EventPayload) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let mut label = text(node.text.as_str()).size(node.style.text.font_size);

    if let Some(color) = node.style.text.color {
        label = label.style(Color::from(color));
    }

    let mut widget = button(label)
        .padding(to_padding(node.style.layout.padding))
        .width(to_length(node.style.layout.width))
        .height(to_length(node.style.layout.height))
        .style(theme::Button::custom(NodeButtonStyle::new(
            node.style.appearance.clone(),
            node.style.text.color,
        )));

    if !node.disabled {
        if let Some(callback) = &node.on_click {
            widget = widget.on_press(on_event(EventPayload::new(
                callback.id.clone(),
                Value::Null,
            )));
        }
    }

    widget.into()
}

fn render_text_input<'a, Message>(
    node: &'a TextInputNode,
    on_event: impl Fn(EventPayload) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let placeholder = node.placeholder.as_deref().unwrap_or_default();
    let mut widget = text_input(placeholder, node.value.as_str())
        .padding(to_padding(node.style.layout.padding))
        .width(to_length(node.style.layout.width))
        .size(node.style.text.font_size)
        .style(theme::TextInput::Custom(Box::new(NodeTextInputStyle::new(
            node.style.appearance.clone(),
            node.style.text.color,
        ))));

    if !node.disabled {
        if let Some(callback) = &node.on_change {
            let callback_id = callback.id.clone();
            widget = widget.on_input(move |value| {
                on_event(EventPayload::new(callback_id.clone(), Value::String(value)))
            });
        }
    }

    wrap_with_container(widget.into(), &node.style, false, false)
}

fn wrap_with_container<'a, Message>(
    content: Element<'a, Message>,
    style: &Style,
    apply_padding: bool,
    apply_size: bool,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let needs_container = apply_padding && style.layout.padding != EdgeInsets::ZERO
        || apply_size
        || has_appearance(style);

    if !needs_container {
        return content;
    }

    let style_snapshot = style.clone();
    let mut wrapper = container(content);

    if apply_padding {
        wrapper = wrapper.padding(to_padding(style.layout.padding));
    }

    if apply_size {
        wrapper = wrapper
            .width(to_length(style.layout.width))
            .height(to_length(style.layout.height));
    }

    if has_appearance(&style_snapshot) {
        wrapper = wrapper.style(move |_theme: &Theme| container::Appearance {
            text_color: style_snapshot.text.color.map(Color::from),
            background: style_snapshot
                .appearance
                .background_color
                .map(|color| Background::Color(Color::from(color))),
            border_radius: style_snapshot.appearance.border_radius.into(),
            border_width: style_snapshot.appearance.border_width,
            border_color: style_snapshot
                .appearance
                .border_color
                .map(Color::from)
                .unwrap_or(Color::TRANSPARENT),
        });
    }

    wrapper.into()
}

fn apply_justify_content<'a, Message>(
    children: Vec<Element<'a, Message>>,
    direction: FlexDirection,
    justify: JustifyContent,
) -> Vec<Element<'a, Message>>
where
    Message: Clone + 'a,
{
    if children.is_empty() {
        return children;
    }

    match justify {
        JustifyContent::Start => children,
        JustifyContent::Center => {
            let mut spaced = Vec::with_capacity(children.len() + 2);
            spaced.push(fill_space(direction, 1));
            spaced.extend(children);
            spaced.push(fill_space(direction, 1));
            spaced
        }
        JustifyContent::End => {
            let mut spaced = Vec::with_capacity(children.len() + 1);
            spaced.push(fill_space(direction, 1));
            spaced.extend(children);
            spaced
        }
        JustifyContent::SpaceBetween => intersperse_spaces(children, direction, false, false, 1),
        JustifyContent::SpaceAround => intersperse_spaces(children, direction, true, true, 1),
        JustifyContent::SpaceEvenly => intersperse_spaces(children, direction, true, true, 2),
    }
}

fn intersperse_spaces<'a, Message>(
    children: Vec<Element<'a, Message>>,
    direction: FlexDirection,
    include_edges: bool,
    trailing_edge: bool,
    portion: u16,
) -> Vec<Element<'a, Message>>
where
    Message: Clone + 'a,
{
    let mut spaced = Vec::with_capacity(children.len() * 2 + 1);
    let mut iter = children.into_iter().peekable();

    if include_edges {
        spaced.push(fill_space(direction, portion));
    }

    while let Some(child) = iter.next() {
        spaced.push(child);

        if iter.peek().is_some() {
            spaced.push(fill_space(direction, portion));
        }
    }

    if trailing_edge {
        spaced.push(fill_space(direction, portion));
    }

    spaced
}

fn fill_space<'a, Message>(direction: FlexDirection, portion: u16) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let fill = Length::FillPortion(portion);

    match direction {
        FlexDirection::Row => Space::with_width(fill).into(),
        FlexDirection::Column => Space::with_height(fill).into(),
    }
}

fn has_appearance(style: &Style) -> bool {
    style.appearance.background_color.is_some()
        || style.appearance.border_color.is_some()
        || style.appearance.border_width > 0.0
        || style.appearance.border_radius > 0.0
        || style.text.color.is_some()
}

fn to_alignment(value: AlignItems) -> Alignment {
    match value {
        AlignItems::Start | AlignItems::Stretch => Alignment::Start,
        AlignItems::Center => Alignment::Center,
        AlignItems::End => Alignment::End,
    }
}

fn to_padding(padding: EdgeInsets) -> Padding {
    Padding {
        top: padding.top,
        right: padding.right,
        bottom: padding.bottom,
        left: padding.left,
    }
}

fn to_length(value: SizeValue) -> Length {
    match value {
        SizeValue::Auto | SizeValue::Shrink => Length::Shrink,
        SizeValue::Fill => Length::Fill,
        SizeValue::Px(px) => Length::Fixed(px as f32),
    }
}

impl From<StyleColor> for Color {
    fn from(value: StyleColor) -> Self {
        Self::from_rgba(value.red, value.green, value.blue, value.alpha)
    }
}

#[derive(Debug, Clone)]
struct NodeButtonStyle {
    appearance: AppearanceStyle,
    text_color: Option<StyleColor>,
}

impl NodeButtonStyle {
    fn new(appearance: AppearanceStyle, text_color: Option<StyleColor>) -> Self {
        Self {
            appearance,
            text_color,
        }
    }
}

impl iced::widget::button::StyleSheet for NodeButtonStyle {
    type Style = Theme;

    fn active(&self, theme: &Self::Style) -> iced::widget::button::Appearance {
        let palette = theme.extended_palette();

        iced::widget::button::Appearance {
            background: Some(
                self.appearance
                    .background_color
                    .map(Color::from)
                    .unwrap_or(palette.primary.strong.color)
                    .into(),
            ),
            border_radius: self.appearance.border_radius.into(),
            border_width: self.appearance.border_width,
            border_color: self
                .appearance
                .border_color
                .map(Color::from)
                .unwrap_or(Color::TRANSPARENT),
            text_color: self
                .text_color
                .map(Color::from)
                .unwrap_or(palette.primary.strong.text),
            ..iced::widget::button::Appearance::default()
        }
    }
}

#[derive(Debug, Clone)]
struct NodeTextInputStyle {
    appearance: AppearanceStyle,
    text_color: Option<StyleColor>,
}

impl NodeTextInputStyle {
    fn new(appearance: AppearanceStyle, text_color: Option<StyleColor>) -> Self {
        Self {
            appearance,
            text_color,
        }
    }
}

impl iced::widget::text_input::StyleSheet for NodeTextInputStyle {
    type Style = Theme;

    fn active(&self, theme: &Self::Style) -> iced::widget::text_input::Appearance {
        let palette = theme.extended_palette();

        iced::widget::text_input::Appearance {
            background: self
                .appearance
                .background_color
                .map(Color::from)
                .unwrap_or(palette.background.base.color)
                .into(),
            border_radius: self.appearance.border_radius.into(),
            border_width: self.appearance.border_width.max(1.0),
            border_color: self
                .appearance
                .border_color
                .map(Color::from)
                .unwrap_or(palette.background.strong.color),
            icon_color: self
                .text_color
                .map(Color::from)
                .unwrap_or(palette.background.weak.text),
        }
    }

    fn focused(&self, theme: &Self::Style) -> iced::widget::text_input::Appearance {
        self.active(theme)
    }

    fn placeholder_color(&self, theme: &Self::Style) -> Color {
        theme.extended_palette().background.strong.color
    }

    fn value_color(&self, theme: &Self::Style) -> Color {
        self.text_color
            .map(Color::from)
            .unwrap_or(theme.extended_palette().background.base.text)
    }

    fn disabled_color(&self, theme: &Self::Style) -> Color {
        let mut color = self.value_color(theme);
        color.a *= 0.5;
        color
    }

    fn selection_color(&self, theme: &Self::Style) -> Color {
        theme.extended_palette().primary.weak.color
    }

    fn disabled(&self, theme: &Self::Style) -> iced::widget::text_input::Appearance {
        let mut appearance = self.active(theme);
        appearance.background = Background::Color(theme.extended_palette().background.weak.color);
        appearance
    }
}
