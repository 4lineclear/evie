use std::cell::Cell;
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::DashMap;

use engine::Engine;

pub mod buffer;
pub mod engine;

/// The central interface
#[derive(Debug, Default)]
pub struct Evie<K: Key> {
    pub mode: Cell<Mode>,
    pub engine: Engine,
    pub mdata: Modes<K>,
    pub trig: ArcSwap<TriggerMap<K>>,
}

impl<K: Key + Default> Evie<K> {
    pub fn new(mdata: Modes<K>) -> Self {
        Self {
            trig: ArcSwap::new(mdata.normal.clone()),
            mdata,
            ..Default::default()
        }
    }
}

impl<K: Key> Evie<K> {
    pub fn apply(&self, action: Action) -> Option<Effect> {
        match action {
            Action::Move(Move::Left) => todo!(),
            Action::Move(Move::Right) => todo!(),
            Action::Move(Move::Up) => todo!(),
            Action::Move(Move::Down) => todo!(),
        }
    }

    pub fn trigger(&self, key: K) -> Option<Action> {
        match self.trig.load().get(&key) {
            Some(Trigger::Map(dm)) => {
                self.trig.store(dm);
                None
            }
            Some(Trigger::End(e)) => {
                self.trig.store(self.root());
                Some(e)
            }
            _ => {
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
pub enum Effect {
    //
}

#[derive(Debug, Clone)]
pub enum Action {
    Move(Move),
}

#[derive(Debug, Clone)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Default)]
pub struct TriggerMap<K: Key> {
    inner: DashMap<K, Trigger<K>>,
}

impl<K: Key> TriggerMap<K> {
    fn get(&self, key: &K) -> Option<Trigger<K>> {
        self.inner.get(key).map(|r| r.clone())
    }
}

#[derive(Debug, Clone)]
pub enum Trigger<K: Key> {
    End(Action),
    Map(Arc<TriggerMap<K>>),
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

pub trait Key: std::hash::Hash + std::cmp::Eq + Copy {}
// impl<K: std::hash::Hash + std::cmp::Eq> Key for K {}

// pub enum KeyEvent {
//     Important(CommonKey),
// }
