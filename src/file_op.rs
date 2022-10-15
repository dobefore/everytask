use crate::error::Result;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
pub(crate) fn open_as_append(path: &str) -> io::Result<std::fs::File> {
    let f = OpenOptions::new().create(true).append(true).open(path)?;
    Ok(f)
}
pub(crate) fn open_as_read(path: &str) -> io::Result<std::fs::File> {
    let f = fs::File::open(path)?;
    Ok(f)
}
pub fn append_line(file: &mut std::fs::File, strline: &str) -> io::Result<()> {
    writeln!(file, "{}", strline)?;
    Ok(())
}
pub fn read_lastline(path: &str) -> Result<Option<String>> {
    let bf = read_lines(path)?;
    match bf.last() {
        Some(s) => Ok(Some(s.trim().into())),
        None => Ok(None),
    }
}
/// read lines from file
///
/// filter lines whose contents are not empty out
pub fn read_lines(path: &str) -> Result<Vec<String>> {
    let v = fs::read_to_string(path)?
        .lines()
        .map(|f| f.trim().to_owned())
        .filter(|e| !e.is_empty())
        .collect();
    Ok(v)
}
pub fn file_empty(path: &str) -> io::Result<bool> {
    let bf = fs::read_to_string(path)?;
    if bf.trim().is_empty() {
        return Ok(true);
    }
    Ok(false)
}

pub fn clear_contents(path: &str) -> io::Result<()> {
    fs::write(path, "")?;
    Ok(())
}
/// return true if todo.txt empty
pub fn create_file(path: &str) -> io::Result<()> {
    if !Path::new(path).exists() {
        fs::File::create(path)?;
        println!("created file {}", path)
    }
    Ok(())
}
