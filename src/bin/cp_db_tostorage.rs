use chrono::prelude::*;
use std::fs;
fn main() {
    let f = "task.db";
    let t = "../storage/shared/task.db";
    let local = Local::now();
    let weekday = local.weekday().to_string();
    if weekday == "Sun" {
        println!("copy task.db to phone storage");
        fs::copy(f, t).unwrap();
    }
}
