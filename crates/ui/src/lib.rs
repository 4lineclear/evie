use iced::widget::text_editor::{Action, Edit};
use iced::widget::{text_editor, text_editor::Content};
use iced::Length::Fill;
use iced::{Element, Subscription, Task};

// type Element<'a, T> = iced::Element<'a, T, iced::Theme, Renderer>;
// type Renderer = renderer::Renderer<iced::Renderer>;

pub type IceResult = iced::Result;

pub fn launch() -> IceResult {
    // todo!()
    iced::application("Editor - Iced", EvieMain::update, EvieMain::view)
        //  .theme(EvieMain::theme);
        // .default_font(Font::MONOSPACE)
        .run_with(EvieMain::new)
}

#[derive(Debug, Default)]
struct EvieMain {
    // theme: highlighter::Theme,
    content: Content,
}

#[derive(Debug, Clone)]
pub enum Message {
    Idle,
    Action(Action),
}

impl EvieMain {
    fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            // theme: highlighter::Theme::Base16Mocha,
            // Task::none(),
            iced::window::get_latest().and_then(|id| iced::window::maximize(id, true)),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::Action(action) = message {
            self.content.perform(action);
        }
        Task::none()
    }
    fn view(&self) -> Element<Message> {
        // |a| {println!("{a:#?}");
        // Message::Idle}
        text_editor(&self.content)
            .height(Fill)
            .on_action(Message::Action)
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
