use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::reader::buffer::BookProgressData;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct LocalBookData {
    pub name: String,
    pub path_to_book: String,
    pub hash: String,
    pub progress: BookProgressData,
    pub format: BookType,
}

impl LocalBookData {
    pub fn create(file: impl Into<String>) -> Result<Self> {
        let str = file.into();
        let hash = hash_file(str.clone());

        let path = Path::new(&str);

        // It's unlikely there is no file name...
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let extension = Path::new(&str).extension();

        let format = match extension {
            Some(ext) => match ext.to_str().unwrap() {
                "txt" => BookType::Txt,
                _ => BookType::Unknown,
            },
            None => BookType::Unknown,
        };
        let progress = BookProgressData::from_file(&str)?;

        Ok(Self {
            name,
            path_to_book: str,
            hash,
            format,
            progress,
        })
    }

    /// Returns the current line, total lines, then the percentage through (in terms of lines)
    ///
    /// Adds 1 to get_line(), as we want to work out 1 indexed line number.
    pub fn get_progress_display(&self) -> (usize, usize, f64) {
        return (
            self.progress.get_line() + 1,
            self.progress.get_total_lines(),
            self.progress.get_pct(),
        );
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum BookType {
    Unknown,
    Txt,
}

pub fn hash_file(file: impl Into<String>) -> String {
    sha256::digest(file.into())
}
