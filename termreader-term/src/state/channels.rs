use crate::state::book_info::ID;
use anyhow::Result;
use std::sync::mpsc::{Receiver, Sender};
use termreader_sources::{
    chapter::Chapter,
    novel::{Novel, NovelPreview},
};

/// Contains all the possible requests that can be made through channels.
/// These are requests that need to happen synchronously
pub enum RequestData {
    /// The results of a search
    SearchResults(Result<Vec<NovelPreview>>),
    /// Info about a novel
    BookInfo(Result<Novel>),
    /// Info about a novel that is being viewed temporarily (i.e. the result of a search).
    /// The difference is in displaying history.
    BookInfoNoOpts(Result<Novel>),
    /// A chapter, that is being viewed temporarily (the novel isn't in the user's library).
    /// The difference is in displaying history.
    ChapterTemp((Result<Chapter>, usize)),
    /// A chapter
    Chapter((Result<Chapter>, usize)),
    /// Update info for a book
    UpdateInfo((ID, Result<Novel>)),
}

/// A struct to hold the sender and reciever
pub struct ChannelData {
    sender: Sender<RequestData>,
    pub reciever: Receiver<RequestData>,
    pub loading: bool,
}

impl ChannelData {
    /// Creates a new channel
    pub fn new() -> Self {
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
