use ratatui::{style::Style, widgets::StatefulWidget};
use termreader_core::book::ChapterProgress;
use termreader_sources::chapter::Chapter;

use crate::trace_dbg;

/// A widget for allowing scrolling in the reader
pub struct GlobalReader {
    /// The contents of the reader
    pub(super) contents: GlobalReaderContents,
    /// The state of the reader
    pub(super) state: GlobalReaderState,
}

#[derive(Debug, Clone)]
pub struct GlobalReaderContents {
    // We want to use words as lines can be cut off halfway through.
    // At the same time, however, we want to be able to insert newlines at the locations they were in in the original text.
    // To do this, we only split on spaces.
    /// The words contained in the chapter text
    pub words: Vec<String>,
    /// The character length of the entire text
    pub text_length: usize,
}

#[derive(Debug, Clone)]
pub struct GlobalReaderState {
    /// The first word being displayed
    pub start_word_idx: usize,
    /// The last word being displayed
    pub end_word_idx: usize,
    /// Stores the previous start words, so that when scrolling back up, the text appears the same as it first did. If the window is reiszed, this can be removed
    pub prev_start_words: Vec<usize>,
    prev_term_width: u16,
    prev_term_height: u16,
}

impl GlobalReaderContents {
    /// Returns the set of lines to be displayed for a given width and height.
    pub fn get_display_lines(
        &self,
        state: &mut GlobalReaderState,
        term_width: u16,
        term_height: u16,
    ) -> Vec<String> {
        if (term_width, term_height) != (state.prev_term_width, state.prev_term_height) {
            state.prev_start_words = Vec::new();
            state.prev_term_width = term_width;
            state.prev_term_height = term_height
        }
        let mut words = self.words.iter().skip(state.start_word_idx).peekable();
        let mut word_count = 0;
        let mut lines = Vec::new();
        let mut current_line = String::new();
        while lines.len() < term_height as usize {
            let next_word = words.peek_mut();
            match next_word {
                Some(next) => {
                    // If we've come across a newline then special logic is required
                    if next == &&String::from("\n") {
                        lines.push(current_line);
                        current_line = String::new();
                        // Move to the next word
                        words.next();
                        // Newlines count as words
                        word_count += 1;
                        continue;
                    }
                    // FIXME: Split the word if it's longer than the terminal width
                    if current_line.is_empty() && next.len() > term_width as usize {
                        panic!("Encountered word longer than terminal width")
                    } else if current_line.is_empty() {
                        // No need for a space if a line is empty
                        // The unwrap is safe as the next word was peeked
                        current_line.push_str(&words.next().unwrap());
                        word_count += 1;
                    } else if current_line.len() + 1 + next.len() < term_width as usize {
                        // Line isn't empty so check if a space will also fit.
                        current_line.push(' ');
                        current_line.push_str(&words.next().unwrap());
                        word_count += 1;
                    } else {
                        // We have a word that doesn't fit, so move onto the next line.
                        lines.push(current_line);
                        current_line = String::new();
                        continue;
                    }
                }
                // If there's no next word we quit
                None => {
                    lines.push(current_line);
                    break;
                }
            }
        }
        state.end_word_idx = state.start_word_idx + word_count;
        lines
    }
}

impl GlobalReader {
    /// Creates a GlobalReader instance from a chapter
    pub fn from_chapter(chapter: &Chapter) -> Self {
        let text = chapter.get_contents();
        let text_length = text.len();
        let lines = text.lines();
        let mut words: Vec<String> = Vec::new();
        for line in lines {
            let mut line_words = line.split_whitespace().map(|w| w.to_string()).collect();
            words.append(&mut line_words);
            words.push(String::from("\n"));
        }
        // Get rid of any trailing whitespace
        while let Some(v) = words.last() {
            if v.trim().is_empty() {
                words.pop();
            } else {
                break;
            }
        }

        Self {
            contents: GlobalReaderContents { words, text_length },
            state: GlobalReaderState {
                start_word_idx: 0,
                end_word_idx: 0,
                prev_start_words: Vec::new(),
                prev_term_width: 0,
                prev_term_height: 0,
            },
        }
    }

    /// Scroll down 1 line
    pub fn scroll_down(&mut self, term_width: u16, term_height: u16) {
        let current_display =
            self.contents
                .get_display_lines(&mut self.state, term_width, term_height);

        // If theres no text then you can't scroll
        if current_display.is_empty() {
            return;
        }

        // Honestly idk how I cooked this up and why it works but it fixes the issue
        let offset = if current_display[0].is_empty() {
            1
        } else if current_display.get(1).is_some_and(|x| !x.is_empty()) {
            current_display[0].split_whitespace().count()
        } else {
            current_display[0].split_whitespace().count() + 1
        };

        trace_dbg!(offset);

        // Create a possible viable scroll
        let proposed_start = self.state.start_word_idx + offset;
        let mut proposed_state = self.state.clone();
        proposed_state.start_word_idx = proposed_start;
        proposed_state
            .prev_start_words
            .push(self.state.start_word_idx);

        // Create a new display with the proposed scroll
        let new_display =
            self.contents
                .get_display_lines(&mut proposed_state, term_width, term_height);

        // If the new display doesn't use the entire terminal
        // then there was no point in scrolling so do nothing
        if new_display.len() < term_height as usize {
            return;
        }

        // Update the state if the scroll was correct
        self.state = proposed_state;
    }

    /// Scroll up 1 line
    pub fn scroll_up(&mut self, term_width: u16, _term_height: u16) {
        // let current_display =
        //     self.contents
        //         .get_display_lines(&mut self.state, term_width, term_height);

        // If we have a previous value (terminal hasn't been resized since scrolling down)
        // we can just use that.
        if let Some(prev) = self.state.prev_start_words.pop() {
            self.state.start_word_idx = prev;
        } else {
            // If we don't have a previous value we need to go backwards
            // until we find a line start or fill the whole line

            // We also need to check if a previous line exists at all:
            if self.state.start_word_idx == 0 {
                return;
            }

            let mut prev_line_start = self.state.start_word_idx - 1;
            // If this is a '\n' it's safe to just go back another char
            if self.contents.words[prev_line_start] == "\n" {
                // I don't think this will ever happen but if it does just display from start
                if prev_line_start == 0 {
                    self.state.start_word_idx = 0;
                    return;
                }
                prev_line_start -= 1;
            }

            let mut prev_line = String::new();

            // FIXME: If a word is longer than terminal width we need to split it
            if self.contents.words[prev_line_start].len() > term_width as usize {
                panic!("Encountered word longer than terminal width")
            }
            // Fine to push as adding to the start/end are the same in this case
            prev_line.push_str(&self.contents.words[prev_line_start]);
            loop {
                // Stop if we ever reach the start
                if prev_line_start == 0 {
                    self.state.start_word_idx = 0;
                    return;
                }
                prev_line_start -= 1;

                let w = &self.contents.words[prev_line_start];
                // Stop if we reach a newline char
                if w == "\n" {
                    self.state.start_word_idx = prev_line_start + 1;
                    return;
                }
                // If we can add the word + a space then do it
                if prev_line.len() + w.len() + 1 <= term_width as usize {
                    // Adding to the start since we construct backwards
                    prev_line = w.to_owned() + " " + &prev_line
                } else {
                    // If it doesn't fit we stop
                    self.state.start_word_idx = prev_line_start + 1;
                    return;
                }
            }
        }
    }

    pub fn get_progress(&self) -> ChapterProgress {
        if self.state.end_word_idx >= self.contents.words.len() - 1 {
            return ChapterProgress::Finished;
        }
        ChapterProgress::Word((self.state.start_word_idx, self.state.end_word_idx))
    }

    pub fn get_total_words(&self) -> usize {
        self.contents.words.len()
    }
}

impl StatefulWidget for GlobalReaderContents {
    type State = GlobalReaderState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let display = self.get_display_lines(state, area.width, area.height);
        for (i, line) in display.into_iter().enumerate() {
            buf.set_string(area.x, area.y + i as u16, line, Style::new())
        }
    }
}
