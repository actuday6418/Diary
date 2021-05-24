use chrono::prelude::*;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
#[macro_use]
extern crate magic_crypt;

use magic_crypt::MagicCryptTrait;
use serde::{Deserialize, Serialize};
use std::thread;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, widgets::Clear, Terminal};
use uuid;

mod input;
mod popup;
mod ui;
use bincode;

#[derive(Serialize, Deserialize)]
pub struct File {
    data: Vec<u8>,
    description: String,
    f_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub year: u64,
    pub month: u64,
    pub date: u64,
    pub content: Vec<u8>,
    pub files: Vec<File>,
}

impl Entry {
    pub fn new(date: u64, month: u64, year: u64, content: Vec<u8>, files: Vec<File>) -> Self {
        Self {
            year: year,
            month: month,
            date: date,
            content: content,
            files: files,
        }
    }
}

pub fn append_entry(content: String, files: Vec<File>) {
    let date = Local::today();
    let key = new_magic_crypt!("passwordgoeshere!", 256);
    let content = key.encrypt_str_to_bytes(content);
    let content = Entry::new(
        date.day().try_into().unwrap(),
        date.month().try_into().unwrap(),
        date.year().try_into().unwrap(),
        content,
        files,
    );
    let content = bincode::serialize(&content).unwrap();
    match OpenOptions::new()
        .write(true)
        .create(false)
        .open("database.json")
    {
        Ok(mut file) => {
            file.seek(SeekFrom::End(0)).unwrap();
            file.write(content.as_slice()).unwrap();
        }
        Err(_) => {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open("database.json")
                .unwrap();
            file.seek(SeekFrom::End(0)).unwrap();
            file.write(content.as_slice()).unwrap();
        }
    }
}

pub fn gen_page() {
    let mut html_page = "<html>
<meta charset=\"UTF-8\">
    <head><style>
.header {
  padding: 5px;
  border-radius: 20px; 
  text-align: center;
  background: #1abc9c;
  color: #D3FFED;
  font-size: 35px;
  box-shadow: 0px 0px 30px 1px #092E13;
}
.entries {
  padding: 30px;
  border-radius: 20px;
  text-align: center;
  background: #2BA67B;
  color: #D3FFED;
  font-size: 25px;
  box-shadow: 0px 0px 30px 1px #092E13;
}
.image {
  display: block;
  border: 5px;
  border-color: #D3FFED;
  border-style: solid;
  border-radius: 20px;
  margin-left: auto;
  margin-right: auto;
  width: 99%;
  box-shadow: 0px 0px 30px 1px #092E13;
  transition: 0.3s;
}
.image:hover {
  border-color: white;
}
.downloader {
  border: 5px;
  padding: 10px;
  color: #D3FFED;
  border-style: solid;
  border-color: #D3FFED;
  border-radius: 20px;
  margin-right: 200px;
  margin-left: 200px;
  transition: 0.3s;
  box-shadow: 0px 0px 30px 1px #092E13;
}
.downloader:hover {
  color: white;
  border-color: white;
  margin-left: 190px;
  margin-right: 190px;
}
body {
  background-color: #175942;
}
</style><body><div class=\"header\">
        <h2>My Diary<h2>
</div> <br><br>"
        .to_owned();
    match OpenOptions::new()
        .write(false)
        .read(true)
        .create(false)
        .open("database.json")
    {
        Ok(mut file) => {
            let mut bf = std::io::BufReader::new(&mut file);
            let key = new_magic_crypt!("passwordgoeshere!", 256);
            let mut entries = Vec::new();
            while let Ok(entry) = bincode::deserialize_from(&mut bf) {
                entries.push(entry);
            }
            entries.iter().for_each(|entry: &Entry| {
                html_page.push_str(
                    format!(
                        "<div class=\"entries\"><br> <h1><b> {}/{}/{} </b></h1> <br> {} <br><br><br><br>",
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
                );
                entry.files.iter().for_each(|x| {
                    let mut temp_dir = std::env::temp_dir();
                    let file_name = uuid::Uuid::new_v4();
                    temp_dir.push(file_name.to_string());
                    let mut file = std::fs::File::create(temp_dir.clone()).unwrap();
                    file.write_all(x.data.as_slice()).unwrap();
                    if x.f_type == String::from("image") {
                        html_page.push_str(format!("<img src=\"{}\" class=\"image\">", temp_dir.to_str().unwrap()).as_str());
                    } else {
                        html_page.push_str(format!("<a href=\"{}\" style=\"text-decoration: none\" download><p class = \"downloader\">Download: {}</p></a>", temp_dir.to_str().unwrap(), x.description).as_str());
                    }
                });
                html_page.push_str("</div><br>");
            });
            html_page.push_str("</body></html>");
            let mut html_file = std::env::temp_dir();
            html_file.push("index.html");
            let mut html_file = std::fs::File::create(html_file).unwrap();
            html_file.write(html_page.as_bytes()).unwrap();
        }
        Err(e) => {
            panic!("{}: No database found. Expected database.json", e);
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() == 2 && args[1] == String::from("--gen-page") {
        gen_page();
    } else {
        let mut state = input::State::AddingText;
        let mut buffer = String::new();
        buffer.push('\u{2016}');
        let mut content_text = String::from("Null");
        let mut curr_files: Vec<File> = Vec::new();
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
                    if state == input::State::AddingFile || state == input::State::AddingText {
                        buffer.pop();
                        buffer.push(c);
                        buffer.push('\u{2016}');
                        update_ui = true;
                    }
                }
                Ok(input::Data::Command(input::SignalType::Close)) => {
                    break 'main;
                }
                Ok(input::Data::Command(input::SignalType::Go)) => {
                    if state == input::State::AddingText {
                        buffer.pop();
                        state = input::State::AddingFile;
                        //append_entry(buffer.clone(), curr_files.clone());
                        content_text = buffer.clone();
                        buffer.clear();
                        update_ui = true;
                    } else if state == input::State::AddingFile {
                        buffer.pop();
                        match std::fs::File::open(buffer.clone()) {
                            Ok(mut file) => {
                                let mut buff = Vec::new();
                                let f_type: String;
                                let desc: String = buffer.split('/').last().unwrap().to_string();
                                if buffer.ends_with(".png")
                                    || buffer.ends_with(".apng")
                                    || buffer.ends_with(".gif")
                                    || buffer.ends_with(".jpeg")
                                    || buffer.ends_with(".jpg")
                                    || buffer.ends_with(".svg")
                                    || buffer.ends_with(".webp")
                                    || buffer.ends_with(".avif")
                                {
                                    f_type = String::from("image");
                                } else {
                                    f_type = String::from("file");
                                }
                                file.read_to_end(&mut buff).unwrap();
                                curr_files.push(File {
                                    data: buff,
                                    description: desc,
                                    f_type: f_type,
                                });
                                buffer.clear();
                                update_ui = true;
                            }
                            Err(_) => {
                                buffer = String::from(format!("{}: File not found!", buffer));
                                state = input::State::Popup;
                                update_ui = true;
                            }
                        }
                    }
                }
                Ok(input::Data::Command(input::SignalType::BackSpace)) => {
                    if state == input::State::AddingFile || state == input::State::AddingText {
                        buffer.pop();
                        buffer.pop();
                        buffer.push('\u{2016}');
                        update_ui = true;
                    }
                }
                Ok(input::Data::Command(input::SignalType::Cancel)) => {
                    if state == input::State::AddingFile {
                        append_entry(content_text.clone(), curr_files);
                        break 'main;
                    } else if state == input::State::Popup {
                        state = input::State::AddingFile;
                        update_ui = true;
                        buffer.clear();
                    }
                }
                _ => {}
            }
            if update_ui {
                update_ui = false;
                terminal
                    .draw(|f| match state {
                        input::State::AddingText => {
                            let widget_main = ui::build_main(buffer.as_str());
                            f.render_widget(widget_main, f.size());
                        }
                        input::State::AddingFile => {
                            let widget = ui::build_file_input(buffer.as_str());
                            f.render_widget(widget, f.size());
                        }
                        input::State::Popup => {
                            let popup = popup::centered_rect(40, 10, f.size());

                            let widget_input = ui::build_message(buffer.as_str());
                            f.render_widget(Clear, popup);
                            f.render_widget(widget_input, popup);
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
