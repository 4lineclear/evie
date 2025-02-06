use std::cell::{BorrowError, BorrowMutError, Cell};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::DashMap;

use engine::Engine;
use thiserror::Error;

use self::engine::{BufferPointer, EngineError};

pub mod buffer;
pub mod engine;

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
        match self.trig.load().get(&key) {
            Some(Trigger::End(a)) => {
                self.trig.store(self.root());
                return Some(a);
            }
            Some(Trigger::Map(tm)) => self.trig.store(tm),
            Some(Trigger::Fail) => self.trig.store(self.root()),
            None => self.trig.store(self.root()),
        }
        None
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

pub type TriggerFallback<K> = Arc<dyn Fn(&K) -> Option<Trigger<K>>>;

#[derive(Clone)]
pub struct TriggerMap<K: Key> {
    inner: DashMap<K, Trigger<K>>,
    fallback: Option<TriggerFallback<K>>,
}

impl<K: Key + Default> Default for TriggerMap<K> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            fallback: None,
        }
    }
}

impl<K: Key + std::fmt::Debug> std::fmt::Debug for TriggerMap<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TriggerMap")
            .field("inner", &self.inner)
            .field("fallback", &self.fallback.as_ref().map(|_| ()))
            .finish()
    }
}

impl<K: Key> TriggerMap<K> {
    pub fn with_fallback(self, fallback: TriggerFallback<K>) -> Self {
        Self {
            inner: self.inner,
            fallback: Some(fallback),
        }
    }

    pub fn new(
        inner: impl IntoIterator<Item = (K, Trigger<K>)>,
        fallback: Option<TriggerFallback<K>>,
    ) -> Self {
        let inner = inner.into_iter().collect();
        Self { inner, fallback }
    }

    fn get(&self, key: &K) -> Option<Trigger<K>> {
        match self.inner.get(key).map(|r| r.clone()) {
            Some(trigger) => Some(trigger),
            None => self
                .fallback
                .as_ref()
                .and_then(|fallback: &TriggerFallback<K>| (fallback)(key)),
        }
    }
}

impl<K: Key, const N: usize> From<[(K, Trigger<K>); N]> for TriggerMap<K> {
    fn from(value: [(K, Trigger<K>); N]) -> Self {
        Self::new(value, None)
    }
}

impl<K: Key> From<TriggerFallback<K>> for TriggerMap<K> {
    fn from(value: TriggerFallback<K>) -> Self {
        Self {
            inner: Default::default(),
            fallback: Some(value),
        }
    }
}

impl<K: Key, const N: usize> From<([(K, Trigger<K>); N], TriggerFallback<K>)> for TriggerMap<K> {
    fn from(value: ([(K, Trigger<K>); N], TriggerFallback<K>)) -> Self {
        Self::from(value.0).with_fallback(value.1)
    }
}

#[derive(Clone, Default)]
pub enum Trigger<K: Key> {
    #[default]
    Fail,
    End(Action),
    Map(Arc<TriggerMap<K>>),
}

impl<K: Key> std::fmt::Debug for Trigger<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fail => write!(f, "Fail"),
            Self::End(ac) => f.debug_tuple("End").field(ac).finish(),
            Self::Map(tm) => f.debug_tuple("Map").field(tm).finish(),
        }
    }
}

impl<K: Key> From<Arc<TriggerMap<K>>> for Trigger<K> {
    fn from(value: Arc<TriggerMap<K>>) -> Self {
        Self::Map(value)
    }
}
// TODO: consider adding micro-mods

#[derive(Debug, Default)]
pub struct Modes<K: Key> {
    normal: Arc<TriggerMap<K>>,
    insert: Arc<TriggerMap<K>>,
    visual: Arc<TriggerMap<K>>,
    command: Arc<TriggerMap<K>>,
    replace: Arc<TriggerMap<K>>,
    terminal: Arc<TriggerMap<K>>,
    universal: Arc<TriggerMap<K>>,
}

impl<K: Key> Modes<K> {
    pub fn new(
        normal: impl Into<TriggerMap<K>>,
        insert: impl Into<TriggerMap<K>>,
        visual: impl Into<TriggerMap<K>>,
        command: impl Into<TriggerMap<K>>,
        replace: impl Into<TriggerMap<K>>,
        terminal: impl Into<TriggerMap<K>>,
        universal: impl Into<TriggerMap<K>>,
    ) -> Self {
        Self {
            normal: normal.into().into(),
            insert: insert.into().into(),
            visual: visual.into().into(),
            command: command.into().into(),
            replace: replace.into().into(),
            terminal: terminal.into().into(),
            universal: universal.into().into(),
        }
    }
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
