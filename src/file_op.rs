use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

pub fn append_line_into_todo(strline: String) {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open("todo.txt")
        .unwrap();
    writeln!(f, "{}", strline).unwrap();
}
pub fn read_lastline_from_todo() -> String {
    let bf = fs::read_to_string("todo.txt").unwrap();
    let ll = bf.lines().last().unwrap().trim();
    ll.to_string()
}
pub fn read_alllines_from_todo() -> Vec<String> {
    let v = fs::read_to_string("todo.txt")
        .unwrap()
        .lines()
        .map(|f| f.to_owned())
        .collect();
    v
}
pub fn file_contents_empty(path: &str) -> bool {
    let bf = fs::read_to_string(path).unwrap();
    if bf.trim() == "" {
        return true;
    }
    false
}

pub fn clear_contents(path: &str) {
    fs::write(path, "").unwrap();
}
/// return true if todo.txt empty
pub fn create_ifnotexist(path: &str) {
    if !Path::new(path).exists() {
        fs::File::create(path).unwrap();
        println!("created file {}",path)
    }
}
