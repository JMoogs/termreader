use crate::{HistoryContext, LibraryContext};
use anyhow::Result;
use std::fs;

pub(super) fn store_library(library: &LibraryContext) -> Result<()> {
    let json = serde_json::to_string_pretty(library)?;
    fs::write("books.json", json)?;
    Ok(())
}

pub(super) fn load_library() -> Result<LibraryContext> {
    if let Ok(data) = fs::read_to_string("books.json") {
        Ok(serde_json::from_str(&data)?)
    } else {
        Ok(LibraryContext::new())
    }
}

pub(super) fn store_history(history: &HistoryContext) -> Result<()> {
    let json = serde_json::to_string_pretty(history)?;
    fs::write("history.json", json)?;
    Ok(())
}

pub(super) fn load_history() -> Result<HistoryContext> {
    if let Ok(data) = fs::read_to_string("history.json") {
        Ok(serde_json::from_str(&data)?)
    } else {
        Ok(HistoryContext::new())
    }
}
