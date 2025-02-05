use iced::{Element, Subscription, Task};

// type Element<'a, T> = iced::Element<'a, T, iced::Theme, Renderer>;
// type Renderer = renderer::Renderer<iced::Renderer>;

pub type IceResult = iced::Result;

pub mod editor;

pub fn launch() -> IceResult {
    // todo!()
    iced::application("Editor - Iced", EvieMain::update, EvieMain::view)
        //  .theme(EvieMain::theme);
        .subscription(EvieMain::subscription)
        // .default_font(Font::MONOSPACE)
        .run_with(EvieMain::new)
}

struct EvieMain {
    // theme: highlighter::Theme,
}

#[derive(Debug, Clone)]
pub enum Message {
    Idle,
}

impl EvieMain {
    fn new() -> (Self, Task<Message>) {
        (
            Self {},
            // theme: highlighter::Theme::Base16Mocha,
            // Task::none(),
            iced::window::get_latest().and_then(|id| iced::window::maximize(id, true)),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
        // event::listen_with(|event, _, _| match event {
        //     iced::Event::Keyboard(event) => todo!(),
        //     iced::Event::Mouse(event) => None,
        //     iced::Event::Window(event) => None,
        //     iced::Event::Touch(event) => None,
        // })
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
        // use keyboard::key::Named::*;
        // match message {
        // Message::KeyPress { key, mods } => match key {
        //     Key::Named(named) => match named {
        //         _ => todo!(),
        //     },
        //     Key::Character(_) => todo!(),
        //     Key::Unidentified => todo!(),
        // },
        // Message::KeyRelease { key, mods } => todo!(),
        // }
    }
    fn view(&self) -> Element<Message> {
        // |a| {println!("{a:#?}");
        // Message::Idle}
        Element::new(editor::evie_editor())
    }

    // fn theme(&self) -> Theme {
    //     if self.theme.is_dark() {
    //         Theme::Dark
    //     } else {
    //         Theme::Light
    //     }
    // }
}
