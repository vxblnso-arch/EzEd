use clap::Parser;
use crossterm::{
    self,
    execute,
    terminal::{Clear, ClearType},
};
use rpassword::read_password;
use std::io::{self, stdout, Write};
use std::path::Path;
use std::process;

macro_rules! loopn {
    ($n:expr, $body:block) => {
        for _ in 0..$n {
            $body
        }
    };
}

mod insert_mode;
mod normal_mode;
mod command_mode;

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

    pub fn up(&mut self) {
        let before: String = self.buf[..self.gap_start].iter().collect();
        let last_newline = before.rfind('\n');

        let _postl_return = match before.rfind('\n') {
            Some(i) => &before[i + 1..],
            None => &before,
        };

        let post2 = match last_newline {
            Some(i) => match before[..i].rfind('\n') {
                Some(j) => &before[j + 1..i],
                None => &before[..i],
            },
            None => "",
        };

        loopn!(post2.chars().count() + 1, {
            self.back()
        });
    }

    pub fn down(&mut self) {
        let after: String = self.buf[self.gap_end..]
            .iter()
            .filter(|&&c| c != '\0')
            .collect();

        let next_return = after.find('\n');

        let _pren_return = match after.find('\n') {
            Some(i) => &after[..i + 1],
            None => &after,
        };

        let next2 = match next_return {
            Some(i) => {
                let post = &after[i + 1..];

                match post.find('\n') {
                    Some(i2) => &post[..i2],
                    None => post,
                }
            }
            None => "",
        };

        loopn!(next2.chars().count() + 1, {
            self.forward()
        })
    }
}

fn redraw(
    stdout: &mut impl Write,
    buffer: &GapBuffer,
    mode: &str,
    current_command: &str,
) {

    let before: String = buffer.buf[..buffer.gap_start].iter().collect::<String>();
    let after: String = buffer.buf[buffer.gap_end..]
        .iter()
        .filter(|&&c| c != '\0')
        .collect::<String>();

    let row = before.chars().filter(|&c| c == '\n').count() as u16;
    let postl_return = match before.rfind('\n') {
        Some(i) => &before[i + 1..],
        None => &before,
    };

    let column = postl_return.chars().fold(0u16, |col, c| match c {
        '\t' => (col / 4 + 1) * 4,
        _ => col + 1,
    });

    let status_line = format!("-- {} --\r\n \r\n", mode);
    let command_line = format!(":{}\r\n", current_command);
    let before_display = before.replace("\n", "\r\n");
    let after_display = after.replace("\n", "\r\n");

    execute!(stdout, crossterm::cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, Clear(ClearType::All)).unwrap();

    write!(
        stdout,
        "{}{}{}{}",
        status_line,
        command_line,
        before_display,
        after_display
    ).unwrap();

    execute!(stdout, crossterm::cursor::MoveTo(column, row + 3)).unwrap();
    stdout.flush().unwrap();
}

#[derive(PartialEq)]
enum EditingMinorMode {
    Yank,
    Delete,
    Normal,
}

#[derive(PartialEq)]
enum EditingMajorMode {
    Insert,
    Normal(EditingMinorMode),
    Command,
    Visual, // This one's gonna be a pain to implement..
}

fn main() {
    // Just Here to Keep it Looking Clean...

    let args = Cli::parse();
    let content = std::fs::read_to_string(&args.file).unwrap_or_default();

    print!("\x1B[2J\x1B[1;1H");

    let mut buffer = GapBuffer::new(content.len() + 1048576);

    for c in content.chars() {
        buffer.insert(c);
    }

    let file_existed = Path::new(&args.file).exists();

    if !file_existed {
        print!("\x1B[2J\x1B[1;1H");

        print!(
            "                             ___________________________
                            |                           |
                            |    File does not exist.   |
                            |       Create it?          |
                            |     [Y]es      (N)o       |
                            |                           |
                            |___________________________|
" //I cant with this box bro
        );

        let question = read_password().unwrap();
        let answer = !matches!(question.as_str(), "No" | "N" | "no" | "n");

        if !answer {
            process::exit(1);
        }
    }

    crossterm::terminal::enable_raw_mode().unwrap();

    let mut stdout = stdout();

    execute!(stdout, crossterm::cursor::MoveTo(0, 0)).unwrap();
    io::stdout().flush().unwrap();

    let mut main_mode = EditingMajorMode::Normal(EditingMinorMode::Normal);

    let mut command = String::new();

    redraw(&mut stdout, &buffer, "NORMAL", &command);

    loop {

        let current_mode = match main_mode {
            EditingMajorMode::Insert => "INSERT",
            EditingMajorMode::Normal(_) => "NORMAL",
            EditingMajorMode::Visual => "VISUAL",
            EditingMajorMode::Command => "COMMAND",
        };

        match main_mode {

            EditingMajorMode::Insert => {
                insert_mode::insert_mode_f(
                    &mut stdout,
                    &mut buffer,
                    &command,
                    &mut main_mode,
                );
            },

            EditingMajorMode::Normal(_) => {
                normal_mode::normal_mode_f(
                    &mut stdout,
                    &mut buffer,
                    &mut command,
                    &mut main_mode,
                );
            },

            EditingMajorMode::Visual => {

            },

            EditingMajorMode::Command => {
                command_mode::command_mode_f(
                    &mut stdout,
                    &mut buffer,
                    &mut command,
                    &mut main_mode,
                    &args.file,
                    &content,
                    file_existed,
                );
            }

        }

    }


}

