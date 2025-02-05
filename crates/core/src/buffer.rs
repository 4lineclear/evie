use std::io;
use std::{ops::Range, path::PathBuf};

use crate::engine::{Command, Edit};

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
    pub line: usize,
    pub cursor: usize,
    pub file: std::path::PathBuf,
}

impl Buffer {
    pub fn edit(&mut self, edit: Edit<Command>) -> ropey::Result<()> {
        match edit {
            Edit::One(cmd) => self.apply(cmd),
            Edit::Multi(vec) => vec.into_iter().try_for_each(|cmd| self.apply(cmd)),
        }
    }

    pub fn apply(&mut self, cmd: Command) -> ropey::Result<()> {
        match cmd {
            Command::Replace(r) => {
                let range = self.fix_range(r.range)?;
                let char_idx = range.start;
                self.remove(range)?;
                self.text.try_insert(char_idx, r.new_text)?;
            }
            Command::Delete(d) => self.remove(self.fix_range(d.range)?)?,
            Command::Insert(r) => self.text.try_insert(self.fix_index(r.index)?, r.text)?,
        }
        Ok(())
    }

    fn fix_index(&self, i: usize) -> ropey::Result<usize> {
        self.text.try_byte_to_char(i)
    }

    fn fix_range(&self, r: Range<usize>) -> ropey::Result<Range<usize>> {
        Ok(self.fix_index(r.start)?..self.fix_index(r.end)?)
    }

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
