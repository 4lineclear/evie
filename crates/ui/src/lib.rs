use evie_core::{Evie, EvieCentral};
use iced::{Element, Task};

use editor::evie_editor;
use trigger::modes;

pub mod editor;
pub mod trigger;

pub type IceResult = iced::Result;

pub const DEFAULT_FONT: iced::Font = iced::Font {
    family: iced::font::Family::Name("Fira Code"),
    ..iced::Font::MONOSPACE
};

pub fn launch() -> IceResult {
    iced::application("Editor - Iced", EvieMain::update, EvieMain::view)
        .font(include_bytes!("../font/ttf/FiraCode-Regular.ttf"))
        .default_font(DEFAULT_FONT)
        .theme(|_| iced::Theme::Dracula)
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
        evie_editor(self.inner.view_buffer("yeah.txt", true).unwrap()).into()
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
    Named(Named),
}

macro_rules! gen_named {
    (enum $name:ident { $($ID:ident),* $(,)? }) => {
        #[derive(Debug, Hash, PartialEq, Eq, Clone)]
        pub enum $name {
            $($ID),*
        }
        impl $name {
            pub fn from_iced(iced: iced::keyboard::key::Named) -> Option<Named> {
                match iced {
                    $(iced::keyboard::key::Named::$ID => Some($name::$ID),)*
                    _ => None
                }
            }
        }
    };
}

#[rustfmt::skip]
gen_named! (
    enum Named {
        Alt, AltGraph, CapsLock, Control, Fn, FnLock, NumLock, ScrollLock, Shift, Meta, Super,
        Enter, Tab, Space, ArrowDown, ArrowLeft, ArrowRight, ArrowUp,
        End, Home, PageDown, PageUp, Backspace, Delete, Insert, Escape,
        Copy, Cut, Paste, Undo, Redo, Select, ContextMenu,
        F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    }
    // PrintScreen, Pause, Power, PowerOff, Standby, WakeUp, LogOff, Hibernate,
    // VolumeUp, VolumeDown, VolumeMute, MediaPlayPause, MediaStop, MediaNext, MediaPrevious
);

impl evie_core::Key for KeyAction {}
