use std::{
    fs::File,
    io::{self, BufReader},
    ops::Range,
    path::{Path, PathBuf},
};

use dashmap::DashMap;
use ropey::Rope;
use thiserror::Error;

use crate::buffer::Buffer;

/// The main engine
#[derive(Debug, Default)]
pub struct Engine {
    base: PathBuf,
    file: DashMap<PathBuf, Buffer>,
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Io Error: {0}")]
    Io(#[from] io::Error),
    #[error("Rope Error: {0}")]
    Rope(#[from] ropey::Error),
}

pub type EngineResult<T> = Result<T, EngineError>;

impl Engine {
    pub fn new(base: PathBuf) -> EngineResult<Self> {
        let base = base.canonicalize()?;
        Ok(Self {
            base,
            ..Default::default()
        })
    }

    pub fn add_file(&self, path: PathBuf, relative: bool) -> EngineResult<Option<Buffer>> {
        let path = if relative {
            self.base.join(path)
        } else {
            path.canonicalize()?
        };
        let text = maybe_read(&path)?.unwrap_or_default();
        let loc = Default::default();
        Ok(self.file.insert(path.clone(), Buffer { path, text, loc }))
    }
}

/// tries to read a file, returns none if it doesn't exist
fn maybe_read(path: &Path) -> EngineResult<Option<Rope>> {
    match File::open(&path) {
        Ok(file) => Ok(Some(Rope::from_reader(BufReader::new(file))?)),
        Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug)]
pub enum Command<'a> {
    Replace(Replace<'a>),
    Delete(Delete),
    Insert(Insert<'a>),
}

#[derive(Debug)]
pub struct Replace<'a> {
    pub range: Range<usize>,
    pub new_text: &'a str,
}

#[derive(Debug)]
pub struct Delete {
    pub range: Range<usize>,
}

#[derive(Debug)]
pub struct Insert<'a> {
    pub index: usize,
    pub text: &'a str,
}

#[derive(Debug)]
pub enum Edit<T> {
    One(T),
    Multi(Vec<T>),
}
