use crate::{HistoryContext, LibraryContext};
use anyhow::Result;
use std::{fs, path::PathBuf};

pub(super) fn store_library(library: &LibraryContext, path: &PathBuf) -> Result<()> {
    let json = if cfg!(debug_assertations) {
        serde_json::to_string_pretty(library)?
    } else {
        serde_json::to_string(library)?
    };
    fs::write(path.join("books.json"), json)?;
    Ok(())
}

pub(super) fn load_library(path: &PathBuf) -> Result<LibraryContext> {
    if let Ok(data) = fs::read_to_string(path.join("books.json")) {
        Ok(serde_json::from_str(&data)?)
    } else {
        Ok(LibraryContext::new())
    }
}

pub(super) fn store_history(history: &HistoryContext, path: &PathBuf) -> Result<()> {
    let json = if cfg!(debug_assertations) {
        serde_json::to_string_pretty(history)?
    } else {
        serde_json::to_string(history)?
    };
    fs::write(path.join("history.json"), json)?;
    Ok(())
}

pub(super) fn load_history(path: &PathBuf) -> Result<HistoryContext> {
    if let Ok(data) = fs::read_to_string(path.join("history.json")) {
        Ok(serde_json::from_str(&data)?)
    } else {
        Ok(HistoryContext::new())
    }
}
