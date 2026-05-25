use crossterm::{event::{Event, KeyCode},};

use std::io::Write;

use crate::{
    GapBuffer,
    redraw,
    EditingMajorMode,
    EditingMinorMode,
};

pub fn insert_mode_f(
    stdout: &mut impl Write,
    buffer: &mut GapBuffer,
    command: &str,
    main_mode: &mut EditingMajorMode,
    
) {
    redraw(stdout, buffer, "INSERT", command);
    crossterm::terminal::enable_raw_mode().unwrap();

    match crossterm::event::read().unwrap() {
        Event::Key(key_event) => match key_event.code {

            KeyCode::Char('(') => {
                buffer.insert('(');
                buffer.insert(')');
                buffer.back();
            }

            KeyCode::Char('{') => {
                buffer.insert('{');
                buffer.insert('}');
                buffer.back();
            }
            
            KeyCode::Char('[') => {
                buffer.insert('[');
                buffer.insert(']');
                buffer.back();
            }

            KeyCode::Tab => {
                buffer.insert('\t');
            }
            
            KeyCode::Char('"') => {
                buffer.insert('"');
                buffer.insert('"');
                buffer.back();
             }

            KeyCode::Char(c) => {
                buffer.insert(c);
            }

            KeyCode::Backspace => {
                buffer.delete();
            }

            KeyCode::Left => {
                buffer.back();
            }

            KeyCode::Enter => {
                buffer.insert('\n');
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

            KeyCode::Esc => {
                *main_mode = EditingMajorMode::Normal(EditingMinorMode::Normal);
            }

            _ => {},
        },

        _ => {},
    }
    redraw(stdout, buffer, "INSERT", command)
}
