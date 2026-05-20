use clap::Parser;
use crossterm::{ self, event::{Event, KeyCode}, execute, terminal::{Clear, ClearType}, };
use rpassword::read_password; use std::fs::File; use std::io::{self, Write, stdout};
use std::path::Path; use std::process;

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
        // wtf somehow this works
        // Don't ask me why though
        loopn!(post2.chars().count() + 1, { self.back() });
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
        // Kinda goofy but it works so we ball
        loopn!(next2.chars().count() + 1, { self.forward() })
    }
}

fn redraw(stdout: &mut impl Write, buffer: &GapBuffer, mode: &str) {
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
    let before_display = before.replace("\n", "\r\n");
    let after_display = after.replace("\n", "\r\n");

    execute!(stdout, crossterm::cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, Clear(ClearType::All)).unwrap();

    write!(stdout, "{}{}{}", status_line, before_display, after_display).unwrap();

    execute!(stdout, crossterm::cursor::MoveTo(column, row + 2)).unwrap();
    stdout.flush().unwrap();
}

#[derive(PartialEq)]
enum EditingMinorMode {
    Yank,
    Delete,
    Normal,
}

#[derive(PartialEq)]
enum EditingMajorMode { // Sounds like i'm making an emacs clone now, The Better Editor.
    // Sadly I will clone vi
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

    let mut file = File::create(&args.file).expect("Could not create file");
    crossterm::terminal::enable_raw_mode().unwrap();
    let mut stdout = stdout();
    redraw(&mut stdout, &buffer, "NORMAL");
    execute!(stdout, crossterm::cursor::MoveTo(0, 0)).unwrap();
    io::stdout().flush().unwrap();

    let mut main_mode = EditingMajorMode::Normal(EditingMinorMode::Normal);
    
    
    loop {

        let current_mode = match main_mode {
            EditingMajorMode::Insert => "INSERT",
            EditingMajorMode::Normal(_) => "NORMAL",
            EditingMajorMode::Visual => "VISUAL",
            EditingMajorMode::Command => "COMMAND",
        };
        
        match main_mode {

            EditingMajorMode::Insert => {

                match crossterm::event::read().unwrap() {
                    Event::Key(key_event) => match key_event.code {

                        KeyCode::Char('(') => {
                            buffer.insert('(');
                            buffer.insert(')');
                            buffer.back();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Char('{') => {
                            buffer.insert('{');
                            buffer.insert('}');
                            buffer.back();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Char('[') => {
                            buffer.insert('[');
                            buffer.insert(']');
                            buffer.back();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Tab => {
                            buffer.insert('\t');
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Char('"') => {
                            buffer.insert('"');
                            buffer.insert('"');
                            buffer.back();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Char(c) => {
                            buffer.insert(c);
                            redraw(&mut stdout, &buffer, current_mode);
                        }

                        KeyCode::Backspace => {
                            buffer.delete();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Left => {
                            buffer.back();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Enter => {
                            buffer.insert('\n');
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Right => {
                            buffer.forward();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Up => {
                            buffer.up();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Down => {
                            buffer.down();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Esc => {
                            main_mode = EditingMajorMode::Normal(EditingMinorMode::Normal);
                            buffer.forward();
                            buffer.back();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
               
                        _ => continue,
                    },
                    _ => continue,
                }
            }

            EditingMajorMode::Normal(EditingMinorMode::Normal) => {
                crossterm::terminal::enable_raw_mode().unwrap();

                match crossterm::event::read().unwrap() {

                    Event::Key(key_event) => match key_event.code {
                        KeyCode::Left => {
                            buffer.back();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Right => {
                            buffer.forward();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Up => {
                            buffer.up();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Down => {
                            buffer.down();
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                        KeyCode::Char('i') => {
                            main_mode = EditingMajorMode::Insert;
                            redraw(&mut stdout, &buffer, current_mode);
                        }

                        KeyCode::Char(':') => {
                            main_mode = EditingMajorMode::Command;
                            redraw(&mut stdout, &buffer, current_mode);
                        }
                       

                    },

                    _ => continue,
                }
            }

            EditingMajorMode::Visual => {
                
            }
            EditingMajorMode::Command => {

                crossterm::terminal::disable_raw_mode().unwrap();
                
                match crossterm::event::read().unwrap() {


                    KeyCode::Char('q') => {

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
                }
            }

            _ =>  {}
        }
        
    }
}
