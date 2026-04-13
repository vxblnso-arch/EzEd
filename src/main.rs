use clap::Parser;
use crossterm::{
    self,
    event::Event,
    event::KeyCode,
    execute,
    terminal::{Clear, ClearType},
};
use rpassword::read_password;
use std::fs::File;
use std::io::{self, Write, stdout};
use std::path::Path;

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
            buf: vec!['\0'; capacity],
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
    pub fn delete(&mut self) {
        if self.gap_start > 0 {
            self.gap_start -= 1;
        }
    }
}

fn redraw(stdout: &mut impl Write, buffer: &GapBuffer) {
    let before: String = buffer.buf[..buffer.gap_start].iter().collect();
    let after: String = buffer.buf[buffer.gap_end..]
        .iter()
        .filter(|&&c| c != '\0')
        .collect();

    let row = before.chars().filter(|&c| c == '\n').count() as u16;
    let column = before.chars().rev().take_while(|&c| c != '\n').count() as u16;

    execute!(stdout, crossterm::cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, Clear(ClearType::All)).unwrap();

    write!(stdout, "{}{}", before, after).unwrap();

    execute!(stdout, crossterm::cursor::MoveTo(column, row)).unwrap();
    stdout.flush().unwrap();
}

fn main() {
    let args = Cli::parse();
    let content = std::fs::read_to_string(&args.file).unwrap_or_default();
    print!("\x1B[2J\x1B[1;1H");

    let mut buffer = GapBuffer::new(content.len() + 1024);
    for c in content.chars() {
        buffer.insert(c);
    }

    let file_existed = Path::new(&args.file).exists();
    let mut file = File::create(&args.file).expect("Could not create file");
    crossterm::terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();
    redraw(&mut stdout, &buffer);
    execute!(stdout, crossterm::cursor::MoveTo(0, 0)).unwrap();
    io::stdout().flush().unwrap();
    loop {
        match crossterm::event::read().unwrap() {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char(c) => {
                    buffer.insert(c);
                    redraw(&mut stdout, &buffer);
                }
                KeyCode::Backspace => {
                    buffer.delete();
                    redraw(&mut stdout, &buffer);
                }
                KeyCode::Left => {
                    buffer.back();
                    redraw(&mut stdout, &buffer);
                }
                KeyCode::Enter => {
                    buffer.insert('\n');
                    redraw(&mut stdout, &buffer);
                }
                KeyCode::Right => {
                    buffer.forward();
                    redraw(&mut stdout, &buffer);
                }
                KeyCode::Esc => {
                    crossterm::terminal::disable_raw_mode().unwrap();

                    print!("\x1B[2J\x1B[1;1H");
                    println!(
                        "                            ________________________
                            |                       |
                            |     Save Changes?     |
                            |   [Y]es      (N)o     |
                            |                       |
                            |_______________________|"
                    );

                    let question = read_password().unwrap();
                    let answer = matches!(question.as_str(), "No" | "N" | "no");
                    if !answer {
                        let part1: String = buffer.buf[..buffer.gap_start].iter().collect();
                        let part2: String = buffer.buf[buffer.gap_end..].iter().collect();

                        file.write_all(part1.as_bytes())
                            .expect("could not write to file");
                        file.write_all(part2.as_bytes())
                            .expect("Could not write to file");

                        file.flush().expect("Could not sync file");
                        break;
                    } else {
                        if !file_existed {
                            std::fs::remove_file(&args.file)
                                .expect("Something went wrong when deleting the file.");
                        }
                        file.write_all(content.as_bytes())
                            .expect("Something weird happened.");
                        break;
                    }
                }
                _ => continue,
            },
            _ => continue,
        }
    }
}
