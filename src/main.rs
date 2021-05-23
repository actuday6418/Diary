use chrono::prelude::*;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
#[macro_use]
extern crate magic_crypt;

use magic_crypt::MagicCryptTrait;
use serde::{Deserialize, Serialize};
use std::thread;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, widgets::Clear, Terminal};

mod input;
mod popup;
mod ui;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub year: u64,
    pub month: u64,
    pub date: u64,
    pub content: Vec<u8>,
}

impl Entry {
    pub fn new(date: u64, month: u64, year: u64, content: Vec<u8>) -> Self {
        Self {
            year: year,
            month: month,
            date: date,
            content: content,
        }
    }
}

pub fn append_entry(content: String) {
    let date = Local::today();
    let key = new_magic_crypt!("passwordgoeshere!", 256);
    let content = key.encrypt_str_to_bytes(content);
    let content = Entry::new(
        date.day().try_into().unwrap(),
        date.month().try_into().unwrap(),
        date.year().try_into().unwrap(),
        content,
    );
    let content = serde_json::to_string(&content).unwrap();
    match OpenOptions::new()
        .write(true)
        .create(false)
        .open("database.json")
    {
        Ok(mut file) => {
            file.seek(SeekFrom::End(0)).unwrap();
            file.write(content.as_bytes()).unwrap();
        }
        Err(_) => {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open("database.json")
                .unwrap();
            file.seek(SeekFrom::End(0)).unwrap();
            file.write(content.as_bytes()).unwrap();
        }
    }
}

pub fn gen_page() {
    let mut html_page = "<html><head><style>
.header {
  padding: 5px;
  border-radius: 20px; 
  text-align: center;
  background: #1abc9c;
  color: white;
  font-size: 35px;
}
.entries {
  padding: 30px;
  border-radius: 20px;
  text-align: center;
  background: #1abcfa;
  color: white;
  font-size: 25px;
}
</style><body><div class=\"header\">
        <h2>My Diary<h2>
</div> <br><br> <div class=\"entries\">"
        .to_owned();
    match OpenOptions::new()
        .write(false)
        .read(true)
        .create(false)
        .open("database.json")
    {
        Ok(file) => {
            let key = new_magic_crypt!("passwordgoeshere!", 256);
            let entries = serde_json::Deserializer::from_reader(file)
                .into_iter::<serde_json::Value>()
                .map(|x| serde_json::from_value::<Entry>(x.unwrap()).unwrap())
                .collect::<Vec<Entry>>();
            entries.iter().for_each(|entry| {
                html_page.push_str(
                    format!(
                        "<br> <h1><b> {}/{}/{} </b></h1> <br> {} <br>",
                        entry.date,
                        entry.month,
                        entry.year,
                        std::str::from_utf8(
                            key.decrypt_bytes_to_bytes(&entry.content.clone())
                                .unwrap()
                                .as_slice()
                        )
                        .unwrap()
                    )
                    .as_str(),
                )
            });
            html_page.push_str("</div></body></html>");
            let mut html_file = std::fs::File::create("index.html").unwrap();
            html_file.write(html_page.as_bytes()).unwrap();
        }
        Err(e) => {
            println!("{}: looping", e);
            loop {}
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() == 2 && args[1] == String::from("--gen-page") {
        gen_page();
    } else {
        let mut state = input::State::AddingText;
        let mut curr_text = String::new();
        let stdout = std::io::stdout().into_raw_mode().unwrap();
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        let stdin_channel = input::spawn_stdin_channel();
        let mut update_ui = true;

        'main: loop {
            match stdin_channel.try_recv() {
                Ok(input::Data::Char(c)) => {
                    curr_text.pop();
                    curr_text.push(c);
                    curr_text.push('\u{2016}');
                    update_ui = true;
                }
                Ok(input::Data::Command(input::SignalType::Close)) => {
                    break 'main;
                }
                Ok(input::Data::Command(input::SignalType::Go)) => {
                    state = input::State::AddingFile;
                    append_entry(curr_text.clone());
                    curr_text.clear();
                    update_ui = true;
                }
                Ok(input::Data::Command(input::SignalType::BackSpace)) => {
                    curr_text.pop();
                    curr_text.pop();
                    curr_text.push('\u{2016}');
                    update_ui = true;
                }
                _ => {}
            }
            if update_ui {
                update_ui = false;
                terminal
                    .draw(|f| {
                        if state == input::State::AddingFile {
                            let popup = popup::centered_rect(90, 90, f.size());

                            let widget_input = ui::build_input(curr_text.clone());
                            f.render_widget(Clear, popup);
                            f.render_widget(widget_input, popup);
                        } else {
                            let widget_main = ui::build_main(curr_text.as_str());
                            f.render_widget(widget_main, f.size());
                        }
                    })
                    .unwrap();
            }
            thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}
// fn main() {
// append_entry();
// }
