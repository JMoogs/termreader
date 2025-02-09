use crate::{
    books_context::BooksContext,
    history::HistCtxSerialize,
    library::LibCtxSerialize,
    updates::{UpdatesContext, UpdatesCtxSerialize},
    HistoryContext, LibraryContext, TRError,
};
use std::{fs, path::PathBuf};

pub(super) fn store_library(library: LibraryContext, path: &PathBuf) -> Result<(), TRError> {
    let data = LibCtxSerialize::from_lib_ctx(library);
    let json = if cfg!(debug_assertions) {
        serde_json::to_string_pretty(&data)?
    } else {
        serde_json::to_string(&data)?
    };
    fs::write(path.join("lib.json"), json)?;
    Ok(())
}

pub(super) fn load_library(
    path: &PathBuf,
    books: &BooksContext,
) -> Result<LibraryContext, TRError> {
    if let Ok(data) = fs::read_to_string(path.join("lib.json")) {
        let ser_ver: LibCtxSerialize = serde_json::from_str(&data)?;
        Ok(ser_ver.to_lib_ctx(books))
    } else {
        Ok(LibraryContext::new())
    }
}

pub(super) fn store_history(history: HistoryContext, path: &PathBuf) -> Result<(), TRError> {
    let data = HistCtxSerialize::from_hist_ctx(history);
    let json = if cfg!(debug_assertions) {
        serde_json::to_string_pretty(&data)?
    } else {
        serde_json::to_string(&data)?
    };
    fs::write(path.join("history.json"), json)?;
    Ok(())
}

pub(super) fn load_history(
    path: &PathBuf,
    books: &BooksContext,
) -> Result<HistoryContext, TRError> {
    if let Ok(data) = fs::read_to_string(path.join("history.json")) {
        let ser_ver: HistCtxSerialize = serde_json::from_str(&data)?;
        Ok(ser_ver.to_hist_ctx(books))
    } else {
        Ok(HistoryContext::new())
    }
}

pub(super) fn store_updates(updates: UpdatesContext, path: &PathBuf) -> Result<(), TRError> {
    let data = UpdatesCtxSerialize::from_updates_ctx(updates);
    let json = if cfg!(debug_assertions) {
        serde_json::to_string_pretty(&data)?
    } else {
        serde_json::to_string(&data)?
    };
    fs::write(path.join("updates.json"), json)?;
    Ok(())
}

pub(super) fn load_updates(
    path: &PathBuf,
    books: &BooksContext,
) -> Result<UpdatesContext, TRError> {
    if let Ok(data) = fs::read_to_string(path.join("updates.json")) {
        let ser_ver: UpdatesCtxSerialize = serde_json::from_str(&data)?;
        Ok(ser_ver.to_updates_ctx(books))
    } else {
        Ok(UpdatesContext::new())
    }
}

pub(super) fn store_books(books: &BooksContext, path: &PathBuf) -> Result<(), TRError> {
    let json = if cfg!(debug_assertions) {
        serde_json::to_string_pretty(books)?
    } else {
        serde_json::to_string(books)?
    };
    fs::write(path.join("books.json"), json)?;
    Ok(())
}

pub(super) fn load_books(path: &PathBuf) -> Result<BooksContext, TRError> {
    if let Ok(data) = fs::read_to_string(path.join("books.json")) {
        Ok(serde_json::from_str(&data)?)
    } else {
        Ok(BooksContext::new())
    }
}
