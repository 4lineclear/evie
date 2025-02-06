use std::io;
use std::{ops::Range, path::PathBuf};

use crate::engine::{Edit, EngineResult};
use crate::BufferAction;

use ropey::Rope;

#[cfg(test)]
mod test;

#[derive(Debug, Default)]
pub struct Buffer {
    pub loc: Location,
    pub path: PathBuf,
    pub text: Rope,
}

#[derive(Debug, Default)]
pub struct Location {
    pub char: usize,
}

impl Buffer {
    pub fn edit(&mut self, edit: Edit<BufferAction>) -> EngineResult<()> {
        match edit {
            Edit::One(cmd) => self.apply(cmd),
            Edit::Multi(vec) => vec.into_iter().try_for_each(|cmd| self.apply(cmd)),
        }
    }

    pub fn apply(&mut self, action: BufferAction) -> EngineResult<()> {
        match action {
            BufferAction::Append(text) => {
                self.text
                    .try_insert(self.fix_index(self.loc.char)?, &text)?;
                self.loc.char += text.len();
                Ok(())
            }
        }
    }

    fn fix_index(&self, i: usize) -> ropey::Result<usize> {
        self.text.try_byte_to_char(i)
    }

    #[allow(unused)]
    fn fix_range(&self, r: Range<usize>) -> ropey::Result<Range<usize>> {
        Ok(self.fix_index(r.start)?..self.fix_index(r.end)?)
    }

    #[allow(unused)]
    fn remove(&mut self, r: Range<usize>) -> ropey::Result<()> {
        self.text.try_remove(r.start..r.end)?;
        Ok(())
    }

    pub async fn write_async(&self) -> io::Result<()> {
        use tokio::{
            fs,
            io::{self, AsyncWriteExt},
        };
        let mut writer = io::BufWriter::new(fs::File::open(&self.path).await?);
        for chunk in self.text.chunks() {
            writer.write(chunk.as_bytes()).await?;
        }
        Ok(())
    }

    pub fn write(&self) -> io::Result<()> {
        use std::{fs, io};
        self.text
            .write_to(io::BufWriter::new(fs::File::open(&self.path)?))
    }
}
