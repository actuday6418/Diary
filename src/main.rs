use std::fs::OpenOptions;
use std::io::{Read};
#[macro_use]
extern crate magic_crypt;

use magic_crypt::MagicCryptTrait;
use std::thread;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, widgets::Clear, Terminal};

mod input;
mod popup;
mod ui;
mod db_ops;

use clap::{App, Arg};


fn main() {
    let matches = App::new("Diary")
      .arg(Arg::with_name("password").short("p").long("password").required(true).takes_value(true).help("This is the password to the database."))
      .arg(Arg::with_name("database").short("d").long("database").default_value(".database").takes_value(true).help("This is the location of the database file."))
      .arg(Arg::with_name("generate-page").long("generate-page").short("g").help("Assert this flag if you want the diary to built into an html file stored at $TEMPDIR.")).get_matches();
    let password = matches.value_of("password").unwrap();
    let database_loc = matches.value_of("database").unwrap();
    if matches.is_present("generate-page") {
        db_ops::gen_page(password, database_loc);
    } else {
        let mut state = input::State::AddingText;
        let mut buffer = String::new();
        buffer.push('\u{2016}');
        let mut content_text = String::from("Null");
        let mut curr_files: Vec<db_ops::File> = Vec::new();
        let stdout = std::io::stdout().into_raw_mode().unwrap();
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        let stdin_channel = input::spawn_stdin_channel();
        let mut update_ui = true;
        let key = new_magic_crypt!(password, 128);

        let mut file = match OpenOptions::new()
            .write(true)
            .read(true)
            .create(false)
            .open(database_loc)
        {
            Ok(mut file) => {
                let verifier: Vec<u8> = bincode::deserialize_from(&mut file).unwrap();
                let verifier = match key.decrypt_bytes_to_bytes(verifier.as_slice()){
                    Ok(string) => string,
                    Err(_) => panic!("Wrong password!"),
                };
                let verifier: String = std::str::from_utf8(verifier.as_slice()).unwrap().to_string();
                if verifier != String::from("917994806418") {
                    panic!("Incorrect password for this database! Exiting")
                } else {
                    file
                }
            }
            Err(_) => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(database_loc)
                    .unwrap();
                bincode::serialize_into(&mut file, &key.encrypt_str_to_bytes(String::from("917994806418"))).unwrap();
                file
            }
        };

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
                                let f_type: db_ops::FileType;
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
                                    f_type = db_ops::FileType::Image;
                                } else if buffer.ends_with(".mp3")
                                    || buffer.ends_with(".wav")
                                    || buffer.ends_with(".ogg")
                                    || buffer.ends_with(".webm")
                                {
                                    f_type = db_ops::FileType::Audio;
                                } else {
                                    f_type = db_ops::FileType::GenericFile;
                                }
                                file.read_to_end(&mut buff).unwrap();
                                let buff = key.encrypt_bytes_to_bytes(buff.as_slice());

                                curr_files.push(db_ops::File {
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
                        db_ops::append_entry(
                            content_text.clone(),
                            curr_files,
                            &mut file,
                            password,
                        );
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
