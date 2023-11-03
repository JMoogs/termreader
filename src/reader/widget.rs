use std::collections::VecDeque;

use ratatui::{prelude::*, widgets::StatefulWidget};

use super::buffer::{BookPortion, DisplayLineIndex, LineReturnData, Linebreaks, NextDisplay};

#[derive(Default, Clone)]
pub struct BookText {
    lines: VecDeque<String>,
}

impl BookText {
    pub fn new() -> Self {
        Self {
            lines: VecDeque::new(),
        }
    }

    pub fn from_copy(portion: &BookPortion) -> Self {
        Self {
            lines: portion.display_copy.clone(),
        }
    }

    pub fn no_op(&mut self, portion: &BookPortion) {
        self.lines = portion.display_copy.clone();
    }

    fn jump(
        &mut self,
        portion: &mut BookPortion,
        term_width: u16,
        term_height: u16,
    ) -> Result<(), anyhow::Error> {
        let new = Self::from_portion(portion, term_width, term_height)?;
        let _ = std::mem::replace(self, new);
        Ok(())
    }

    fn scroll_down(
        &mut self,
        portion: &mut BookPortion,
        scrolls: usize,
    ) -> Result<(), anyhow::Error> {
        for _ in 0..scrolls {
            let n = Self::portion_scroll_down(portion)?;
            if !n.1 {
                let _ = std::mem::replace(self, Self::from_copy(portion));
                return Ok(());
            }
        }

        let _ = std::mem::replace(self, Self::from_copy(portion));
        Ok(())
    }

    fn scroll_up(
        &mut self,
        portion: &mut BookPortion,
        scrolls: usize,
    ) -> Result<(), anyhow::Error> {
        for _ in 0..scrolls {
            let n = Self::portion_scroll_up(portion)?;
            if !n.1 {
                let _ = std::mem::replace(self, Self::from_copy(portion));
                return Ok(());
            }
        }

        let _ = std::mem::replace(self, Self::from_copy(portion));
        Ok(())
    }

    /// Returns a boolean, representing whether or not EOF has been reached, i.e. true when EOF is reached.
    /// If EOF is reached, it returns the same section as before.
    fn portion_scroll_down(portion: &mut BookPortion) -> Result<(Self, bool), anyhow::Error> {
        // Add new line:
        let mut new_start = (portion.display_end.0, portion.display_end.1 + 2);
        let add_line_data = portion.get_line((portion.display_end.0, portion.display_end.1 + 2))?;
        match add_line_data {
            LineReturnData::NonExistent => return Ok((Self::from_copy(portion), true)),
            LineReturnData::LineEnd | LineReturnData::LineEmpty => {
                new_start = (new_start.0 + 1, 0);
                let data = portion.get_line(new_start)?;
                match data {
                    LineReturnData::NonExistent => return Ok((Self::from_copy(portion), true)),
                    LineReturnData::LineEmpty => {
                        portion.display_copy.push_back(String::new());
                        portion
                            .display_line_idxs
                            .push_back(DisplayLineIndex::empty_line(new_start.0));
                        portion.display_end = new_start;
                    }
                    LineReturnData::LineExists(l) => {
                        portion.display_copy.push_back(l.line);
                        portion
                            .display_line_idxs
                            .push_back(DisplayLineIndex::from_pairs(
                                new_start,
                                (l.line_no, l.line_char),
                            ));
                        portion.display_end = (l.line_no, l.line_char);
                    }
                    _ => unreachable!(),
                }
            }
            LineReturnData::LineExists(l) => {
                portion.display_copy.push_back(l.line);
                portion
                    .display_line_idxs
                    .push_back(DisplayLineIndex::from_pairs(
                        new_start,
                        (l.line_no, l.line_char),
                    ));
                portion.display_end = (l.line_no, l.line_char);
            }
            _ => unreachable!(),
        }

        // Remove line:
        portion.display_line_idxs.pop_front();
        portion.display_start = portion.display_line_idxs[0].get_start();
        portion.display_copy.pop_front();
        Ok((Self::from_copy(portion), false))
    }

    fn portion_scroll_up(portion: &mut BookPortion) -> Result<(Self, bool), anyhow::Error> {
        // Add new line:
        let mut new_start = if portion.display_start.1 != 0 {
            (portion.display_start.0, portion.display_start.1 - 1)
        } else {
            if portion.display_start.0 == 0 {
                return Ok((Self::from_copy(portion), true));
            }
            // Because of how the code works, it's safe to input a large number for the line end.
            // Avoiding using usize::MAX as the value may be incremented in the function.
            (portion.display_start.0 - 1, isize::MAX as usize)
        };

        let add_line_data = portion.get_line_backwards(new_start)?;

        match add_line_data {
            LineReturnData::NonExistent => return Ok((Self::from_copy(portion), true)),
            LineReturnData::LineEnd => {
                if new_start.0 == 0 {
                    return Ok((Self::from_copy(portion), true));
                }
                new_start = (new_start.0 - 1, isize::MAX as usize);
                let data = portion.get_line_backwards(new_start)?;
                match data {
                    LineReturnData::NonExistent => return Ok((Self::from_copy(portion), true)),
                    LineReturnData::LineEmpty => {
                        portion.display_copy.push_front(String::new());
                        portion
                            .display_line_idxs
                            .push_front(DisplayLineIndex::empty_line(new_start.0));
                        portion.display_start = (new_start.0, 0);
                    }
                    LineReturnData::BackwardsLineExists {
                        line,
                        last_char_idx,
                    } => {
                        portion.display_copy.push_front(line.line);
                        portion
                            .display_line_idxs
                            .push_front(DisplayLineIndex::from_pairs(
                                (line.line_no, line.line_char),
                                (line.line_no, last_char_idx),
                            ));
                        portion.display_start = (new_start.0, line.line_char);
                    }
                    // Reaches here
                    _ => unreachable!(),
                }
            }
            LineReturnData::LineEmpty => {
                portion.display_copy.push_front(String::new());
                portion
                    .display_line_idxs
                    .push_front(DisplayLineIndex::empty_line(new_start.0));
                portion.display_start = (new_start.0, 0);
            }
            LineReturnData::BackwardsLineExists {
                line,
                last_char_idx,
            } => {
                portion.display_copy.push_front(line.line);
                portion
                    .display_line_idxs
                    .push_front(DisplayLineIndex::from_pairs(
                        (line.line_no, line.line_char),
                        (line.line_no, last_char_idx),
                    ));
                portion.display_start = (line.line_no, line.line_char);
            }
            _ => unreachable!(),
        }

        // Remove line:
        portion.display_line_idxs.pop_back();
        portion.display_end =
            portion.display_line_idxs[portion.display_line_idxs.len() - 1].get_end();
        portion.display_copy.pop_back();
        Ok((Self::from_copy(portion), false))
    }

    fn from_portion(
        portion: &mut BookPortion,
        term_width: u16,
        term_height: u16,
    ) -> Result<BookText, anyhow::Error> {
        let term_width = term_width as usize;
        let term_height = term_height as usize;
        portion.term_width = term_width;
        portion.term_height = term_height;
        portion.breaks = Linebreaks::new();
        portion.display_line_idxs = VecDeque::new();
        portion.display_copy = VecDeque::new();

        let mut display_lines = VecDeque::with_capacity(term_height);

        let mut line_start = portion.display_start;

        let mut eof = false;

        loop {
            let line = portion.get_line(line_start)?;
            match line {
                LineReturnData::NonExistent => {
                    eof = true;
                    break;
                }
                LineReturnData::LineEnd => {
                    line_start = (line_start.0 + 1, 0);
                }
                LineReturnData::LineEmpty => {
                    portion
                        .display_line_idxs
                        .push_back(DisplayLineIndex::empty_line(line_start.0));
                    display_lines.push_back(String::new());
                    portion.display_end = line_start;
                    line_start = (line_start.0 + 1, 0);

                    if display_lines.len() == term_height {
                        break;
                    }
                }
                LineReturnData::LineExists(line) => {
                    portion
                        .display_line_idxs
                        .push_back(DisplayLineIndex::from_pairs(
                            line_start,
                            (line.line_no, line.line_char),
                        ));
                    display_lines.push_back(line.line);
                    portion.display_end = (line.line_no, line.line_char);
                    line_start = (line.line_no, line.line_char + 2);

                    if display_lines.len() == term_height {
                        break;
                    }
                }
                _ => unreachable!(),
            }
        }

        // If we hit eof, it's possible that more lines should be renedered before the first line.
        if eof {
            let mut line_end = portion.display_start;
            loop {
                let prev_line = portion.get_line_backwards(line_end)?;
                match prev_line {
                    LineReturnData::NonExistent => break,
                    LineReturnData::LineEnd => {
                        if line_end.0 == 0 {
                            break;
                        }
                        line_end = (line_end.0 - 1, isize::MAX as usize)
                    }
                    LineReturnData::LineEmpty => {
                        portion
                            .display_line_idxs
                            .push_front(DisplayLineIndex::empty_line(line_end.0));
                        display_lines.push_front(String::new());
                        portion.display_start = line_end;
                        if line_end.0 == 0 {
                            break;
                        }
                        line_end = (line_end.0 - 1, isize::MAX as usize);

                        if display_lines.len() == term_height {
                            break;
                        }
                    }
                    LineReturnData::BackwardsLineExists {
                        line,
                        last_char_idx,
                    } => {
                        portion
                            .display_line_idxs
                            .push_front(DisplayLineIndex::from_pairs(
                                (line.line_no, line.line_char),
                                (line_end.0, last_char_idx),
                            ));
                        display_lines.push_front(line.line);
                        portion.display_start = (line.line_no, line.line_char);

                        if line_end.0 == 0 {
                            break;
                        }
                        line_end = (line_end.0 - 1, isize::MAX as usize);

                        if display_lines.len() == term_height {
                            break;
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        portion.display_copy = display_lines;

        Ok(Self::from_copy(portion))
    }
}

impl StatefulWidget for BookText {
    type State = BookPortion;
    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state.display_next {
            NextDisplay::NoOp => self.no_op(state),
            NextDisplay::Jump => self.jump(state, area.width, area.height).unwrap(),
            NextDisplay::ScrollDown(scrolls) => self.scroll_down(state, scrolls).unwrap(),
            NextDisplay::ScrollUp(scrolls) => self.scroll_up(state, scrolls).unwrap(),
        }

        if state.term_width != area.width as usize || state.term_height != area.height as usize {
            self.jump(state, area.width, area.height).unwrap();
        }

        for (i, line) in self.lines.into_iter().enumerate() {
            buf.set_string(area.x, area.y + i as u16, line, Style::new());
        }
    }
}
