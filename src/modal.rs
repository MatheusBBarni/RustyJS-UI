use iced::advanced::widget::Tree;
use iced::advanced::{layout, mouse, overlay, renderer, Clipboard, Layout, Shell, Widget};
use iced::keyboard;
use iced::{event, Element, Event, Length, Point, Rectangle, Size};

pub struct ModalHost<'a, Message, Renderer = iced::Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    content: Element<'a, Message, Renderer>,
    modals: Vec<RenderedModal<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> ModalHost<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    pub fn new(
        content: impl Into<Element<'a, Message, Renderer>>,
        modals: Vec<RenderedModal<'a, Message, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            modals,
        }
    }
}

pub struct RenderedModal<'a, Message, Renderer = iced::Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    content: Element<'a, Message, Renderer>,
    on_request_close: Option<Message>,
}

impl<'a, Message, Renderer> RenderedModal<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    pub fn new(
        content: impl Into<Element<'a, Message, Renderer>>,
        on_request_close: Option<Message>,
    ) -> Self {
        Self {
            content: content.into(),
            on_request_close,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for ModalHost<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::with_capacity(1 + self.modals.len());
        children.push(Tree::new(&self.content));
        children.extend(self.modals.iter().map(|modal| Tree::new(&modal.content)));
        children
    }

    fn diff(&self, tree: &mut Tree) {
        let expected_children = 1 + self.modals.len();

        if tree.children.len() != expected_children {
            tree.children = self.children();
            return;
        }

        tree.children[0].diff(&self.content);

        for (child_tree, modal) in tree.children[1..].iter_mut().zip(&self.modals) {
            child_tree.diff(&modal.content);
        }
    }

    fn width(&self) -> iced::Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> iced::Length {
        self.content.as_widget().height()
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget().layout(renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation<Message>,
    ) {
        self.content
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        if self.modals.is_empty() {
            return self
                .content
                .as_widget_mut()
                .overlay(&mut tree.children[0], layout, renderer);
        }

        let modals = self
            .modals
            .iter_mut()
            .zip(tree.children.iter_mut().skip(1))
            .map(|(modal, state)| ActiveModal {
                state,
                content: &mut modal.content,
                on_request_close: modal.on_request_close.clone(),
            })
            .collect();

        Some(overlay::Element::new(
            Point::ORIGIN,
            Box::new(ModalStackOverlay { modals }),
        ))
    }
}

impl<'a, Message, Renderer> From<ModalHost<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + renderer::Renderer,
{
    fn from(host: ModalHost<'a, Message, Renderer>) -> Self {
        Element::new(host)
    }
}

struct ActiveModal<'a, 'b, Message, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    state: &'b mut Tree,
    content: &'b mut Element<'a, Message, Renderer>,
    on_request_close: Option<Message>,
}

struct ModalStackOverlay<'a, 'b, Message, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    modals: Vec<ActiveModal<'a, 'b, Message, Renderer>>,
}

impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, Renderer>
    for ModalStackOverlay<'a, 'b, Message, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    fn layout(&self, renderer: &Renderer, bounds: Size, _position: Point) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds)
            .width(Length::Fill)
            .height(Length::Fill);

        layout::Node::with_children(
            bounds,
            self.modals
                .iter()
                .map(|modal| modal.content.as_widget().layout(renderer, &limits))
                .collect(),
        )
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation<Message>,
    ) {
        for (modal, child_layout) in self.modals.iter_mut().zip(layout.children()) {
            modal
                .content
                .as_widget()
                .operate(modal.state, child_layout, renderer, operation);
        }
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let mut layouts = layout.children().collect::<Vec<_>>();
        let Some(top_layout) = layouts.pop() else {
            return event::Status::Ignored;
        };
        let Some(top_modal) = self.modals.last_mut() else {
            return event::Status::Ignored;
        };

        let viewport = layout.bounds();
        let status = top_modal.content.as_widget_mut().on_event(
            top_modal.state,
            event.clone(),
            top_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &viewport,
        );

        if status == event::Status::Captured {
            return status;
        }

        if matches!(
            event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Escape,
                ..
            })
        ) {
            if let Some(message) = top_modal.on_request_close.clone() {
                shell.publish(message);
            }

            return event::Status::Captured;
        }

        match event {
            Event::Mouse(_) | Event::Touch(_) | Event::Keyboard(_) => event::Status::Captured,
            Event::Window(_) => event::Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some((modal, child_layout)) = self.modals.last().zip(layout.children().last()) else {
            return mouse::Interaction::default();
        };

        modal.content.as_widget().mouse_interaction(
            modal.state,
            child_layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();

        for (modal, child_layout) in self.modals.iter().zip(layout.children()) {
            modal.content.as_widget().draw(
                modal.state,
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                &viewport,
            );
        }
    }

    fn is_over(
        &self,
        layout: Layout<'_>,
        _renderer: &Renderer,
        cursor_position: iced::Point,
    ) -> bool {
        layout.bounds().contains(cursor_position)
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Renderer>> {
        let top_layout = layout.children().last()?;
        let top_modal = self.modals.last_mut()?;

        top_modal
            .content
            .as_widget_mut()
            .overlay(top_modal.state, top_layout, renderer)
    }
}
