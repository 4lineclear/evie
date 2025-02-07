use std::sync::Arc;

use dashmap::DashMap;

use crate::{Action, Key};

// TODO: consider creating proc macro for generating trigger trees

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

    pub(crate) fn get(&self, key: &K) -> Option<Trigger<K>> {
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

#[derive(Clone)]
pub enum Trigger<K: Key> {
    End(Action),
    Map(Arc<TriggerMap<K>>),
}

impl<K: Key> std::fmt::Debug for Trigger<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    pub(super) normal: Arc<TriggerMap<K>>,
    pub(super) insert: Arc<TriggerMap<K>>,
    pub(super) visual: Arc<TriggerMap<K>>,
    pub(super) command: Arc<TriggerMap<K>>,
    pub(super) replace: Arc<TriggerMap<K>>,
    pub(super) terminal: Arc<TriggerMap<K>>,
    pub(super) universal: Arc<TriggerMap<K>>,
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
            normal: Arc::new(normal.into()),
            insert: Arc::new(insert.into()),
            visual: Arc::new(visual.into()),
            command: Arc::new(command.into()),
            replace: Arc::new(replace.into()),
            terminal: Arc::new(terminal.into()),
            universal: Arc::new(universal.into()),
        }
    }
}
