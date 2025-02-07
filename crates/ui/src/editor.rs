use std::cell::RefCell;

use evie_core::BufferView;
use iced::advanced::text::Paragraph as _;
use iced::advanced::text::Renderer as _;
use iced::advanced::widget::tree;
use iced::advanced::{layout, text, Widget};
use iced::advanced::{renderer, Renderer as _};
use iced::keyboard::key;
use iced::widget;
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::Size;
use iced::{alignment, Background, Border, Color, Element, Length, Padding, Pixels, Theme};
use iced_renderer::graphics::text::Paragraph;

use crate::{KeyAction, Message, Named, DEFAULT_FONT};

pub fn evie_editor(bf: BufferView<KeyAction>) -> Editor {
    Editor::new(bf)
}

#[derive(Debug)]
pub struct Editor {
    bv: BufferView<KeyAction>,
    // ed: iced::widget::TextEditor<>,
    styling: Styling,
}

#[derive(Debug)]
pub struct Styling {
    width: Length,
    height: Length,
    font: Option<iced::Font>,
    text_size: Option<Pixels>,
    line_height: Option<LineHeight>,
    padding: Padding,
    wrapping: Wrapping,
}

impl Styling {
    fn new() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            font: None,
            text_size: None,
            line_height: None,
            padding: Padding::new(5.0),
            wrapping: Wrapping::default(),
        }
    }
}

impl Editor {
    pub fn new(bv: BufferView<KeyAction>) -> Self {
        Self {
            bv,
            styling: Styling::new(),
        }
    }

    fn update_state(
        &self,
        renderer: &iced::Renderer,
        state: &mut State,
        styling: &Styling,
        text_bounds: iced::Rectangle,
    ) {
        state.pg = Paragraph::with_text(text::Text {
            content: &self.bv.rope().unwrap().chunks().collect::<String>(),
            bounds: text_bounds.size(),
            size: styling.text_size.unwrap_or_else(|| renderer.default_size()),
            line_height: styling.line_height.unwrap_or_default(),
            font: styling.font.unwrap_or(DEFAULT_FONT),
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: Shaping::Advanced,
            wrapping: styling.wrapping,
        });
        // println!("{:#?}", state.pg.buffer());
    }
}

type EditorState = RefCell<State>;

#[derive(Debug, Default)]
struct State {
    pg: Paragraph,
}

impl Widget<Message, Theme, iced::Renderer> for Editor {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<tree::State>()
    }
    fn state(&self) -> tree::State {
        tree::State::Some(Box::new(EditorState::default()))
    }
    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size {
            width: self.styling.width,
            height: self.styling.height,
        }
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let limits = limits.width(self.styling.width).height(self.styling.height);
        if let Length::Shrink = self.styling.height {
            let state = tree.state.downcast_ref::<EditorState>().borrow();
            let min_bounds = state.pg.min_bounds();
            layout::Node::new(
                limits
                    .height(min_bounds.height)
                    .max()
                    .expand(Size::new(0.0, self.styling.padding.vertical())),
            )
        } else {
            layout::Node::new(limits.max())
        }
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let mut state = tree.state.downcast_ref::<EditorState>().borrow_mut();

        let bounds = layout.bounds();
        let styling = &self.styling;
        let style = default(&theme);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        let text_bounds = bounds.shrink(styling.padding);
        self.update_state(renderer, &mut state, styling, text_bounds);
        renderer.fill_paragraph(&state.pg, text_bounds.position(), style.value, text_bounds);
    }

    fn on_event(
        &mut self,
        tree: &mut tree::Tree,
        event: iced::Event,
        layout: layout::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        _shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> iced_renderer::core::event::Status {
        if let Some(ka) = decode_event(event) {
            if self.bv.on_key(ka).unwrap() {
                self.update_state(
                    renderer,
                    &mut tree.state.downcast_ref::<EditorState>().borrow_mut(),
                    &self.styling,
                    layout.bounds().shrink(self.styling.padding),
                );
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
            key: key::Key::Named(named),
            ..
        } => Some(KeyAction::Named(Named::from_iced(named)?)),
        KeyPressed {
            text: Some(text), ..
        } => Some(KeyAction::Letter(text.chars().find(|c| !c.is_control())?)),
        _ => None,
    }
}

#[derive(Debug)]
pub struct Style {
    pub background: Background,
    pub border: Border,
    pub icon: Color,
    pub placeholder: Color,
    pub value: Color,
    pub selection: Color,
}

/// The default style of a [`TextEditor`].
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        icon: palette.background.weak.text,
        placeholder: palette.background.strong.color,
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    }

    // match status {
    //     Status::Active => active,
    //     Status::Hovered => Style {
    //         border: Border {
    //             color: palette.background.base.text,
    //             ..active.border
    //         },
    //         ..active
    //     },
    //     Status::Focused => Style {
    //         border: Border {
    //             color: palette.primary.strong.color,
    //             ..active.border
    //         },
    //         ..active
    //     },
    //     Status::Disabled => Style {
    //         background: Background::Color(palette.background.weak.color),
    //         value: active.placeholder,
    //         ..active
    //     },
    // }
}
