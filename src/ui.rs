use crate::bridge::EventPayload;
use crate::style::{
    AlignItems, AppearanceStyle, Color as StyleColor, EdgeInsets, FlexDirection, JustifyContent,
    SizeValue, Style,
};
use crate::vdom::{ButtonNode, SelectInputNode, TextInputNode, TextNode, UiNode, ViewNode};
use iced::theme;
use iced::widget::{button, column, container, pick_list, row, text, text_input, Space};
use iced::{alignment, Alignment, Background, Color, Element, Length, Padding, Theme};
use serde_json::Value;
use std::rc::Rc;

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
        UiNode::SelectInput(select_node) => render_select_input(select_node, on_event),
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
    let (children, fill_main_axis) = apply_justify_content(
        children,
        node.style.layout.flex_direction,
        node.style.layout.justify_content,
    );

    let content = match node.style.layout.flex_direction {
        FlexDirection::Row => {
            let mut layout = row(children)
                .spacing(node.style.layout.spacing)
                .align_items(to_alignment(node.style.layout.align_items));

            if fill_main_axis {
                layout = layout.width(Length::Fill);
            }

            layout.into()
        }
        FlexDirection::Column => {
            let mut layout = column(children)
                .spacing(node.style.layout.spacing)
                .align_items(to_alignment(node.style.layout.align_items));

            if fill_main_axis {
                layout = layout.height(Length::Fill);
            }

            layout.into()
        }
    };

    wrap_view(content, &node.style)
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

fn render_select_input<'a, Message>(
    node: &'a SelectInputNode,
    on_event: impl Fn(EventPayload) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let placeholder = node.placeholder.as_deref().unwrap_or_default();
    let selected_label = node
        .selected_option()
        .map(|option| option.label.as_str())
        .unwrap_or(placeholder);

    if node.disabled || node.on_change.is_none() {
        return render_select_input_display(node, selected_label, node.selected_option().is_none());
    }

    let callback_id = node.on_change.as_ref().unwrap().id.clone();
    let pick_list_style = Rc::new(NodePickListStyle::new(
        node.style.appearance.clone(),
        node.style.text.color,
        false,
    ));
    let menu_style = Rc::new(NodeMenuStyle::new(
        node.style.appearance.clone(),
        node.style.text.color,
    ));

    let mut widget = pick_list(
        node.options.clone(),
        node.selected_option().cloned(),
        move |option| {
            on_event(EventPayload::new(
                callback_id.clone(),
                Value::String(option.value),
            ))
        },
    )
    .padding(to_padding(node.style.layout.padding))
    .width(to_length(node.style.layout.width))
    .text_size(node.style.text.font_size)
    .style(theme::PickList::Custom(pick_list_style, menu_style));

    if !placeholder.is_empty() {
        widget = widget.placeholder(placeholder);
    }

    wrap_with_container(widget.into(), &node.style, false, false)
}

fn render_select_input_display<'a, Message>(
    node: &'a SelectInputNode,
    label: &'a str,
    is_placeholder: bool,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let mut label_widget = text(label).size(node.style.text.font_size);

    if let Some(color) = node.style.text.color {
        let mut color = Color::from(color);

        if node.disabled || is_placeholder {
            color.a *= 0.7;
        }

        label_widget = label_widget.style(color);
    }

    let mut chevron = text("v").size(node.style.text.font_size);

    if let Some(color) = node.style.text.color {
        let mut color = Color::from(color);

        if node.disabled {
            color.a *= 0.7;
        }

        chevron = chevron.style(color);
    }

    let content = row(vec![
        label_widget.into(),
        Space::with_width(Length::Fill).into(),
        chevron.into(),
    ])
    .align_items(Alignment::Center);

    wrap_with_container(content.into(), &node.style, true, true)
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

fn wrap_view<'a, Message>(content: Element<'a, Message>, style: &Style) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let mut wrapper = container(content)
        .padding(to_padding(style.layout.padding))
        .width(to_length(style.layout.width))
        .height(to_length(style.layout.height))
        .align_x(view_horizontal_alignment(style))
        .align_y(view_vertical_alignment(style));

    if has_appearance(style) {
        let style_snapshot = style.clone();
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
) -> (Vec<Element<'a, Message>>, bool)
where
    Message: Clone + 'a,
{
    if children.is_empty() {
        return (children, false);
    }

    match justify {
        JustifyContent::Start | JustifyContent::Center | JustifyContent::End => (children, false),
        JustifyContent::SpaceBetween => (
            intersperse_spaces(children, direction, false, false, 1),
            true,
        ),
        JustifyContent::SpaceAround => {
            (intersperse_spaces(children, direction, true, true, 1), true)
        }
        JustifyContent::SpaceEvenly => {
            (intersperse_spaces(children, direction, true, true, 2), true)
        }
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

fn to_horizontal_alignment(value: AlignItems) -> alignment::Horizontal {
    match value {
        AlignItems::Start | AlignItems::Stretch => alignment::Horizontal::Left,
        AlignItems::Center => alignment::Horizontal::Center,
        AlignItems::End => alignment::Horizontal::Right,
    }
}

fn to_vertical_alignment(value: AlignItems) -> alignment::Vertical {
    match value {
        AlignItems::Start | AlignItems::Stretch => alignment::Vertical::Top,
        AlignItems::Center => alignment::Vertical::Center,
        AlignItems::End => alignment::Vertical::Bottom,
    }
}

fn justify_to_horizontal_alignment(value: JustifyContent) -> alignment::Horizontal {
    match value {
        JustifyContent::Start
        | JustifyContent::SpaceBetween
        | JustifyContent::SpaceAround
        | JustifyContent::SpaceEvenly => alignment::Horizontal::Left,
        JustifyContent::Center => alignment::Horizontal::Center,
        JustifyContent::End => alignment::Horizontal::Right,
    }
}

fn justify_to_vertical_alignment(value: JustifyContent) -> alignment::Vertical {
    match value {
        JustifyContent::Start
        | JustifyContent::SpaceBetween
        | JustifyContent::SpaceAround
        | JustifyContent::SpaceEvenly => alignment::Vertical::Top,
        JustifyContent::Center => alignment::Vertical::Center,
        JustifyContent::End => alignment::Vertical::Bottom,
    }
}

fn view_horizontal_alignment(style: &Style) -> alignment::Horizontal {
    match style.layout.flex_direction {
        FlexDirection::Row => justify_to_horizontal_alignment(style.layout.justify_content),
        FlexDirection::Column => to_horizontal_alignment(style.layout.align_items),
    }
}

fn view_vertical_alignment(style: &Style) -> alignment::Vertical {
    match style.layout.flex_direction {
        FlexDirection::Row => to_vertical_alignment(style.layout.align_items),
        FlexDirection::Column => justify_to_vertical_alignment(style.layout.justify_content),
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

#[derive(Debug, Clone)]
struct NodePickListStyle {
    appearance: AppearanceStyle,
    text_color: Option<StyleColor>,
    disabled: bool,
}

impl NodePickListStyle {
    fn new(appearance: AppearanceStyle, text_color: Option<StyleColor>, disabled: bool) -> Self {
        Self {
            appearance,
            text_color,
            disabled,
        }
    }
}

impl iced::widget::pick_list::StyleSheet for NodePickListStyle {
    type Style = Theme;

    fn active(&self, theme: &Self::Style) -> iced::widget::pick_list::Appearance {
        let palette = theme.extended_palette();
        let mut text_color = self
            .text_color
            .map(Color::from)
            .unwrap_or(palette.background.base.text);
        let mut placeholder_color = palette.background.strong.color;
        let mut handle_color = self
            .text_color
            .map(Color::from)
            .unwrap_or(palette.background.weak.text);

        if self.disabled {
            text_color.a *= 0.6;
            placeholder_color.a *= 0.6;
            handle_color.a *= 0.6;
        }

        iced::widget::pick_list::Appearance {
            text_color,
            placeholder_color,
            handle_color,
            background: self
                .appearance
                .background_color
                .map(Color::from)
                .unwrap_or_else(|| {
                    if self.disabled {
                        palette.background.weak.color
                    } else {
                        palette.background.base.color
                    }
                })
                .into(),
            border_radius: self.appearance.border_radius.into(),
            border_width: self.appearance.border_width.max(1.0),
            border_color: self
                .appearance
                .border_color
                .map(Color::from)
                .unwrap_or(palette.background.strong.color),
        }
    }

    fn hovered(&self, theme: &Self::Style) -> iced::widget::pick_list::Appearance {
        if self.disabled {
            return self.active(theme);
        }

        let palette = theme.extended_palette();
        let mut appearance = self.active(theme);

        if self.appearance.border_color.is_none() {
            appearance.border_color = palette.primary.strong.color;
        }

        appearance
    }
}

#[derive(Debug, Clone)]
struct NodeMenuStyle {
    appearance: AppearanceStyle,
    text_color: Option<StyleColor>,
}

impl NodeMenuStyle {
    fn new(appearance: AppearanceStyle, text_color: Option<StyleColor>) -> Self {
        Self {
            appearance,
            text_color,
        }
    }
}

impl iced::overlay::menu::StyleSheet for NodeMenuStyle {
    type Style = Theme;

    fn appearance(&self, theme: &Self::Style) -> iced::overlay::menu::Appearance {
        let palette = theme.extended_palette();

        iced::overlay::menu::Appearance {
            text_color: self
                .text_color
                .map(Color::from)
                .unwrap_or(palette.background.base.text),
            background: self
                .appearance
                .background_color
                .map(Color::from)
                .unwrap_or(palette.background.weak.color)
                .into(),
            border_width: self.appearance.border_width.max(1.0),
            border_radius: self.appearance.border_radius.into(),
            border_color: self
                .appearance
                .border_color
                .map(Color::from)
                .unwrap_or(palette.background.strong.color),
            selected_text_color: palette.primary.strong.text,
            selected_background: palette.primary.weak.color.into(),
        }
    }
}
