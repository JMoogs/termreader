// This module is responsible for sending requests accross threads.
// This is used to allow the program to operate as normal while making a (blocking) web request.
// This is required as async is not used.
use anyhow::Result;
use std::sync::mpsc::{Receiver, Sender};
use termreader_core::id::ID;
use termreader_sources::{
    chapter::Chapter,
    novel::{Novel, NovelPreview},
};

/// Contains all the possible requests that can be made through channels.
/// These are requests that need to happen synchronously
pub enum RequestData {
    /// The results of a search
    SearchResults(Result<Vec<NovelPreview>>),
    /// Info about a novel.
    BookInfo((Result<Novel>, BookInfoDetails)),
    /// A chapter and it's number (TODO: Add chapter number details to chapters if possible)
    Chapter((ID, Result<Chapter>, usize)),
    // /// Update info for a book
    // Updated(Book),
}

pub enum BookInfoDetails {
    SourceNoOptions,
    SourceWithOptions,
    HistoryWithOptions,
}

pub struct ChannelData {
    /// The sender
    sender: Sender<RequestData>,
    /// A reciever that may be cloned to send data through
    pub reciever: Receiver<RequestData>,
    /// Whether or not a request has been made
    pub loading: bool,
}

impl ChannelData {
    /// Creates a new channel
    pub fn build() -> Self {
        let (sender, reciever) = std::sync::mpsc::channel();
        Self {
            sender,
            reciever,
            loading: false,
        }
    }

    /// Get a sender for the channel
    pub fn get_sender(&self) -> Sender<RequestData> {
        return self.sender.clone();
    }
}
