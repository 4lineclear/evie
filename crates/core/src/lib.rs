use std::{
    cell::{BorrowError, BorrowMutError, Cell},
    path::{Path, PathBuf},
    sync::Arc,
};

use arc_swap::ArcSwap;

use engine::Engine;
use thiserror::Error;

use engine::{BufferPointer, EngineError};
use trigger::{Modes, Trigger, TriggerMap};

pub mod buffer;
pub mod engine;
pub mod trigger;

#[derive(Debug, Error)]
pub enum EvieError {
    #[error("inner borrow failed: {0}")]
    Borrow(#[from] BorrowError),
    #[error("inner borrow failed: {0}")]
    BorrowMut(#[from] BorrowMutError),
    #[error("engine error: {0}")]
    Engine(#[from] EngineError),
}

/// The central interface
#[derive(Debug, Default)]
pub struct Evie<K: Key> {
    pub mode: Cell<Mode>,
    pub engine: Engine,
    pub mdata: Modes<K>,
    pub trig: ArcSwap<TriggerMap<K>>,
}

pub type EvieCentral<K> = Arc<Evie<K>>;

impl<K: Key> Evie<K> {
    pub fn central(mdata: Modes<K>) -> EvieCentral<K> {
        Arc::new(Self::new(mdata))
    }
    pub fn new(mdata: Modes<K>) -> Self {
        Self {
            mode: Default::default(),
            engine: Default::default(),
            trig: mdata.normal.clone().into(),
            mdata,
        }
    }

    pub fn view_buffer(
        self: &Arc<Self>,
        path: impl AsRef<Path>,
        relative: bool,
    ) -> Result<BufferView<K>, EngineError> {
        Ok(BufferView {
            evie: self.clone(),
            buffer: self.engine.norm_path(path, relative)?,
        })
    }

    pub fn add_buffer(
        &self,
        path: impl AsRef<Path>,
        relative: bool,
    ) -> Result<BufferPointer, EngineError> {
        self.engine.add_buffer(path, relative)
    }

    fn apply(&self, action: CoreAction) -> Result<(), EvieError> {
        match action {
            CoreAction::SetMode(mode) => self.change_mode(mode),
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct BufferView<K: Key> {
    evie: EvieCentral<K>,
    buffer: PathBuf,
}

impl<K: Key> BufferView<K> {
    pub fn rope(&self) -> Result<ropey::Rope, EvieError> {
        Ok(self
            .evie
            .engine
            .get_buffer(&self.buffer, true)?
            .borrow()
            .text
            .clone())
    }
    pub fn on_key(&self, key: K) -> Result<bool, EvieError> {
        let Some(action) = self.evie.trigger(key) else {
            return Ok(false);
        };
        match action {
            Action::Core(ca) => self.evie.apply(ca)?,
            Action::Buffer(ba) => self
                .evie
                .engine
                .get_buffer(&self.buffer, true)?
                .try_borrow_mut()?
                .apply(ba)?,
        }
        Ok(true)
    }
}

impl<K: Key> Evie<K> {
    pub fn trigger(&self, key: K) -> Option<Action> {
        match self.do_trig(self.trig.load().get(&key)) {
            Some(output) => output,
            None => self.do_trig(self.mdata.universal.get(&key)).flatten(),
        }
    }
    pub fn do_trig(&self, trig: Option<Trigger<K>>) -> Option<Option<Action>> {
        match trig {
            Some(Trigger::End(a)) => {
                self.trig.store(self.root());
                return Some(Some(a));
            }
            Some(Trigger::Map(tm)) => {
                self.trig.store(tm);
                Some(None)
            }
            None => {
                self.trig.store(self.root());
                None
            }
        }
    }

    pub fn change_mode(&self, mode: Mode) {
        self.mode.set(mode);
        self.trig.store(self.root());
    }

    fn root(&self) -> Arc<TriggerMap<K>> {
        match self.mode.get() {
            Mode::Normal => &self.mdata.normal,
            Mode::Insert => &self.mdata.insert,
            Mode::Visual => &self.mdata.visual,
            Mode::Command => &self.mdata.command,
            Mode::Replace => &self.mdata.replace,
            Mode::Terminal => &self.mdata.terminal,
        }
        .clone()
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Core(CoreAction),
    Buffer(BufferAction),
}

#[derive(Debug, Clone)]
pub enum CoreAction {
    SetMode(Mode),
}

#[derive(Debug, Clone)]
pub enum BufferAction {
    Append(String),
}

impl From<CoreAction> for Action {
    fn from(value: CoreAction) -> Self {
        Self::Core(value)
    }
}

impl From<BufferAction> for Action {
    fn from(value: BufferAction) -> Self {
        Self::Buffer(value)
    }
}

#[derive(Debug, Clone)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

// TODO: consider making this a trait instead.
#[derive(Debug, Default, Clone, Copy)]
pub enum Mode {
    /// The default mode
    #[default]
    Normal,
    /// The familiar, plain mode
    Insert,
    /// The selection mode
    Visual,
    /// The command mode
    Command,
    /// Like insert, but overwrites instead of inserts
    Replace,
    /// Terminal support
    Terminal,
}

pub trait Key: std::hash::Hash + std::cmp::Eq + std::fmt::Debug + Clone {}
// impl<K: std::hash::Hash + std::cmp::Eq> Key for K {}

// pub enum KeyEvent {
//     Important(CommonKey),
// }
