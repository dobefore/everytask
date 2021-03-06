use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

pub fn append_line_into_file(path: &str, strline: String) {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    writeln!(f, "{}", strline).unwrap();
}
pub fn read_lastline_from_file(path: &str) -> Option<String> {
    let bf = fs::read_to_string(path).unwrap();
   match bf.lines().last() {
       Some(s)=>Some(s.trim().into()),
       None=>None
   }  
   
}
pub fn read_alllines_from_file(path: &str) -> Vec<String> {
    let v = fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|f| f.trim().to_owned())
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
        println!("created file {}", path)
    }
}
