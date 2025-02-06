use evie_core::{Evie, EvieCentral};
use iced::{widget::scrollable, Element, Task};

use editor::evie_editor;
use trigger::modes;

pub mod editor;
pub mod trigger;

pub type IceResult = iced::Result;

pub fn launch() -> IceResult {
    iced::application("Editor - Iced", EvieMain::update, EvieMain::view)
        //  .theme(EvieMain::theme);
        // .default_font(Font::MONOSPACE)
        .run_with(EvieMain::new)
}

#[derive(Debug)]
struct EvieMain {
    inner: EvieCentral<KeyAction>, // content: Content,
}

#[derive(Debug, Clone)]
pub enum Message {
    Idle,
}

impl EvieMain {
    fn new() -> (Self, Task<Message>) {
        let evie_main = Self {
            inner: Evie::central(modes()),
        };
        let task = iced::window::get_latest().and_then(|id| iced::window::maximize(id, true));
        (evie_main, task)
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        // if let Message::Action(_action) = message {}
        Task::none()
    }
    fn view(&self) -> Element<Message> {
        self.inner.add_buffer("yeah.txt", true).unwrap();
        scrollable(evie_editor(
            self.inner.view_buffer("yeah.txt", true).unwrap(),
        ))
        .into()
    }
    // fn theme(&self) -> Theme {
    //     if self.theme.is_dark() {
    //         Theme::Dark
    //     } else {
    //         Theme::Light
    //     }
    // }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum KeyAction {
    Letter(char),
    Escape,
    Enter,
}

impl evie_core::Key for KeyAction {}
