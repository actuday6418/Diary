use chrono::prelude::*;
use std::convert::TryInto;
use std::io::{Seek, SeekFrom, Write};
use std::fs::OpenOptions;
#[macro_use]
extern crate magic_crypt;

use magic_crypt::MagicCryptTrait;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::TryRecvError;
use std::thread;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{ Constraint, Direction, Layout},
    widgets::Clear,
    Terminal,
};

mod ui;
mod input;
mod popup;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub year: u64,
    pub month: u64,
    pub date: u64,
    pub content: String,
}

impl Entry {
    pub fn new(date: u64, month: u64, year: u64, content: String) -> Self {
        Self {
            year: year,
            month: month,
            date: date,
            content: content,
        }
    }
}

pub fn append_entry() {
    let date = Local::today();
    let mut content = String::new();
    std::io::stdin().read_line(&mut content).unwrap();
    let key = new_magic_crypt!("passwordgoeshere!", 256);
    let content = key.encrypt_str_to_base64(content);
    let content = Entry::new(
        date.day().try_into().unwrap(),
        date.month().try_into().unwrap(),
        date.year().try_into().unwrap(),
        content,
    );
    let content = serde_json::to_string(&content).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .open("database.json")
        .unwrap();
    file.seek(SeekFrom::End(0)).unwrap();
    file.write(content.as_bytes()).unwrap();
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
    let file = OpenOptions::new()
        .write(false)
        .read(true)
        .create(false)
        .open("database.json")
        .unwrap();
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
                key.decrypt_base64_to_string(entry.content.clone()).unwrap()
            )
            .as_str(),
        )
    });
    html_page.push_str("</div></body></html>");
    let mut html_file = std::fs::File::create("index.html").unwrap();
    html_file.write(html_page.as_bytes()).unwrap();
}

fn main() {
    let mut state = input::State::AddingText;
    let mut curr_text = String::new();
    let stdout = std::io::stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    let widget_main =
                        ui::build_main("welkdm");
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(100)])
                        .split(f.size());
                    f.render_widget(widget_main, f.size());
                    if state == input::State::AddingText {
                        let popup = popup::centered_rect(90, 90, f.size());

                        let widget_input = ui::build_input(curr_text.clone());
                        f.render_widget(Clear, popup);
                        f.render_widget(widget_input, popup);
                    }
                })
                .unwrap();
  append_entry();
  gen_page();
}
