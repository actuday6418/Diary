# Diary - Encrypted memories [![Latest version]][crates.io]
Diary is a TUI program written in Rust for GNU/Linux / \*BSD / Android  (It probably works on other platforms too, but who cares) that helps you write a digital journal from the comfort of a TUI. It ecrypts both text and file data, and can decrypt and build a rich HTML representation of your diary when required.

[Latest Version]: https://img.shields.io/crates/v/diary.svg
[crates.io]: https://crates.io/crates/diary

# Install
```cargo install diary```

# Usage 
```
USAGE:
    diary [FLAGS] [OPTIONS] --password <password>

FLAGS:
    -g, --generate-page    Assert this flag if you want the diary to built into an html file stored at $TEMPDIR.
    -h, --help             Prints help information
    -V, --version          Prints version information

OPTIONS:
    -d, --database <database>    This is the location of the database file. [default: .database]
    -p, --password <password>    This is the password to the database.
```

## To make an entry
1. Run the program with the password and optionally the database options. 
2. On the first screen, enter the day's diary entry. The date and day will be added by Diary. 
3. Esc saves the entry and exits. ```Ctrl+c``` exits the application without saving. ```Alt+n``` takes you to the next screen for adding files.
4. To add files, simply type in the file's location. After each file, hit ```Alt+n```. If the file doesn't exist, you get an alert.
5. Esc to save and exit, and ```Ctrl+c``` to exit without saving.
  
## To view the diary in HTML
1. Run the program with the ```-g``` (```--generate-page```) flag.
2. "index.html" and other required files are now saved to ```/tmp```. Open this file with a browser. 
  ```firefox /tmp/index.html```
 
# Important notes
1. Every entry (text and file data) is encrypted with AES-128 encrytpion. It is therefore practically impossible to access your diary without the password. So do NOT forget it.
2. Remember to clean up the decrypted files in /tmp if necessary.
  ```rm -rf /tmp/*```
3. Make sure you're writing to the right database. By default, diary writes to ```./.database```, but a custom database may be specified with the ```-d``` flag.
  
# Features
1. AES-128 encryption for text and file data.
2. TUI interface without sacrificing functionality or usability
3. Decrypts the whole database only when required (```-g``` flag), and into rich HTML5.

# Running

https://user-images.githubusercontent.com/56124831/119493025-5d5efd00-bd7d-11eb-93bf-3178bf35d821.mp4
