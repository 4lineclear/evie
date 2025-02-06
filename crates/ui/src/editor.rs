use evie_core::BufferView;
use iced::advanced::widget::tree;
use iced::advanced::{layout, text, Widget};
use iced::keyboard::key;
use iced::{alignment, Element, Length, Theme};

use crate::{KeyAction, Message};

pub fn evie_editor(bf: BufferView<KeyAction>) -> Editor {
    Editor::new(bf)
}

#[derive(Debug)]
pub struct Editor {
    bv: BufferView<KeyAction>,
    width: Length,
    height: Length,
}

impl Editor {
    pub fn new(bv: BufferView<KeyAction>) -> Self {
        Self {
            bv,
            width: Length::Fill,
            height: Length::Shrink,
        }
    }
}

type EditorState = ();

impl<Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer>
    Widget<Message, Theme, Renderer> for Editor
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<tree::State>()
    }
    fn state(&self) -> tree::State {
        tree::State::Some(Box::new(EditorState::default()))
    }
    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        // TODO: actually use dimensions
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let _state: EditorState = *tree.state.downcast_ref();

        renderer.fill_text(
            text::Text {
                content: self.bv.rope().unwrap().chunks().collect(),
                bounds: bounds.size(),
                size: renderer.default_size(),
                line_height: text::LineHeight::default(),
                font: renderer.default_font(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            },
            bounds.position(),
            style.text_color,
            bounds,
        );
    }

    fn on_event(
        &mut self,
        _state: &mut tree::Tree,
        event: iced::Event,
        _layout: layout::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        _shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> iced_renderer::core::event::Status {
        if let Some(ka) = decode_event(event) {
            if self.bv.on_key(ka).unwrap() {
                return iced_renderer::core::event::Status::Captured;
            }
        }
        iced_renderer::core::event::Status::Ignored
    }
}

impl<'a> Into<Element<'a, Message>> for Editor {
    fn into(self) -> Element<'a, Message> {
        Element::new(self)
    }
}

fn decode_event(event: iced::Event) -> Option<KeyAction> {
    use iced::{keyboard::Event::*, Event::*};
    let Keyboard(event) = event else {
        return None;
    };
    match event {
        KeyPressed {
            key: key::Key::Named(key::Named::Escape),
            ..
        } => Some(KeyAction::Escape),
        KeyPressed {
            text: Some(text), ..
        } => Some(KeyAction::Letter(text.chars().find(|c| !c.is_control())?)),
        _ => None,
    }
}
