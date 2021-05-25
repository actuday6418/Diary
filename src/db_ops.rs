use chrono::prelude::*;
use std::convert::TryInto;
use serde::{Deserialize, Serialize};
use std::io::{ Seek, SeekFrom, Write};
use std::fs::OpenOptions;
use uuid;
use bincode;

use magic_crypt::MagicCryptTrait;
#[derive(Serialize, Deserialize)]
pub struct File {
    pub data: Vec<u8>,
    pub description: String,
    pub f_type: FileType,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Image,
    Audio,
    GenericFile,
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

pub fn append_entry(content: String, files: Vec<File>, file: &mut std::fs::File, password: &str) {
    let date = Local::today();
    let key = new_magic_crypt!(password, 128);
    let content = key.encrypt_str_to_bytes(content);
    let content = Entry::new(
        date.day().try_into().unwrap(),
        date.month().try_into().unwrap(),
        date.year().try_into().unwrap(),
        content,
        files,
    );
    let content = bincode::serialize(&content).unwrap();
    file.seek(SeekFrom::End(0)).unwrap();
    file.write(content.as_slice()).unwrap();
}

pub fn gen_page(password: &str, database_loc: &str) {
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
  transition: 0.3s;
}
.header:hover {
  color: white;
  box-shadow: 0px 0px 60px 1px #092E13;
}
.entries {
  padding: 30px;
  border-radius: 20px;
  text-align: center;
  background: #2BA67B;
  color: #D3FFED;
  font-size: 25px;
  box-shadow: 0px 0px 30px 1px #092E13;
  margin: 3%;
  transition: 0.3s;
}
.entries:hover { 
  color: white;
  box-shadow: 0px 0px 60px 1px #092E13;
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
  box-shadow: 0px 0px 60px 1px #092E13;
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
  box-shadow: 0px 0px 60px 1px #092E13;
}
audio {
  border: 5px;
  background-color: #D3FFED;
  border-style: solid;
  border-radius: 20px;
  border-color: #D3FFED;
  box-shadow: 0px 0px 30px 1px #092E13;
  width: 50%;
  transition: 0.3s;
}
audio:hover{
  border-color: white;
  width: 52%;
  box-shadow: 0px 0px 60px 1px #092E13;
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
        .open(database_loc)
    {
        Ok(mut file) => {
            let mut bf = std::io::BufReader::new(&mut file);
            let key = new_magic_crypt!(password, 128);
            let mut entries = Vec::new();
            let _: Vec<u8> = bincode::deserialize_from(&mut bf).unwrap();
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
                    file.write_all(key.decrypt_bytes_to_bytes(x.data.as_slice()).unwrap().as_slice()).unwrap();
                    if x.f_type == FileType::Image {
                        html_page.push_str(format!("<img src=\"{}\" class=\"image\"><br>", temp_dir.to_str().unwrap()).as_str());
                    } else if x.f_type == FileType::Audio {
                        html_page.push_str(format!("<audio controls src=\"{}\">Audio playback not supported by this browser! File: {}</audio><br>", temp_dir.to_str().unwrap(), x.description).as_str());
                    } else if x.f_type == FileType::GenericFile {
                        html_page.push_str(format!("<a href=\"{}\" style=\"text-decoration: none\" download=\"{}\"><p class = \"downloader\">Download: {}</p></a><br>", temp_dir.to_str().unwrap(),x.description, x.description).as_str());
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
