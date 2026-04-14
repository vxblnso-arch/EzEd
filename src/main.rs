use clap::Parser;
use crossterm::{
    self,
    event::{Event, KeyCode},
    execute,
    terminal::{Clear, ClearType},
};
use rpassword::read_password;
use std::fs::File;
use std::io::{self, Write, stdout};
use std::path::Path;
use std::process;

macro_rules! loopn {
    ($n:expr, $body:block) => {
        for _ in 0..$n {
            $body
        }
    };
}

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

        let postl_return = match before.rfind('\n') {
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

        loopn!(postl_return.chars().count() + 1, { self.back() });
        let target = postl_return.chars().count().min(post2.chars().count());
        loopn!(target, { self.forward() });
    }
    pub fn down(&mut self) {
        let after: String = self.buf[self.gap_end..]
            .iter()
            .filter(|&&c| c != '\0')
            .collect();

        let next_return = after.find('\n');
        let pren_return = match after.find('\n') {
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
        loopn!(pren_return.chars().count() + 1, { self.forward() });
        let target = pren_return.chars().count().min(next2.chars().count());
        loopn!(target, { self.back() })
    }
}

fn redraw(stdout: &mut impl Write, buffer: &GapBuffer) {
    let before: String = buffer.buf[..buffer.gap_start]
        .iter()
        .collect::<String>()
        .replace("\n", "\r\n");

    let after: String = buffer.buf[buffer.gap_end..]
        .iter()
        .filter(|&&c| c != '\0')
        .collect::<String>()
        .replace("\n", "\r\n");

    let row = before.chars().filter(|&c| c == '\n').count() as u16;
    let postl_return = match before.rfind('\n') {
        Some(i) => &before[i + 1..],
        None => &before,
    };
    let column = postl_return.chars().fold(0u16, |col, c| match c {
        '\t' => (col / 8 + 1) * 8,
        _ => col + 1,
    });

    execute!(stdout, crossterm::cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, Clear(ClearType::All)).unwrap();

    write!(stdout, "{}{}", before, after).unwrap();

    execute!(stdout, crossterm::cursor::MoveTo(column, row)).unwrap();
    stdout.flush().unwrap();
}

fn main() {
    // Just Here to Keep it Looking Clean...
    let args = Cli::parse();
    let content = std::fs::read_to_string(&args.file).unwrap_or_default();
    print!("\x1B[2J\x1B[1;1H");

    let mut buffer = GapBuffer::new(content.len() + 10240); // I think i can
    // just make this number huge and not deal with resize logic /s
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
                KeyCode::Up => {
                    buffer.up();
                    redraw(&mut stdout, &buffer);
                }
                KeyCode::Down => {
                    buffer.down();
                    redraw(&mut stdout, &buffer);
                }
                KeyCode::Tab => {
                    buffer.insert('\t');
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
                            |_______________________|" // Absolutely insane that THAT looks correct in the terminal.
                    );

                    let question = read_password().unwrap(); // Idk but it looks weird if
                    // I can see the 'yes' or 'no' input?
                    let answer = matches!(question.as_str(), "No" | "N" | "no" | "n");
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
                        file.write_all(content.as_bytes())
                            .expect("Something weird happened.");

                        if !file_existed {
                            std::fs::remove_file(&args.file)
                                .expect("Something went wrong when deleting the file.");
                            break;
                        }
                        break;
                    }
                }
                _ => continue,
            },
            _ => continue,
        }
    }
}
