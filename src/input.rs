use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use termion::event::Key;
use std::io;
use termion::input::TermRead;

pub enum SignalType {
    Close,
    Go,
    BackSpace,
}

pub enum Data {
    Command(SignalType),
    Char(char),
}

#[derive(PartialEq)]
pub enum State {
  AddingText,
  AddingFile,
}

pub fn spawn_stdin_channel() -> Receiver<Data> {
    let (tx, rx) = mpsc::channel::<Data>();
    let mut input_mode = State::AddingText;

    thread::spawn(move || loop {
        let stdin = io::stdin();
        for c in stdin.keys() {
            match input_mode {
              State::AddingText =>
            match c.unwrap() {
                Key::Ctrl('c') => tx.send(Data::Command(SignalType::Close)).unwrap(),
                Key::Char('\n') => tx.send(Data::Char('\n')).unwrap(),
                Key::Alt('n') => {
                    input_mode = State::AddingFile;
                    tx.send(Data::Command(SignalType::Go)).unwrap();
                }
                Key::Char(x) => tx.send(Data::Char(x)).unwrap(),
                Key::Backspace => tx.send(Data::Command(SignalType::BackSpace)).unwrap(),
                _ => {}
            }
            _ => {
                match c.unwrap() {
                    Key::Char('\n') => {
                        tx.send(Data::Command(SignalType::Go)).unwrap();
                    }
                    Key::Backspace => tx.send(Data::Command(SignalType::BackSpace)).unwrap(),
                    Key::Char(x) => tx.send(Data::Char(x)).unwrap(),
                    Key::Ctrl('c') => tx.send(Data::Command(SignalType::Close)).unwrap(),
                    _ => {}
                }
            }
        }
        }
    });
    thread::sleep(std::time::Duration::from_millis(10));
    rx
}
