#![allow(unused)]
use std::{cell::RefCell, ops::DerefMut, time::Instant};

use iced::{
    advanced::{
        layout, mouse, renderer,
        text::{self, highlighter, Editor as _},
        widget::operation,
        Text, Widget,
    },
    alignment,
    widget::{
        text::{LineHeight, Wrapping},
        text_editor,
    },
    Font, Length, Padding, Pixels, Point, Renderer, Size,
};
use iced_renderer::core::{widget, SmolStr};
use ropey::Rope;

use crate::Message;

use self::internal::Editor;

mod internal;

pub fn evie_editor<'a, Theme>() -> TextEditor<'a, highlighter::PlainText, Theme>
where
    Theme: text_editor::Catalog,
{
    TextEditor::default()
}

#[derive(Debug)]
pub struct TextEditor<'a, Highlighter, Theme = iced::Theme>
where
    Highlighter: text::Highlighter,
    Theme: text_editor::Catalog,
    Renderer: text::Renderer,
{
    editor: Editor,
    font: Option<Font>,
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
    highlighter_format: fn(&Highlighter::Highlight, &Theme) -> highlighter::Format<Font>,
}

impl<'a, Theme> Default for TextEditor<'a, highlighter::PlainText, Theme>
where
    Theme: text_editor::Catalog,
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
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Char(SmolStr),
}

impl<'a, Highlighter, Theme> TextEditor<'a, Highlighter, Theme>
where
    Highlighter: text::Highlighter,
    Theme: text_editor::Catalog,
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

impl<'a, Highlighter, Theme> Widget<Message, Theme, Renderer> for TextEditor<'a, Highlighter, Theme>
where
    Highlighter: text::Highlighter,
    Theme: text_editor::Catalog,
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
        let mut internal = &self.editor;
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

        // internal.editor.update(
        //     limits.shrink(self.padding).max(),
        //     self.font.unwrap_or_else(|| renderer.default_font()),
        //     self.text_size.unwrap_or_else(|| renderer.default_size()),
        //     self.line_height,
        //     self.wrapping,
        //     state.highlighter.borrow_mut().deref_mut(),
        // );

        layout::Node::new(limits.max())
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
        use iced::{
            advanced::{text::Renderer as _, Renderer as _},
            widget::text_editor::Status,
        };

        let bounds = layout.bounds();

        let internal = &self.editor;
        let state = tree.state.downcast_ref::<State<Highlighter>>();

        let font = self.font.unwrap_or_else(|| renderer.default_font());

        // internal.editor.highlight(
        //     font,
        //     state.highlighter.borrow_mut().deref_mut(),
        //     |highlight| (self.highlighter_format)(highlight, theme),
        // );

        let is_disabled = self.on_edit.is_none();
        let is_mouse_over = cursor.is_over(bounds);

        let status = if is_disabled {
            Status::Disabled
        } else if state.focus.is_some() {
            Status::Focused
        } else if is_mouse_over {
            Status::Hovered
        } else {
            Status::Active
        };

        let style = theme.style(&self.class, status);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        let text_bounds = bounds.shrink(self.padding);

        renderer.fill_text(
            Text {
                content: internal.get_inner(),
                bounds: text_bounds.size(),
                size: self.text_size.unwrap_or_else(|| renderer.default_size()),
                line_height: self.line_height,
                font,
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: self.wrapping,
            },
            text_bounds.position(),
            style.value,
            text_bounds,
        );

        let translation = text_bounds.position() - Point::ORIGIN;

        let Some(focus) = state.focus.as_ref() else {
            return;
        };
        // match internal.editor.cursor() {
        //     Cursor::Caret(position) if focus.is_cursor_visible() => {
        //         let cursor = Rectangle::new(
        //             position + translation,
        //             Size::new(
        //                 1.0,
        //                 self.line_height
        //                     .to_absolute(
        //                         self.text_size.unwrap_or_else(|| renderer.default_size()),
        //                     )
        //                     .into(),
        //             ),
        //         );
        //
        //         if let Some(clipped_cursor) = text_bounds.intersection(&cursor) {
        //             renderer.fill_quad(
        //                 renderer::Quad {
        //                     bounds: clipped_cursor,
        //                     ..renderer::Quad::default()
        //                 },
        //                 style.value,
        //             );
        //         }
        //     }
        //     Cursor::Selection(ranges) => {
        //         for range in ranges
        //             .into_iter()
        //             .filter_map(|range| text_bounds.intersection(&(range + translation)))
        //         {
        //             renderer.fill_quad(
        //                 renderer::Quad {
        //                     bounds: range,
        //                     ..renderer::Quad::default()
        //                 },
        //                 style.selection,
        //             );
        //         }
        //     }
        //     Cursor::Caret(_) => {}
        // }
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State<Highlighter>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State {
            focus: None,
            last_click: None,
            drag_click: None,
            partial_scroll: 0.0,
            highlighter: RefCell::new(Highlighter::new(&self.highlighter_settings)),
            highlighter_settings: self.highlighter_settings.clone(),
            highlighter_format_address: self.highlighter_format as usize,
        })
    }

    fn operate(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        _layout: iced::advanced::Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Highlighter>>();

        operation.focusable(state, None);
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
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        let is_disabled = self.on_edit.is_none();

        if cursor.is_over(layout.bounds()) {
            if is_disabled {
                mouse::Interaction::NotAllowed
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

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

impl<Highlighter: text::Highlighter> operation::Focusable for State<Highlighter> {
    fn is_focused(&self) -> bool {
        self.focus.is_some()
    }

    fn focus(&mut self) {
        self.focus = Some(Focus::now());
    }

    fn unfocus(&mut self) {
        self.focus = None;
    }
}
