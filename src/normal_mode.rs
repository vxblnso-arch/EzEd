use crossterm::{event::{Event, KeyCode},};

use std::io::Write;

use crate::{
    GapBuffer,
    redraw,
    EditingMajorMode,
    EditingMinorMode,
};

pub fn normal_mode_f(
    stdout: &mut impl Write,
    buffer: &mut GapBuffer,
    command: &str,
    main_mode: &mut EditingMajorMode,
    
) {
    redraw(stdout, buffer, "NORMAL", command);

    crossterm::terminal::enable_raw_mode().unwrap();

    match crossterm::event::read().unwrap() {

        Event::Key(key_event) => match key_event.code {

            KeyCode::Left => {
                buffer.back();
            }

            KeyCode::Right => {
                buffer.forward();
            }

            KeyCode::Up => {
                buffer.up();
            }

            KeyCode::Down => {
                buffer.down();
            }

            KeyCode::Char('i') => {
                *main_mode = EditingMajorMode::Insert;
            }

            KeyCode::Char(':') => {
                *main_mode = EditingMajorMode::Command;
  
            }

            _ => {},

        },

        _ => {},
    }
    redraw(stdout, buffer, "NORMAL", command);
}
