use iced::Size;
use ropey::Rope;

#[derive(Debug, Default)]
pub struct Editor {
    pub(crate) rope: Rope,
}

impl Editor {
    pub fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }
    pub fn get_inner(&self) -> String {
        self.rope.chunks().collect()
    }
}
