use crate::{error::TaskError, file_op::read_alllines_from_file};
use std::io::Write;
use std::{
    fs::{self, OpenOptions},
};

/// sort files about targets out ,which put finished/failed tasks at the start of file)
///
/// # example
///
/// 1.task 1 OK(something)
///
/// 2.task 2 not done
///
/// 3.task 3 BAD(give up)
///
/// after sort --->
///
/// 1.task 1 OK(something)
///
/// 2.task 3 BAD(give up)
///
/// 3.task 2 not done/marked
///
pub fn sort(files: Option<&[&'static str]>) -> Result<(), TaskError> {
    if let Some(ff) = files {
        for f in ff {
            // read file to vec
            // parse lines
            let (mk, notmk) = parse_lines(f);
            //    clear file contents
            mk.clear_file_contents()?;
            // write/append to file,begin with Marked
            mk.append_to_file()?;
            notmk.append_to_file()?;
        }
    }

    Ok(())
}
trait AppendFile {
    type Item;
    /// append stream lines to file
    fn append_to_file(&self) -> Result<Self::Item, TaskError>;
    fn clear_file_contents(&self) -> Result<(), TaskError>;
}
#[derive(Debug, Default)]
struct Marked {
    marks: Vec<String>,
    fname: &'static str,
}

impl Marked {
    fn new(marks: Vec<String>, fname: &'static str) -> Self {
        Self { marks, fname }
    }
}
impl AppendFile for Marked {
    type Item = u8;
    fn append_to_file(&self) -> Result<u8, TaskError> {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.fname)?;
            let mut n=0;
        for i in self.marks.clone() {
            n+=1;
            writeln!(f, "{}", i)?;
        }
        Ok(n)
    }
    fn clear_file_contents(&self) -> Result<(), TaskError> {
        fs::write(self.fname, "")?;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct NotMarked {
    notmarks: Vec<String>,
    fname: &'static str,
}

impl NotMarked {
    fn new(notmarks: Vec<String>, fname: &'static str) -> Self {
        Self { notmarks, fname }
    }
}

impl AppendFile for NotMarked {
    type Item = ();
    fn append_to_file(&self) -> Result<(), TaskError> {
        Ok(())
    }
    fn clear_file_contents(&self) -> Result<(), TaskError> {
        Ok(())
    }
}
/// loop vec to split it into two vec,one includes records that have been marked,the other
/// includes ones that haven't.
///
/// return lines without prefix e.g. 1. 2.
fn parse_lines(fname: &'static str) -> (Marked, NotMarked) {
    //  read file to string lines
    let lines = read_alllines_from_file(fname);
    let mut mk = vec![];
    let mut notmk = vec![];
    for l in lines {
        if l.contains("BAD") || l.contains("OK") {
            // remove prefix e.g. 1.
            // split string ,remove first,then concatenate them
            let mut sep = l.split('.').map(|e| e.to_string()).collect::<Vec<_>>();
            sep.remove(0);
            let line = sep.join(".");
            mk.push(line);
        } else {
            // remove prefix e.g. 1.
            // split string ,remove first,then concatenate them
            let mut sep = l.split('.').map(|e| e.to_string()).collect::<Vec<_>>();
            sep.remove(0);
            let line = sep.join(".");
            notmk.push(line);
        }
    }

    (Marked::new(mk, fname), NotMarked::new(notmk, fname))
}
