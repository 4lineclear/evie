#![allow(unused)]
use std::{cell::RefCell, ops::DerefMut, time::Instant};

use iced::{
    advanced::{
        layout, mouse,
        text::{self, highlighter, Editor as _},
        Widget,
    },
    widget::{
        text::{LineHeight, Wrapping},
        text_editor,
    },
    Length, Padding, Pixels, Size,
};
use iced_renderer::core::SmolStr;
use ropey::Rope;

use crate::Message;

pub fn evie_editor<'a, Theme>() -> TextEditor<'a, highlighter::PlainText, Theme, super::Renderer>
where
    Theme: text_editor::Catalog,
{
    TextEditor::default()
}

#[derive(Debug)]
pub struct TextEditor<'a, Highlighter, Theme = iced::Theme, Renderer = crate::Renderer>
where
    Highlighter: text::Highlighter,
    Theme: text_editor::Catalog,
    Renderer: text::Renderer,
{
    editor: Editor,
    font: Option<Renderer::Font>,
    text_size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    padding: Padding,
    wrapping: Wrapping,
    class: Theme::Class<'a>,
    key_binding: Option<Box<dyn KeyBinding + 'a>>,
    on_edit: Option<Box<dyn OnEdit + 'a>>,
    highlighter_settings: Highlighter::Settings,
    highlighter_format: fn(&Highlighter::Highlight, &Theme) -> highlighter::Format<Renderer::Font>,
}

impl<'a, Theme, Renderer> Default for TextEditor<'a, highlighter::PlainText, Theme, Renderer>
where
    Theme: text_editor::Catalog,
    Renderer: text::Renderer,
{
    fn default() -> Self {
        Self {
            editor: Editor::default(),
            font: None,
            text_size: None,
            line_height: LineHeight::default(),
            width: Length::Fill,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            wrapping: Wrapping::default(),
            class: Theme::default(),
            key_binding: None,
            on_edit: None,
            highlighter_settings: (),
            highlighter_format: |_, _| highlighter::Format::default(),
            // width: Length::Fill,
            // height: Length::Shrink,
            // on_edit: None,
            // editor: Editor::default(),
            // font: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Editor {
    rope: Rope,
}

#[derive(Debug)]
pub enum Action {
    Char(SmolStr),
}

impl<'a, Highlighter, Theme, Renderer> TextEditor<'a, Highlighter, Theme, Renderer>
where
    Highlighter: text::Highlighter,
    Theme: text_editor::Catalog,
    Renderer: text::Renderer,
{
    fn apply_key(
        &mut self,
        key: iced::keyboard::Key,
    ) -> iced::advanced::graphics::core::event::Status {
        let Some(on_edit) = &self.on_edit else {
            return iced::advanced::graphics::core::event::Status::Ignored;
        };
        use iced::keyboard::Key::*;
        let Character(text) = key else {
            return iced::advanced::graphics::core::event::Status::Ignored;
        };
        (on_edit)(Action::Char(text.clone()));
        self.editor
            .rope
            .insert(self.editor.rope.len_chars(), text.as_str());
        iced::advanced::graphics::core::event::Status::Captured
    }
}

impl<'a, Highlighter, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TextEditor<'a, Highlighter, Theme, Renderer>
where
    Highlighter: text::Highlighter,
    Theme: text_editor::Catalog,
    Renderer: text::Renderer<Editor = Editor, Font = iced::Font>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        // let mut internal = self.content.0.borrow_mut();
        let state = tree.state.downcast_mut::<State<Highlighter>>();

        if state.highlighter_format_address != self.highlighter_format as usize {
            state.highlighter.borrow_mut().change_line(0);

            state.highlighter_format_address = self.highlighter_format as usize;
        }

        if state.highlighter_settings != self.highlighter_settings {
            state
                .highlighter
                .borrow_mut()
                .update(&self.highlighter_settings);

            state.highlighter_settings = self.highlighter_settings.clone();
        }

        let limits = limits.width(self.width).height(self.height);

        self.editor.update(
            limits.shrink(self.padding).max(),
            self.font.unwrap_or_else(|| renderer.default_font()),
            self.text_size.unwrap_or_else(|| renderer.default_size()),
            self.line_height,
            self.wrapping,
            state.highlighter.borrow_mut().deref_mut(),
        );

        match self.height {
            Length::Fill | Length::FillPortion(_) | Length::Fixed(_) => {
                layout::Node::new(limits.max())
            }
            Length::Shrink => {
                let min_bounds = self.editor.min_bounds();

                layout::Node::new(
                    limits
                        .height(min_bounds.height)
                        .max()
                        .expand(Size::new(0.0, self.padding.vertical())),
                )
            }
        }
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        todo!()
    }

    fn size_hint(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::stateless()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::None
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        Vec::new()
    }

    fn diff(&self, _tree: &mut iced::advanced::widget::Tree) {}

    fn operate(
        &self,
        _state: &mut iced::advanced::widget::Tree,
        _layout: iced::advanced::Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn iced::advanced::widget::Operation,
    ) {
    }

    fn on_event(
        &mut self,
        _state: &mut iced::advanced::widget::Tree,
        _event: iced::Event,
        _layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        _shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> iced_renderer::core::event::Status {
        iced_renderer::core::event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _state: &iced::advanced::widget::Tree,
        _layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        iced::advanced::mouse::Interaction::None
    }

    fn overlay<'b>(
        &'b mut self,
        _state: &'b mut iced::advanced::widget::Tree,
        _layout: iced::advanced::Layout<'_>,
        _renderer: &Renderer,
        _translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        None
    }
}

impl iced::advanced::text::Editor for Editor {
    type Font = iced::Font;

    fn with_text(text: &str) -> Self {
        todo!()
    }

    fn is_empty(&self) -> bool {
        todo!()
    }

    fn cursor(&self) -> text::editor::Cursor {
        todo!()
    }

    fn cursor_position(&self) -> (usize, usize) {
        todo!()
    }

    fn selection(&self) -> Option<String> {
        todo!()
    }

    fn line(&self, index: usize) -> Option<&str> {
        todo!()
    }

    fn line_count(&self) -> usize {
        todo!()
    }

    fn perform(&mut self, action: text_editor::Action) {
        todo!()
    }

    fn bounds(&self) -> Size {
        todo!()
    }

    fn min_bounds(&self) -> Size {
        todo!()
    }

    fn update(
        &mut self,
        new_bounds: Size,
        new_font: Self::Font,
        new_size: Pixels,
        new_line_height: LineHeight,
        new_wrapping: Wrapping,
        new_highlighter: &mut impl text::Highlighter,
    ) {
        todo!()
    }

    fn highlight<H: text::Highlighter>(
        &mut self,
        font: Self::Font,
        highlighter: &mut H,
        format_highlight: impl Fn(&H::Highlight) -> highlighter::Format<Self::Font>,
    ) {
        todo!()
    }
}

trait KeyBinding:
    Fn(text_editor::KeyPress) -> Option<text_editor::Binding<Message>> + std::fmt::Debug
{
}

impl<KB: Fn(text_editor::KeyPress) -> Option<text_editor::Binding<Message>> + std::fmt::Debug>
    KeyBinding for KB
{
}

trait OnEdit: Fn(Action) -> Message + std::fmt::Debug {}

impl<E: Fn(Action) -> Message + std::fmt::Debug> OnEdit for E {}

/// The state of a [`TextEditor`].
#[derive(Debug)]
pub struct State<Highlighter: text::Highlighter> {
    focus: Option<Focus>,
    last_click: Option<mouse::Click>,
    drag_click: Option<mouse::click::Kind>,
    partial_scroll: f32,
    highlighter: RefCell<Highlighter>,
    highlighter_settings: Highlighter::Settings,
    highlighter_format_address: usize,
}

#[derive(Debug, Clone, Copy)]
struct Focus {
    updated_at: Instant,
    now: Instant,
    is_window_focused: bool,
}

impl Focus {
    const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

    fn now() -> Self {
        let now = Instant::now();

        Self {
            updated_at: now,
            now,
            is_window_focused: true,
        }
    }

    fn is_cursor_visible(&self) -> bool {
        self.is_window_focused
            && ((self.now - self.updated_at).as_millis() / Self::CURSOR_BLINK_INTERVAL_MILLIS) % 2
                == 0
    }
}
