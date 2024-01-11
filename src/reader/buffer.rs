use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::BufRead,
    io::BufReader,
};

#[derive(Debug)]
pub enum BufferType {
    Local(LocalBuffer),
    Global(VecDeque<String>),
}

#[derive(Debug)]
pub struct LocalBuffer {
    pub file_path: String,
    pub buffer: VecDeque<String>,
    pub buffer_start_line: usize,
    pub buffer_end_line: usize,
}

impl LocalBuffer {
    // CAUTION: There is logic which assumes the current surrounded line is not shifted
    // out of the buffer, i.e. `SHIFT_AMOUNT` is expected to be less than `NEXT_LINES`
    pub const PREVIOUS_LINES: usize = 50;
    pub const NEXT_LINES: usize = 150;
    pub const MINIMUM_JUMP_LINES: usize = 30;
    pub const SHIFT_AMOUNT: usize = 100;

    pub fn empty(file_path: String) -> Self {
        Self {
            file_path,
            buffer: VecDeque::new(),
            buffer_start_line: 1,
            buffer_end_line: 0,
        }
    }

    pub fn new(file_path: String, start_line: usize, end_line: usize) -> Result<Self> {
        if end_line < start_line {
            panic!("The given end line is before the starting line.")
        }

        let line_count = (end_line - start_line) + 1;

        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);

        let mut buf = VecDeque::with_capacity(line_count);

        let mut lines = reader.lines().skip(start_line);

        // Return an empty buffer if the starting line is past EOF.
        match lines.next() {
            None => return Ok(Self::empty(file_path)),
            Some(l) => {
                buf.push_back(l?);
            }
        }

        for (i, line) in lines.enumerate() {
            if i + 1 >= line_count {
                break;
            }

            buf.push_back(line?);
        }

        // It is possible that the end line is beyond EOF, so calculate it manually.
        let buffer_end_line = (start_line + buf.len()) - 1;

        Ok(Self {
            file_path,
            buffer: buf,
            buffer_start_line: start_line,
            buffer_end_line,
        })
    }

    fn remove_lines_front(&mut self, lines: usize) {
        if self.buffer_end_line < self.buffer_start_line {
            return;
        }

        let line_count = (self.buffer_end_line - self.buffer_start_line) + 1;

        if line_count <= lines {
            self.buffer_start_line = self.buffer_end_line + 1;
            self.buffer = VecDeque::new();
        } else {
            self.buffer_start_line += lines;
            for _ in 0..lines {
                self.buffer.pop_front();
            }
        }
    }

    fn remove_lines_back(&mut self, lines: usize) {
        if self.buffer_end_line < self.buffer_start_line {
            return;
        }

        let line_count = (self.buffer_end_line - self.buffer_start_line) + 1;

        if line_count <= lines {
            if self.buffer_start_line == 0 {
                self.buffer_start_line = 1;
                self.buffer_end_line = 0;
            } else {
                self.buffer_end_line = self.buffer_start_line - 1;
            }
            self.buffer = VecDeque::new();
        } else {
            self.buffer_end_line -= lines;
            for _ in 0..lines {
                self.buffer.pop_back();
            }
        }
    }

    pub fn add_lines_back(&mut self, lines: usize) -> Result<bool> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);

        let mut added = false;
        for (i, line) in reader.lines().skip(self.buffer_end_line + 1).enumerate() {
            if i >= lines {
                break;
            }

            self.buffer.push_back(line?);
            self.buffer_end_line += 1;
            added = true;
        }

        Ok(added)
    }

    fn add_lines_front(&mut self, lines: usize) -> Result<bool> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);

        let buf_lines = if lines > self.buffer_start_line {
            // This is required for both clauses to be of the same type.
            #[allow(clippy::iter_skip_zero)]
            reader.lines().skip(0)
        } else {
            reader.lines().skip(self.buffer_start_line - lines)
        };

        let mut added = false;

        for (i, line) in buf_lines.enumerate() {
            if i >= lines {
                break;
            }

            self.buffer.push_front(line?);
            self.buffer_start_line -= 1;
            added = true;
        }

        Ok(added)
    }

    pub fn surround_line(&mut self, line: usize) -> Result<()> {
        let start = if line < Self::PREVIOUS_LINES {
            0
        } else {
            line - Self::PREVIOUS_LINES
        };

        let end = line + Self::NEXT_LINES;

        if start.abs_diff(self.buffer_start_line) > Self::MINIMUM_JUMP_LINES {
            let buffer = Self::new(self.file_path.clone(), start, end)?;
            let _ = std::mem::replace(self, buffer);
        } else {
            if self.buffer_start_line < start {
                self.remove_lines_front(start - self.buffer_start_line);
            } else {
                self.add_lines_front(self.buffer_start_line - start)?;
            }
            if self.buffer_end_line < end {
                self.add_lines_back(end - self.buffer_end_line)?;
            } else {
                self.remove_lines_back(self.buffer_end_line - end);
            }
        }

        Ok(())
    }

    /// Returns true if the buffer's end was increased.
    pub fn shift_down(&mut self) -> Result<bool> {
        self.remove_lines_front(Self::SHIFT_AMOUNT);

        self.add_lines_back(Self::SHIFT_AMOUNT)
    }

    /// Returns true if the buffer's beginning was increased.
    pub fn shift_up(&mut self) -> Result<bool> {
        self.remove_lines_back(Self::SHIFT_AMOUNT);

        self.add_lines_front(Self::SHIFT_AMOUNT)
    }

    pub fn is_empty(&self) -> bool {
        self.buffer_end_line < self.buffer_start_line
    }

    /// Converts a line in the file to the corresponding line in the buffer returning none if the file line is not currently in the buffer.
    pub fn get_buffer_line(&self, file_line: usize) -> Option<usize> {
        if file_line > self.buffer_end_line || file_line < self.buffer_start_line {
            return None;
        }

        Some(file_line - self.buffer_start_line)
    }
}

#[derive(Debug)]
pub struct BookPortion {
    pub buffer: BufferType,
    // Line, then char
    /// The line followed by the character at which to start the display.
    /// This is the line and character of the file, *NOT* the buffer.
    pub display_start: (usize, usize),
    pub display_end: (usize, usize),
    pub display_line_idxs: VecDeque<DisplayLineIndex>,
    pub display_copy: VecDeque<String>,
    pub display_next: NextDisplay,
    pub term_width: usize,
    pub term_height: usize,
    pub breaks: Linebreaks,
}

// Going backwards, it's possible for the way words are ordered to change.
// To prevent this, linebreaks must be inserted when going forwards, so they can be preserved going backwards.
// The key is the line, and the vector contains the character indexes at which to break lines.
#[derive(Debug)]
pub struct Linebreaks {
    breaks: HashMap<usize, Vec<usize>>,
}

impl Default for Linebreaks {
    fn default() -> Self {
        Self::new()
    }
}

impl Linebreaks {
    pub fn new() -> Self {
        Self {
            breaks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, line: usize, char: usize) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.breaks.entry(line) {
            e.insert(vec![char]);
        } else {
            self.breaks.get_mut(&line).unwrap().push(char);
        }
    }

    pub fn get_line(&self, line: usize) -> Option<&Vec<usize>> {
        self.breaks.get(&line)
    }

    /// Returns None if there is no break on the current line. Otherwise returns the index of the previous break.
    pub fn get_previous_break(&self, current: (usize, usize)) -> Option<usize> {
        let mut v = self.get_line(current.0)?.clone();
        v.sort_by(|a, b| b.partial_cmp(a).unwrap());
        v.into_iter().find(|&num| num + 1 < current.1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DisplayLineIndex {
    pub start_line: usize,
    pub start_char: usize,
    pub end_line: usize,
    pub end_char: usize,
}

impl DisplayLineIndex {
    pub fn get_start(&self) -> (usize, usize) {
        (self.start_line, self.start_char)
    }

    pub fn get_end(&self) -> (usize, usize) {
        (self.end_line, self.end_char)
    }

    pub fn from_pairs(start: (usize, usize), end: (usize, usize)) -> Self {
        Self {
            start_line: start.0,
            start_char: start.1,
            end_line: end.0,
            end_char: end.1,
        }
    }

    pub fn empty_line(line_no: usize) -> Self {
        Self {
            start_line: line_no,
            start_char: 0,
            end_line: line_no,
            end_char: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum NextDisplay {
    NoOp,
    ScrollDown(usize),
    ScrollUp(usize),
    Jump,
}

#[derive(Debug, Clone)]
pub struct LineData {
    pub line: String,
    pub line_no: usize,
    pub line_char: usize,
}

#[derive(Debug, Clone)]
pub enum LineReturnData {
    NonExistent,
    LineEnd,
    LineEmpty,
    LineExists(LineData),
    BackwardsLineExists {
        line: LineData,
        last_char_idx: usize,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BookProgressData {
    pub total_lines: usize,
    pub progress: BookProgress,
}

impl BookProgressData {
    pub fn new(total_lines: usize, progress: BookProgress) -> Self {
        Self {
            total_lines,
            progress,
        }
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let progress = BookProgress::Location((0, 0));

        let lines = BufReader::new(std::fs::File::open(path)?).lines().count();

        Ok(Self {
            total_lines: lines,
            progress,
        })
    }

    #[inline]
    pub fn get_pct(&self) -> f64 {
        match self.progress {
            BookProgress::Location((line, _)) => 100.0 * line as f64 / self.total_lines as f64,
            BookProgress::Finished => 100.0,
        }
    }

    #[inline]
    pub fn get_line(&self) -> usize {
        match self.progress {
            BookProgress::Location((line, _)) => line,
            BookProgress::Finished => self.total_lines - 1,
        }
    }

    #[inline]
    pub fn get_total_lines(&self) -> usize {
        self.total_lines
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum BookProgress {
    Location((usize, usize)),
    Finished,
}

impl BookProgress {
    pub const FINISHED: Self = BookProgress::Finished;
    pub const NONE: Self = BookProgress::Location((0, 0));
}

impl BookPortion {
    // Returns the current line and character of the display start.
    pub fn get_progress(&self) -> Result<BookProgressData> {
        match &self.buffer {
            BufferType::Global(lines) => {
                // Comparison of the last line's length fails for some reason, so use this for now...
                if self.display_end.0 == lines.len() - 1
                    && self
                        .display_end
                        .1
                        .abs_diff(lines[lines.len() - 1].len() - 1)
                        < 50
                {
                    Ok(BookProgressData {
                        total_lines: lines.len(),
                        progress: BookProgress::Finished,
                    })
                } else {
                    // return Ok(BookProgress::Location(self.display_start));
                    Ok(BookProgressData {
                        total_lines: lines.len(),
                        progress: BookProgress::Location(self.display_start),
                    })
                }
            }
            BufferType::Local(buffer) => {
                let file = buffer.file_path.clone();
                let lines = BufReader::new(std::fs::File::open(file)?).lines().count();
                // Not entirely accurate but close enough.
                if self.display_end.0 == lines - 1 {
                    Ok(BookProgressData {
                        total_lines: lines,
                        progress: BookProgress::Finished,
                    })
                } else {
                    Ok(BookProgressData {
                        total_lines: lines,
                        progress: BookProgress::Location(self.display_start),
                    })
                }
            }
        }
    }

    pub fn get_line(&mut self, start: (usize, usize)) -> Result<LineReturnData> {
        let idx = start.0;
        let line_exists = match &mut self.buffer {
            BufferType::Global(lines) => lines.len() > idx,
            BufferType::Local(buffer) => {
                let b_idx = buffer.get_buffer_line(idx);
                if b_idx.is_some() {
                    true
                } else {
                    buffer.surround_line(idx)?;
                    // If it's still not in the buffer then the line doesn't exist in the file.
                    buffer.get_buffer_line(idx).is_some()
                }
            }
        };

        if line_exists {
            let l = self.get_line_unchecked(start);
            Ok(l)
        } else {
            Ok(LineReturnData::NonExistent)
        }
    }

    /// Returns None if there are no characters remaining in the line, and the character index was larger than 0.
    pub fn get_line_unchecked(&mut self, start: (usize, usize)) -> LineReturnData {
        let idx = start.0;
        let full_line = match &self.buffer {
            BufferType::Global(lines) => lines[idx].clone(),
            BufferType::Local(buffer) => {
                let b_idx = buffer.get_buffer_line(idx).expect(
                    "Line was not in buffer. Maybe use the checked version of this function?",
                );
                buffer.buffer[b_idx].clone()
            }
        };

        let empty_check: bool = full_line
            .chars()
            .skip(start.1)
            .collect::<String>()
            .as_str()
            .trim()
            .is_empty();

        if empty_check {
            if start.1 == 0 {
                return LineReturnData::LineEmpty;
            }
            return LineReturnData::LineEnd;
        }

        let possible_chars: Vec<char> = full_line.chars().skip(start.1).collect();
        // If the possible chars <= term_width, then we just take the entire line.
        if possible_chars.len() <= self.term_width {
            return LineReturnData::LineExists(LineData {
                line: possible_chars.iter().collect(),
                line_no: start.0,
                line_char: start.1 + possible_chars.len() - 1,
            });
        }

        let mut display_chars: Vec<char> = possible_chars
            .into_iter()
            .take(self.term_width + 1)
            .collect();
        if !display_chars.contains(&' ') {
            // Removes the extra character
            display_chars.pop();
            // Adds space for a hyphen
            display_chars.pop();
            display_chars.push('-');
            let line_char = start.1 + display_chars.len() - 1;
            self.breaks.insert(start.0, line_char);
            return LineReturnData::LineExists(LineData {
                line: display_chars.into_iter().collect(),
                line_no: start.0,
                line_char,
            });
        }
        // All term_width + 1 characters are in the vector, or it would have returned earlier
        let l = display_chars.pop().unwrap().is_whitespace();
        if l {
            let line_char = start.1 + display_chars.len() - 1;
            self.breaks.insert(start.0, line_char);
            return LineReturnData::LineExists(LineData {
                line: display_chars.into_iter().collect(),
                line_no: start.0,
                line_char,
            });
        }

        loop {
            let l = display_chars.pop();
            let whitespace = match l {
                None => unreachable!(),
                Some(ch) => ch.is_whitespace(),
            };
            if whitespace {
                let line_char = start.1 + display_chars.len() - 1;
                self.breaks.insert(start.0, line_char);
                return LineReturnData::LineExists(LineData {
                    line: display_chars.into_iter().collect(),
                    line_no: start.0,
                    line_char,
                });
            }
        }
    }

    /// Returns None when the line does not exist in the buffer
    pub fn get_line_backwards(&mut self, end: (usize, usize)) -> Result<LineReturnData> {
        if end.1 == 0 {
            return Ok(LineReturnData::LineEnd);
        }
        let idx = end.0;
        let line_exists = match &mut self.buffer {
            BufferType::Global(lines) => lines.len() > idx,
            BufferType::Local(buffer) => {
                let b_idx = buffer.get_buffer_line(idx);
                if b_idx.is_some() {
                    true
                } else {
                    buffer.surround_line(idx)?;
                    // If it's still not in the buffer then the line doesn't exist in the file.
                    buffer.get_buffer_line(idx).is_some()
                }
            }
        };

        if line_exists {
            let l = self.get_line_backwards_unchecked(end);
            Ok(l)
        } else {
            Ok(LineReturnData::NonExistent)
        }
    }

    pub fn get_line_backwards_unchecked(&mut self, mut end: (usize, usize)) -> LineReturnData {
        if end.1 == 0 {
            return LineReturnData::LineEnd;
        }
        let idx = end.0;
        let full_line = match &self.buffer {
            BufferType::Global(lines) => lines[idx].clone(),
            BufferType::Local(buffer) => {
                let b_idx = buffer.get_buffer_line(idx).expect(
                    "Line was not in buffer. Maybe use the checked version of this function?",
                );
                buffer.buffer[b_idx].clone()
            }
        };
        if full_line.is_empty() {
            return LineReturnData::LineEmpty;
        }
        let last_char_idx = full_line.len() - 1;

        // Adjust to be in the valid range - large values correspond to the end of the line.
        if end.1 > last_char_idx {
            end.1 = last_char_idx;
        }

        let (line, shortened) = match self.breaks.get_previous_break(end) {
            None => (
                full_line.chars().take(end.1 + 1).collect::<Vec<char>>(),
                None,
            ),
            Some(v) => (
                full_line
                    .chars()
                    .take(end.1 + 1)
                    .skip(v + 2)
                    .collect::<Vec<char>>(),
                Some(v),
            ),
        };

        // let line: Vec<char> = full_line.chars().take(end.1 + 1).collect();
        let l_len = line.len();

        if l_len <= self.term_width {
            let line_char = match shortened {
                None => 0,
                Some(v) => v + 2,
            };
            return LineReturnData::BackwardsLineExists {
                line: LineData {
                    line: line.into_iter().collect(),
                    line_no: end.0,
                    line_char,
                },
                last_char_idx,
            };
        }

        let mut display_chars: Vec<char> =
            line.into_iter().rev().take(self.term_width + 1).collect();

        let has_whitespace = display_chars.contains(&' ');
        if !has_whitespace {
            let mut dis: Vec<char> = display_chars.into_iter().skip(2).rev().collect();
            dis.push('-');
            let line_char = (end.1 + 1) - dis.len();
            return LineReturnData::BackwardsLineExists {
                line: LineData {
                    line: dis.into_iter().collect(),
                    line_no: end.0,
                    line_char,
                },
                last_char_idx,
            };
        }

        let l = display_chars.pop().unwrap().is_whitespace();
        if l {
            let line_char = end.1 - display_chars.len();
            return LineReturnData::BackwardsLineExists {
                line: LineData {
                    line: display_chars.into_iter().rev().collect(),
                    line_no: end.0,
                    line_char,
                },
                last_char_idx,
            };
        }

        loop {
            let l = display_chars.pop();
            let whitespace = match l {
                None => {
                    // Not really sure what to do in this scenario - for now, just return an empty string
                    return LineReturnData::BackwardsLineExists {
                        line: LineData {
                            line: String::new(),
                            line_no: end.0,
                            line_char: end.1,
                        },
                        last_char_idx,
                    };
                }
                Some(ch) => ch.is_whitespace(),
            };
            if whitespace {
                let line_char = end.1 - display_chars.len();
                return LineReturnData::BackwardsLineExists {
                    line: LineData {
                        line: display_chars.into_iter().rev().collect(),
                        line_no: end.0,
                        line_char,
                    },
                    last_char_idx,
                };
            }
        }
    }

    pub fn with_buffer(buffer: BufferType, display_start: (usize, usize)) -> Self {
        Self {
            buffer,
            display_start,
            display_end: (0, 0),
            display_line_idxs: VecDeque::new(),
            display_copy: VecDeque::new(),
            display_next: NextDisplay::Jump,
            term_width: 0,
            term_height: 0,
            breaks: Linebreaks::new(),
        }
    }
    pub fn empty() -> Self {
        let buffer = VecDeque::from(vec![String::from("Loading... please wait...")]);
        Self {
            buffer: BufferType::Global(buffer),
            display_start: (0, 0),
            display_end: (0, 0),
            display_line_idxs: VecDeque::new(),
            display_copy: VecDeque::new(),
            display_next: NextDisplay::Jump,
            term_width: 0,
            term_height: 0,
            breaks: Linebreaks::new(),
        }
    }
}
