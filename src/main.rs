mod matrix;

extern crate termion;

use std::io::{stdin, stdout, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;
use termion::color::{Blue, Fg};
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;
use termion::{clear, terminal_size};

fn main() {
    let stdout_raw = match stdout().into_raw_mode() {
        Ok(s) => s,
        Err(_) => panic!("please run rmatrix in a xterm compatible terminal"),
    };

    let size = terminal_size().expect("could not get the size of the terminal");

    // Open the alternate screen.
    let mut screen = AlternateScreen::from(stdout_raw);
    write!(
        screen,
        "{}{}{}{}",
        clear::All,
        Goto(1, 1),
        Fg(Blue),
        termion::cursor::Hide
    )
    .unwrap();
    screen.flush().unwrap();

    let mut matrix = matrix::Matrix::new((size.0 / 2) as usize, size.1 as usize);

    let rx = spawn_stdin_handle_thread();

    loop {
        write!(screen, "{}{}", Goto(1, 1), Fg(Blue)).unwrap();
        // stop on control-c or q
        if rx.try_recv().is_ok() {
            break;
        }

        matrix.tick();
        matrix.render(&mut screen);
        thread::sleep(Duration::from_millis(40));
        // TODO: add growing and shrinking of matrix
    }

    write!(screen, "{}", termion::cursor::Show).unwrap();
}

fn spawn_stdin_handle_thread() -> Receiver<()> {
    // Thread for handling stdin.
    let (tx, rx) = channel();
    thread::spawn(|| {
        let tx = tx; // move
        let stdin = stdin();
        for key in stdin.keys() {
            match key {
                Ok(Key::Char('q')) => tx.send(()).unwrap(),
                Ok(Key::Ctrl('c')) => tx.send(()).unwrap(),
                _ => {}
            }
        }
    });
    rx
}
