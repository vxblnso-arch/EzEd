use clap::Parser;
use crossterm::{
    self, ExecutableCommand,
    event::Event,
    event::KeyCode,
    execute,
    terminal::{Clear, ClearType},
};
use std::fs;
use std::io::{self, Read, Write, stdout};

#[derive(Parser)]
struct Cli {
    file: String,
}

struct GapBuffer {
    buf: Vec<char>,
    gap_start: usize,
    gap_end: usize,
}
impl GapBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buf: vec![' '; capacity], // Initialize with empty space
            gap_start: 0,
            gap_end: capacity,
        }
    }
    pub fn forward(&mut self) {
        if self.gap_end < self.buf.len() {
            self.buf.swap(self.gap_start, self.gap_end);
            self.gap_start += 1;
            self.gap_end += 1;
        }
    }
    pub fn back(&mut self) {
        if self.gap_start != 0 {
            self.gap_start -= 1;
            self.gap_end -= 1;
            self.buf.swap(self.gap_start, self.gap_end);
        }
    }
    pub fn insert(&mut self, c: char) {
        if self.gap_start < self.gap_end {
            self.buf[self.gap_start] = c;
            self.gap_start += 1;
        }
    }
    pub fn delete(&mut self) {}
}

fn main() {
    let args = Cli::parse();
    let content =
        std::fs::read_to_string(&args.file).expect("Unable to read file or file does not exist.");
    print!("\x1B[2J\x1B[1;1H");
    print!("{content}");

    let mut buffer = GapBuffer::new(content.len() + 1024);
    for c in content.chars() {
        buffer.insert(c);
    }

    crossterm::terminal::enable_raw_mode().unwrap();
    GapBuffer::new(1024);
    let mut stdout = stdout();
    loop {
        match crossterm::event::read().unwrap() {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char(c) => {
                    buffer.insert(c);
                    stdout.execute(crossterm::cursor::MoveRight(1)).unwrap();
                }
                KeyCode::Backspace => {
                    // Call a GapBuffer::delete()
                    stdout.execute(crossterm::cursor::MoveLeft(1)).unwrap();
                    execute!(stdout, Clear(ClearType::UntilNewLine)).unwrap();
                    todo!();
                }
                KeyCode::Left => {
                    buffer.back();
                    stdout.execute(crossterm::cursor::MoveLeft(1)).unwrap();
                }
                KeyCode::Enter => {
                    buffer.insert('\n');
                    stdout.execute(crossterm::cursor::MoveRight(1)).unwrap();
                }
                KeyCode::Right => {
                    buffer.forward();
                    stdout.execute(crossterm::cursor::MoveRight(1)).unwrap();
                }
                _ => continue,
            },
            _ => continue,
        }
    }
}
