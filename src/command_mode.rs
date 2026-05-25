use crossterm::{event::{Event, KeyCode},};

use std::fs::File;
use std::io::Write;
use std::process;
use rpassword::read_password;

use crate::{
    GapBuffer,
    redraw,
    EditingMajorMode,
    EditingMinorMode,
};

pub fn command_mode_f(
    stdout: &mut impl Write,
    buffer: &mut GapBuffer,
    command: &mut String,
    main_mode: &mut EditingMajorMode,  
    args_file: &str,
    content: &str,
    file_existed: bool,
) {
    redraw(stdout, buffer, "COMMAND", command);

    crossterm::terminal::enable_raw_mode().unwrap();

    match crossterm::event::read().unwrap() {

        Event::Key(key_event) => match key_event.code {
            KeyCode::Char(c) => {
                command.push(c);
            }

            KeyCode::Backspace => {
                command.pop();
            }

            KeyCode::Enter => {
                match command.as_str() {
                    "q" => {
                        crossterm::terminal::disable_raw_mode().unwrap();

                        let mut file =
                            File::create(args_file).expect("Could not create file");

                        print!("\x1B[2J\x1B[1;1H");

                        println!(
                            "                            ________________________ 
                            |                       |
                            |   Save Changes?       |
                            |   [Y]es      (N)o     |
                            |                       |
                            |_______________________|"
                        );

                        let question = read_password().unwrap();

                        let answer = matches!(
                            question.as_str(),
                            "No" | "N" | "no" | "n" | "nope" | "nada"
                        );

                        if !answer {
                            let part1: String =
                                buffer.buf[..buffer.gap_start].iter().collect();

                            let part2: String =
                                buffer.buf[buffer.gap_end..].iter().collect();

                            file.write_all(part1.as_bytes())
                                .expect("could not write to file");

                            file.write_all(part2.as_bytes())
                                .expect("Could not write to file");

                            file.flush().expect("Could not sync file");

                            process::exit(0);
                        } else {
                            file.write_all(content.as_bytes())
                                .expect("Something weird happened.");

                            if !file_existed {
                                std::fs::remove_file(args_file)
                                    .expect(
                                        "Something went wrong when deleting the file.",
                                    );
                            }

                            process::exit(0);
                        }
                    }

                    "wq" => {
                        let mut file =
                            File::create(args_file).expect("Could not create file");

                        let part1: String =
                            buffer.buf[..buffer.gap_start].iter().collect();

                        let part2: String =
                            buffer.buf[buffer.gap_end..].iter().collect();

                        file.write_all(part1.as_bytes())
                            .expect("could not write to file");

                        file.write_all(part2.as_bytes())
                            .expect("Could not write to file");

                        file.flush().expect("Could not sync file");

                        process::exit(0);
                    }

                    _ => {},
                }
            }

            _ => {},
        },

        _ => {},
    }
    redraw(stdout, buffer, "COMMAND", command )
}
