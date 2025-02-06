use std::{
    borrow::Cow,
    cell::RefCell,
    fs::File,
    io::{self, BufReader},
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use ropey::Rope;
use thiserror::Error;

use crate::buffer::Buffer;

pub type BufferPointer = Arc<RefCell<Buffer>>;

/// The main engine
#[derive(Debug, Default)]
pub struct Engine {
    base: PathBuf,
    file: DashMap<PathBuf, BufferPointer>,
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Io Error: {0}")]
    Io(#[from] io::Error),
    #[error("Rope Error: {0}")]
    Rope(#[from] ropey::Error),
    #[error("Missing path: {0}")]
    MissingPath(PathBuf),
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

    pub fn get_buffer(
        &self,
        path: impl AsRef<Path>,
        relative: bool,
    ) -> EngineResult<BufferPointer> {
        let path = self.norm_path(path, relative)?;
        self.file
            .get(&path)
            .ok_or(EngineError::MissingPath(path))
            .map(|e| e.clone())
    }

    pub fn add_buffer(
        &self,
        path: impl AsRef<Path>,
        relative: bool,
    ) -> EngineResult<BufferPointer> {
        let path = self.norm_path(path, relative)?;
        let buf = Arc::new(RefCell::new(Buffer {
            path: path.clone(),
            text: maybe_read(&path)?.unwrap_or_default(),
            loc: Default::default(),
        }));
        self.file.insert(path, buf.clone());
        Ok(buf)
    }

    pub(crate) fn norm_path(
        &self,
        path: impl AsRef<Path>,
        relative: bool,
    ) -> EngineResult<PathBuf> {
        if relative {
            Ok(self.base.join(path))
        } else {
            Ok(path.as_ref().to_owned())
        }
    }
}

/// tries to read a file, returns none if it doesn't exist
fn maybe_read(path: &Path) -> EngineResult<Option<Rope>> {
    Ok(handle_nf(
        File::open(&path).and_then(|file| Rope::from_reader(BufReader::new(file))),
    )?)
}

fn handle_nf<T>(res: Result<T, io::Error>) -> Result<Option<T>, io::Error> {
    match res {
        Ok(value) => Ok(Some(value)),
        Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => Ok(None),
        Err(e) => Err(e.into()),
    }
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
    pub text: Cow<'a, str>,
}

#[derive(Debug)]
pub enum Edit<T> {
    One(T),
    Multi(Vec<T>),
}
