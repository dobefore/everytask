use crate::error::CustomError;
use crate::error::Result;
use crate::file_op::*;
use crate::parse_args::{Args, DBOption, RetrieveCommand};
use crate::pay::{write_pay, Pay};
use crate::sql_op::fetch_all;
use crate::sql_op::fetch_one;
use crate::sql_op::open_db;
use crate::task_handle::capture_task;
use crate::task_handle::move_task;
use crate::task_handle::roots_marked;
use crate::task_handle::roots_not_marked;
use crate::task_handle::RootTask;
use chrono::prelude::*;
use counter::Counter;
use std::collections::HashMap;
use std::fmt::Display;

use std::process;
use std::{
    fs,
    io::{self, Write},
};

pub(crate) static TODO_FILE: &str = "todo.txt";
pub(crate) static PAY_FILE: &str = "pay.txt";
pub(crate) static TASK_DB_FILE: &str = "task.db";
pub(crate) static SUMMARY_FILE: &str = "summary.txt";
pub(crate) static DATE_FILE: &str = "date.txt";
pub(crate) static TASK_CANDIDATES: &str = "task_candidates.txt";
pub(crate) static ALL_TASKS: &str = "all_tasks.txt";
pub(crate) static TODAY_TASKS: &str = "today_tasks.txt";
pub(crate) static TOMORROW_TASKS: &str = "tomorrow_tasks.txt";
pub(crate) static MONTH_FILE: &str = "months.txt";
pub(crate) static ALIDRIVE_CMD: &str = "./alidrive_uploader";
static NEXT: &str = "next";
// two aoms:
//1.every daynight copy files to local storage
// 2. end time >= begin time
trait ToHM {
    fn dur_to_hm(&self) -> String;
}
/// record today's date
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}
impl From<Date> for String {
    fn from(date: Date) -> Self {
        format!("{}-{}-{}", date.year, date.month, date.day)
    }
}
impl Into<Date> for String {
    fn into(self) -> Date {
        let v: Vec<u32> = self.split("-").map(|r| r.parse::<u32>().unwrap()).collect();
        Date {
            year: v.get(0).unwrap().to_owned(),
            month: v.get(1).unwrap().to_owned(),
            day: v.get(2).unwrap().to_owned(),
        }
    }
}
/// to_string()
impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.year, self.month, self.day)
    }
}
impl Date {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn weekday(&self) -> String {
        let local = Local::now();
        let y = local.year();
        let local_dt = Local.ymd(y, self.month, self.day);
        local_dt.weekday().to_string()
    }
    fn write_date() -> Result<()> {
        let dt: String = Date::today_date().into();
        fs::write(DATE_FILE, dt)?;
        Ok(())
    }

    pub(crate) fn today_date() -> Self {
        let local = Local::now();
        Self {
            year: local.year() as u32,
            month: local.month(),
            day: local.day(),
        }
    }

    /// load_date_from_file
    pub fn load_date_from_str(s: &str) -> Self {
        let ds = s.to_owned();
        ds.into()
    }
    /// load date from file,and set it as current date
    pub(crate) fn load_set_date(&mut self) -> Result<()> {
        let ds = fs::read_to_string(DATE_FILE)?.trim().to_owned();
        let dt: Date = if ds == "" {
            eprintln!("no date in file");
            let dt: String = Date::today_date().into();
            dt.into()
        } else {
            ds.into()
        };
        self.set_day(dt.day);
        self.set_month(dt.month);
        self.set_year(dt.year);
        Ok(())
    }

    pub fn set_day(&mut self, day: u32) {
        self.day = day;
    }

    pub fn set_month(&mut self, month: u32) {
        self.month = month;
    }

    pub fn set_year(&mut self, year: u32) {
        self.year = year;
    }
}
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct TimeStamp {
    hour: u32,
    minute: u32,
}
impl Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}
impl From<TimeStamp> for String {
    fn from(ts: TimeStamp) -> Self {
        format!("{:02}:{:02}", &ts.hour, &ts.minute)
    }
}
impl Into<TimeStamp> for String {
    fn into(self) -> TimeStamp {
        let v: Vec<u32> = self
            .split(":")
            .filter_map(|r| r.parse::<u32>().ok())
            .collect();
        TimeStamp {
            hour: v.get(0).unwrap().to_owned(),
            minute: v.get(1).unwrap().to_owned(),
        }
    }
}
impl TimeStamp {
    pub fn new() -> Self {
        Self::default()
    }

    /// convert u16 int to [`TimeStamp`]
    fn from_u16(ts: u16) -> Self {
        let h = ts / 60;
        let m = ts % 60;
        Self {
            hour: h.into(),
            minute: m.into(),
        }
    }

    fn return_ts(&self) -> String {
        self.to_owned().into()
    }
    fn current_ts() -> TimeStamp {
        let local = Local::now();
        // let l = format!("{:02}:{:02}", local.hour(), local.minute());
        let mut ts = TimeStamp::new();
        ts.set_hour(local.hour());
        ts.set_minute(local.minute());
        ts
    }

    pub fn set_hour(&mut self, hour: u32) {
        self.hour = hour;
    }

    pub fn set_minute(&mut self, minute: u32) {
        self.minute = minute;
    }
}
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct OneTaskTs {
    begin_ts: TimeStamp,
    end_ts: TimeStamp,
    one_task_duration: u32,
}
impl ToHM for OneTaskTs {
    fn dur_to_hm(&self) -> String {
        let pre_dur = self.one_task_duration;
        let h = pre_dur / 60;
        let m = pre_dur % 60;
        format!("{} h {} min", h, m)
    }
}
impl From<(String, String, u32)> for OneTaskTs {
    fn from(items: (String, String, u32)) -> Self {
        Self {
            begin_ts: items.0.into(),
            end_ts: items.1.into(),
            one_task_duration: items.2,
        }
    }
}
impl OneTaskTs {
    pub fn new() -> Self {
        Self::default()
    }

    fn load_dur_from_str(&mut self, dur: &str) {
        let i = dur.parse::<u32>().unwrap();
        self.one_task_duration = i
    }
    fn load_end_ts_from_str(&mut self, ts: &str) {
        self.end_ts = ts.to_owned().into()
    }
    fn load_begin_ts_from_str(&mut self, ts: &str) {
        self.begin_ts = ts.to_owned().into()
    }
    fn calcu_set_onetask_dur(&mut self) {
        if self.begin_ts != TimeStamp::default() || self.end_ts != TimeStamp::default() {
            let g_h = self.begin_ts.hour;
            let g_m = self.begin_ts.minute;
            let b_h = self.end_ts.hour;
            if b_h < g_h {
                panic!("begin_ts.hour < end_ts.hour,this should mot happen")
            }
            let mut b_m = self.end_ts.minute;
            //  h*60+min

            let (h_min, m_min) = if b_m < g_m {
                //  eg 26-30,this will borrow 1h from b_h
                b_m += 60;
                let m_min = b_m - g_m;
                let h = if b_h == g_h {
                    0
                } else {
                    // b_h>gh
                    b_h - 1 - g_h
                };
                let h_min = h * 60;
                (h_min, m_min)
            } else {
                //  eg 40-30 /30-30
                let m_min = b_m - g_m;
                let h = b_h - g_h;
                let h_min = h * 60;
                (h_min, m_min)
            };
            // sum up all mins
            let sum_min = m_min + h_min;
            // panic if one task dur > 20h
            if sum_min > 20 * 60 {
                panic!("one task dur incorrect {}", sum_min)
            }
            self.one_task_duration = sum_min;

            return;
        }
        println!("DayEndTs stay init state")
    }

    pub fn begin_ts(&self) -> TimeStamp {
        self.begin_ts
    }

    pub fn end_ts(&self) -> TimeStamp {
        self.end_ts
    }

    pub fn one_task_duration(&self) -> u32 {
        self.one_task_duration
    }

    pub fn set_begin_ts(&mut self, begin_ts: TimeStamp) {
        self.begin_ts = begin_ts;
    }

    pub fn set_end_ts(&mut self, end_ts: TimeStamp) {
        self.end_ts = end_ts;
    }

    pub fn set_one_task_duration(&mut self, one_task_duration: u32) {
        self.one_task_duration = one_task_duration;
    }
}
fn input_something<T: AsRef<str> + Display>(text_hint: T) -> io::Result<String> {
    print!("{}", text_hint);
    io::stdout().flush()?;
    let mut bf = String::new();
    io::stdin().read_line(&mut bf)?;

    Ok(bf.trim().to_owned())
}
/// print out fixed tasks from file line by line
fn display_task() -> Result<Vec<String>> {
    let v = fs::read_to_string(TASK_CANDIDATES)?;
    let mut n = 0;
    let mut vv = vec![];
    for l in v.lines() {
        n += 1;
        println!("{}  {}", n, &l);
        vv.push(l.to_owned());
    }
    Ok(vv)
}
/// set task of task_instance  by input num or plain task
fn match_task(task_str: String, fix_task_vec: Vec<String>) -> Result<String> {
    let vlen = fix_task_vec.len();
    let s = task_str.parse::<u32>();
    match s {
        Ok(x) => {
            if x > vlen as u32 {
                dbg!("out number range {}", x);
                return Err(CustomError::OutOfIndex("task index out of range".to_string()).into());
            }
            let n = x - 1;
            let item = fix_task_vec.get(n as usize).unwrap().to_owned();
            // I now am used to taking out pot from rice cooker.
            // if item == "午餐" {
            //     input_something("have you take out pot from rice cooker").unwrap();
            // }
            Ok(item)
        }
        Err(_e) => Ok(task_str),
    }
}
/// create file  if not exist once app starts
fn init_file() -> Result<()> {
    let files = [
        TODAY_TASKS,
        ALL_TASKS,
        TOMORROW_TASKS,
        TODO_FILE,
        TASK_CANDIDATES,
        MONTH_FILE,
        SUMMARY_FILE,
    ];
    for f in files {
        create_file(f)?;
    }
    Ok(())
}
/// ts:timestamp
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct DayEndTs {
    getup_ts: TimeStamp,
    bed_ts: TimeStamp,
    day_duration: u32,
}
impl From<(String, String, u32)> for DayEndTs {
    fn from(items: (String, String, u32)) -> Self {
        Self {
            getup_ts: items.0.into(),
            bed_ts: items.1.into(),
            day_duration: items.2,
        }
    }
}
impl ToHM for DayEndTs {
    fn dur_to_hm(&self) -> String {
        let pre_dur = self.day_duration;
        let h = pre_dur / 60;
        let m = pre_dur % 60;
        format!("{} h {} min", h, m)
    }
}
impl DayEndTs {
    pub fn new() -> Self {
        Self::default()
    }

    fn load_getup_ts(&mut self, hm_str: &str) {
        self.getup_ts = hm_str.to_owned().into()
    }
    fn load_bed_ts(&mut self, hm_str: &str) {
        self.bed_ts = hm_str.to_string().into()
    }

    fn calcu_set_day_dur(&mut self) {
        if self.getup_ts != TimeStamp::default() || self.bed_ts != TimeStamp::default() {
            let g_h = self.getup_ts.hour;
            let g_m = self.getup_ts.minute;
            let b_h = self.bed_ts.hour;
            let mut b_m = self.bed_ts.minute;

            //  h*60+min
            let (h_min, m_min) = if b_m < g_m {
                //  eg 26-30,this will borrow 1h from b_h
                b_m += 60;
                let m_min = b_m - g_m;
                let h = if b_h == g_h {
                    0
                } else {
                    // b_h>gh
                    b_h - 1 - g_h
                };
                let h_min = h * 60;
                (h_min, m_min)
            } else {
                //  eg 40-30 /30-30
                let m_min = b_m - g_m;
                let h = b_h - g_h;
                let h_min = h * 60;
                (h_min, m_min)
            };

            // sum up all mins
            let sum_min = m_min + h_min;
            self.day_duration = sum_min;
            return;
        }
        println!("DayEndTs stay init state")
    }

    pub fn getup_ts(&self) -> TimeStamp {
        self.getup_ts
    }

    pub fn bed_ts(&self) -> TimeStamp {
        self.bed_ts
    }

    pub fn day_duration(&self) -> u32 {
        self.day_duration
    }

    pub fn set_getup_ts(&mut self, getup_ts: TimeStamp) {
        self.getup_ts = getup_ts;
    }

    pub fn set_bed_ts(&mut self, bed_ts: TimeStamp) {
        self.bed_ts = bed_ts;
    }

    pub fn set_day_duration(&mut self, day_duration: u32) {
        self.day_duration = day_duration;
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{};;{};;{};;{};;{}",
            self.onetaskts.begin_ts,
            self.onetaskts.end_ts,
            self.onetaskts.one_task_duration,
            self.task,
            self.detail
        )
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Task {
    /// for db insert use
    pub index: u64,
    pub task: String,
    pub date: Date,
    pub dayendts: DayEndTs,
    pub onetaskts: OneTaskTs,
    pub detail: String,
}

///```
/// let  l=vec!["我","你","我","我","你"].iter().map(|e|e.to_string()).collect::<Vec<_>>();

/// let word="我";
/// let o=index_task_vec(&word, &l);
/// assert_eq!(vec![0,2,3],o);
/// ```
fn index_task_vec(task: &str, task_vec: &Vec<String>) -> Vec<u8> {
    let mut n = 0;
    let mut v = vec![];
    for i in task_vec {
        if task == i {
            v.push(n);
        }
        n += 1;
    }
    v
}
/// merge dupicated tasks into a single key of map
fn process_data(dt: TaskPercentage) -> HashMap<String, u16> {
    let t = dt.tasks;
    let dur = dt.duration;
    // remove duplicated tasks
    let char_counts = t.iter().collect::<Counter<_>>();
    let mut task_index_dict = HashMap::new();
    char_counts
        .keys()
        .into_iter()
        .map(|e| e.to_string())
        .for_each(|i| {
            let ind = index_task_vec(&i, &t);
            task_index_dict.insert(i, ind);
        });
    let mut final_dict = HashMap::new();
    task_index_dict.into_iter().for_each(|(k, v)| {
        let n = get_value_by_index(v, dur.clone());
        final_dict.insert(k, n);
    });

    final_dict
}
/// index=(1,4)
///
/// l=\[1,2,3,4,5]
///
/// output:
///
/// v1=l[1]=2
///
/// v2=l[4]=5
///
/// v1+v2=7
fn get_value_by_index(idx: Vec<u8>, values: Vec<u16>) -> u16 {
    let mut s = 0;
    for i in idx {
        s += values.get::<usize>(i.into()).unwrap()
    }
    s
}

#[derive(Debug, Default)]
struct TaskPercentage {
    sql_path: String,
    date: String,
    tasks: Vec<String>,
    duration: Vec<u16>,
}
impl TaskPercentage {
    fn new() -> Self {
        Self::default()
    }
    fn path(mut self, path: &str) -> Self {
        self.sql_path = path.into();
        self
    }
    fn latest_date(mut self) -> Result<Self> {
        let sql = "SELECT date_ FROM everydaytask ORDER BY id DESC";
        let c = open_db(&self.sql_path)?;
        if let Some(r) = fetch_one(&c, sql)? {
            self.date = r;
        } else {
            eprintln!("no date retrieved from db")
        }
        Ok(self)
    }

    fn latest_data(mut self) -> Result<Self> {
        let date = &self.date;
        let sql = format!(
            "SELECT one_task_dur,task FROM everydaytask WHERE date_='{}'",
            date
        );
        let c = open_db(&self.sql_path)?;
        let r = fetch_all(&sql, &c, Task::to_task)?;
        if let Some(res) = r {
            for i in res {
                self.tasks.push(i.task);
                self.duration.push(i.onetaskts.one_task_duration() as u16);
            }
        }

        Ok(self)
    }
}
fn get_percentage_rounded(x: f32, y: f32) -> String {
    let ret = (x * 100.0) / y;
    let rounded = ret.round();
    format!("{}%", rounded)
}

/// describe how much time a task occupied in a workday.
///
/// e.g. lunch:1h; a workday 16h.
///
///  taks percentage=1*60/16*60
fn task_percentage_rounded(sql_path: &str) -> Result<()> {
    let t = TaskPercentage::new()
        .path(sql_path)
        .latest_date()?
        .latest_data()?;
    let s = process_data(t);
    let all: u16 = s
        .values()
        .into_iter()
        .map(|e| e.to_owned())
        .collect::<Vec<_>>()
        .iter()
        .sum();
    for (k, v) in s {
        let percent = get_percentage_rounded(v.into(), all.into());
        let ts = TimeStamp::from_u16(v).to_string();
        println!(
            "task: {} || duration: {}min || percentage: {}",
            k, ts, percent
        );
    }
    Ok(())
}

impl Task {
    ///parse todo line to [`Task`] .
    fn str_to_task(line: &str) -> Result<Task> {
        let mut task = Task::new();
        let parts = line.split(";;").collect::<Vec<&str>>();
        task.onetaskts
            .set_begin_ts(parts.get(0).as_ref().unwrap().to_string().into());
        task.onetaskts
            .set_end_ts(parts.get(1).as_ref().unwrap().to_string().into());
        task.onetaskts
            .set_one_task_duration(parts.get(2).as_ref().unwrap().parse()?);

        task.set_task(parts.get(3).as_ref().unwrap());
        task.set_detail(parts.get(4).as_ref().unwrap());
        Ok(task)
    }
    fn parse_task_str(&mut self) -> Result<Vec<Task>> {
        let todo_lines = read_lines(TODO_FILE)?;
        let mut tasks = vec![];
        let dt = self.date;
        // calculate set dayts
        let first = todo_lines.first();
        let last = todo_lines.last();
        let f_parts = first.as_ref().unwrap().split(";;").collect::<Vec<&str>>();
        let l_parts = last.as_ref().unwrap().split(";;").collect::<Vec<&str>>();

        let mut dayendts = DayEndTs::new();
        dayendts.load_bed_ts(l_parts.get(1).as_ref().unwrap());
        dayendts.load_getup_ts(f_parts.get(0).as_ref().unwrap());
        dayendts.calcu_set_day_dur();

        for l in todo_lines {
            let parts = l.split(";;").collect::<Vec<&str>>();

            let mut onetaskts = OneTaskTs::new();
            onetaskts.load_begin_ts_from_str(parts.get(0).as_ref().unwrap());
            onetaskts.load_dur_from_str(parts.get(2).as_ref().unwrap());
            onetaskts.load_end_ts_from_str(parts.get(1).as_ref().unwrap());

            let mut task = Task::new();
            task.set_onetaskts(onetaskts);
            task.set_task(parts.get(3).as_ref().unwrap());
            task.set_detail(parts.get(4).as_ref().unwrap());
            task.set_date(dt);
            task.set_dayendts(dayendts.clone());
            tasks.push(task);
        }
        Ok(tasks)
    }
    fn load_set_date(&mut self) -> Result<()> {
        self.date.load_set_date()?;
        Ok(())
    }

    fn set_detail(&mut self, detail: &str) {
        self.detail = detail.to_owned();
    }
    fn set_task(&mut self, task: &str) {
        self.task = task.to_owned();
    }

    fn new() -> Task {
        Self::default()
    }
    fn set_index(&mut self, idx: u64) {
        self.index = idx;
    }
    /// summary timing_check
    /// make sure bed time is > 22:00,if so confirm 1 time.
    /// else confirm 3 times if < 22:00
    fn timing_check() -> Result<()> {
        let mut count = 0;
        let now = chrono::Local::now();
        loop {
            let r = input_something(format!(
                "you want to summary when you're at {}:{}? (y/n)",
                now.hour(),
                now.minute()
            ))?;
            if now.hour() >= 22 {
                if r.trim().to_ascii_uppercase() == "Y" {
                    break;
                } else {
                    std::process::abort();
                }
            } else {
                if r.trim().to_ascii_uppercase() != "Y" {
                    std::process::abort();
                } else if r.trim().to_ascii_uppercase() == "Y" {
                    count += 1;
                    if count == 3 {
                        break;
                    }
                }
                continue;
            }
        }
        Ok(())
    }
    fn write_summary(body_lines: Vec<String>, task: &Task) -> Result<()> {
        let sp = "==================";
        let mut f = open_as_append(SUMMARY_FILE)?;
        // preface
        let date_str = format!("date {} {}", task.date.to_string(), task.date.weekday());
        let day_time_note = format!(
            "the day is from {} to {} ,last {} ",
            task.dayendts.getup_ts().to_string(),
            task.dayendts.bed_ts().to_string(),
            task.dayendts.dur_to_hm()
        );
        append_line(&mut f, &date_str)?;
        append_line(&mut f, &day_time_note)?;
        // body
        append_line(&mut f, sp)?;
        for t in body_lines {
            append_line(&mut f, &t)?;
            append_line(&mut f, sp)?;
        }

        // wifi charge. Now I use wifi of bandwidth,so I don't need to charge.
        // loop {
        //     let anwser = input_something("WIFI charge? (Y/N)")?;
        //     let an = anwser.trim().to_uppercase();
        //     if an == "Y" {
        //         append_line(&mut f, "WIFI 是否充电：是")?;
        //         break;
        //     } else if an == "N" {
        //         append_line(&mut f, "WIFI 是否充电：否")?;
        //         break;
        //     } else {
        //         continue;
        //     }
        // }
        append_line(&mut f, "\n")?;
        Ok(())
    }
    fn summary(conn: &rusqlite::Connection) -> Result<()> {
        // summary timing check
        Task::timing_check()?;
        // automatically insert payments to db.
        write_pay()?;

        let mut task = Task::new();
        task.load_set_date()?;
        let tasks = task.parse_task_str()?;
        // fet last usn/index
        let sql = "SELECT id from everydaytask  ORDER BY id DESC";
        let mut last_idx: u64 = match fetch_one(&conn, sql)? {
            Some(ls) => ls,
            None => 0,
        };
        // insert records
        let sql = "INSERT INTO everydaytask VALUES (?,?,?,?,?,?,?,?,?,?)";
        let mut summary_lines = vec![];

        for mut t in tasks.clone() {
            //    summary body
            let s = format!(
                "task from {} to {} \ntask: {} detail: {} ,last {}",
                t.onetaskts.begin_ts,
                t.onetaskts.end_ts,
                t.task(),
                t.detail(),
                t.onetaskts.dur_to_hm()
            );
            summary_lines.push(s);
            println!("{}", t.date.to_string());
            last_idx += 1;
            t.set_index(last_idx);
            conn.execute(
                sql,
                rusqlite::params![
                    t.index(),
                    t.date.to_string(),
                    t.dayendts.getup_ts().to_string(),
                    t.dayendts.bed_ts().to_string(),
                    t.dayendts.day_duration(),
                    t.onetaskts.begin_ts().to_string(),
                    t.onetaskts.end_ts().to_string(),
                    t.onetaskts.one_task_duration(),
                    t.task(),
                    t.detail()
                ],
            )?;
        }
        // write today's tasks to summary file
        let first = tasks.first();
        Task::write_summary(summary_lines, first.as_ref().unwrap())?;
        task_percentage_rounded(TASK_DB_FILE)?;
        // move marked tasks from today task file to all task file
        move_task(TODAY_TASKS, ALL_TASKS)?;
        // clear file contents
        clear_contents(TODO_FILE)?;
        clear_contents(DATE_FILE)?;
        Ok(())
    }

    fn fetch_month(conn: &rusqlite::Connection, month: u8) -> Result<()> {
        let records = Pay::retrieve_month(&conn, month)?;
        if let Some(res) = records {
            res.iter().for_each(|e| println!("{}", e))
        }
        Ok(())
    }
    fn fetch_day(conn: &rusqlite::Connection, month: u8, day: u8) -> Result<()> {
        let mut date = Date::today_date();
        date.set_day(day as u32);
        date.set_month(month as u32);
        let records = Pay::retrieve_day(&conn, date)?;
        if let Some(res) = records {
            res.iter().for_each(|e| println!("{}", e))
        }

        Ok(())
    }
    fn to_task(row: &rusqlite::Row) -> rusqlite::Result<Task, rusqlite::Error> {
        let mut onetaskts = OneTaskTs::new();
        onetaskts.set_begin_ts(row.get::<_, String>(5)?.into());
        onetaskts.set_end_ts(row.get::<_, String>(6)?.into());
        onetaskts.set_one_task_duration(row.get::<_, u32>(7)?);

        let mut dayendts = DayEndTs::new();
        dayendts.set_getup_ts(row.get::<_, String>(2)?.into());
        dayendts.set_bed_ts(row.get::<_, String>(3)?.into());
        dayendts.set_day_duration(row.get::<_, u32>(4)?);

        Ok(Task {
            index: row.get(0)?,
            date: row.get::<_, String>(1)?.into(),
            dayendts,
            onetaskts,
            task: row.get(8)?,
            detail: row.get(9)?,
        })
    }
    /// first query field `task`,then query field `detail` if `task` return no result .
    fn search_task(conn: &rusqlite::Connection, words: &str) -> Result<()> {
        let sql = format!("SELECT * FROM everydaytask where task like '%{}%'", words);
        let tasks = fetch_all(&sql, conn, Task::to_task)?;
        if let Some(res) = tasks {
            res.iter().for_each(|e| println!("date: {},{}", e.date, e))
        }
        let sql = format!("SELECT * FROM everydaytask where detail like '%{}%'", words);
        let details = fetch_all(&sql, conn, Task::to_task)?;
        if let Some(res) = details {
            res.iter().for_each(|e| println!("date: {},{}", e.date, e))
        }

        Ok(())
    }

    /// re-order tasks in task files.
    /// # Rules:
    /// ## for [`TODAY_TASKS`] and MONTH_TASKS
    /// 1. root tasks: put below if all branch tasks in a root task are marked as either OK or BAD
    ///
    /// 2. branch tasks: put below if some of them are marked as either OK or BAD
    ///
    /// 3. prefix each root task with a correct serial number
    /// # example
    ///
    /// a task sample
    /// ```
    /// [
    /// root-task{
    /// 1. branch-task 1
    ///
    /// 2. branch-task 2
    /// }
    /// ]
    /// ```
    fn order() -> Result<()> {
        // month file,need to read filename from another file
        let mut files = read_lines(MONTH_FILE)?;
        files.push(TODAY_TASKS.into());
        for fpath in files {
            Task::order_logic(&fpath)?
        }
        Ok(())
    }
    fn order_logic(fpath: &str) -> Result<()> {
        // today task file，
        let ts = capture_task(fpath)?;
        let mut f = open_as_append(fpath)?;
        let roots = ts.iter().map(|e| RootTask::from_str(e)).collect::<Vec<_>>();
        let roots_marked = roots_marked(&roots);
        let roots_not_marked = roots_not_marked(&roots);
        clear_contents(fpath)?;
        for r in roots_not_marked {
            let s = r.to_string();
            f.write_all(format!("{}\n", s).as_bytes())?;
        }
        for r in roots_marked {
            let s = r.to_string();
            f.write_all(format!("{}\n", s).as_bytes())?;
        }
        Ok(())
    }
    /// query field `item` .
    fn search_pay(conn: &rusqlite::Connection, words: &str) -> Result<()> {
        let results = Pay::query_pay(conn, words)?;
        if let Some(res) = results {
            res.iter().for_each(|e| println!("{}", e))
        }
        Ok(())
    }

    /// upload files to cloud.
    ///
    /// ```
    ///  ./alidrive_uploader -c config.yaml . everydaytask/
    /// ```
    fn upload() {
        process::Command::new(ALIDRIVE_CMD)
            .args(&["-c", "config.yaml", ".", "everydaytask/"])
            .status()
            .expect("run alidrive_uploader error");
    }
    pub(crate) fn start(args: Args) -> Result<()> {
        let cmd = args.cmd;
        let conn = open_db(TASK_DB_FILE)?;
        conn.execute(include_str!("create.sql"), [])?;

        if let Some(rcmd) = cmd {
            match rcmd {
                RetrieveCommand::Pay {
                    mut retrieve,
                    insert,
                } => {
                    if insert {
                        // insert payments into db
                        write_pay()?;
                    }
                    if !retrieve.is_empty() {
                        if retrieve.len() == 1 {
                            // indicate only month is present
                            let month = retrieve.remove(0);
                            Task::fetch_month(&conn, month)?;
                        } else {
                            // only take first two elements from vec,eben if there are more than 2 elements
                            let month = retrieve.remove(0);
                            let day = retrieve.remove(0);
                            Task::fetch_day(&conn, month, day)?;
                        }
                    }
                }
                RetrieveCommand::Search { dbname, words } => match dbname {
                    DBOption::Pay => {
                        Task::search_pay(&conn, &words)?;
                    }
                    DBOption::Task => {
                        Task::search_task(&conn, &words)?;
                    }
                },
                RetrieveCommand::Task {
                    // mut retrieve,
                    summary,
                    order,
                    upload,
                } => {
                    // if !retrieve.is_empty() {
                    //     if retrieve.len() == 1 {
                    //         // indicate only month is present
                    //         let month = retrieve.remove(0);

                    //     } else {
                    //         // only take first two elements from vec,eben if there are more than 2 elements
                    //         let month = retrieve.remove(0);
                    //         let day = retrieve.remove(1);
                    //     }
                    // }
                    if summary {
                        Task::summary(&conn)?;
                    }
                    if order {
                        Task::order()?;
                    }
                    if upload {
                        Task::upload();
                    }
                }
            }
        } else {
            // run task process procedures
            Task::task_process()?;
        }

        Ok(())
    }
    fn task_detail_input(task: &mut Task, f: &mut std::fs::File) -> Result<()> {
        task.onetaskts.calcu_set_onetask_dur();
        // single task;
        // print fixed tasks ,waiting for being selected
        let v = display_task()?;
        let task_str = loop {
            let get_input = input_something(format!(
                "{}-{}做了什么？",
                task.onetaskts.begin_ts.return_ts(),
                task.onetaskts.end_ts.return_ts()
            ))?;
            if !get_input.trim().is_empty() {
                break get_input;
            }
        };
        let task_str = match_task(task_str, v)?;

        let mut detail = input_something("输入工作细节(l:use last line from next)：")?;
        if detail.is_empty() {
            detail = "0".to_owned();
        } else if detail.trim() == "l" {
            // read last line from next file
            let lines = read_lastline(NEXT)?;
            detail = if let Some(l) = lines {
                l
            } else {
                "0".to_owned()
            };
        }
        task.set_task(&task_str);
        task.set_detail(&detail);

        let line = task.to_string();
        append_line(f, &line)?;
        Ok(())
    }
    fn task_process() -> Result<()> {
        init_file()?;
        // re-oder task file
        Task::order()?;

        // init file : create files
        let mut task = Task::new();
        let mut f = open_as_append(TODO_FILE)?;
        // check if todo.txt is empty
        if file_empty(TODO_FILE)? {
            // write today's date
            if file_empty(DATE_FILE)? {
                Date::write_date()?;
            }
            task.load_set_date()?;

            // true   ,start from input getup_ts,this as begin_ts
            let get_getup_ts = input_something("when did you getup：")?;
            // task.dayendts.set_getup_ts(get_getup_ts.into());
            task.onetaskts.set_begin_ts(get_getup_ts.into());
        } else {
            // parse one task begin ts
            let todo_lines = read_lines(TODO_FILE)?;
            let last_line = todo_lines.last();
            let temp_task = Task::str_to_task(last_line.as_ref().unwrap())?;
            task.onetaskts.set_begin_ts(temp_task.onetaskts.end_ts());
        }

        // main work
        let cur_ts = TimeStamp::current_ts();
        // choose task_mode
        let get_task_mode = input_something("input task-mode s:single,m:multi：").unwrap();
        if get_task_mode == "s" {
            task.onetaskts.set_end_ts(cur_ts);
            // set one day dur
            Task::task_detail_input(&mut task, &mut f)?;
        } else if get_task_mode == "m" {
            task.onetaskts.set_end_ts(cur_ts.clone());
            let curts = task.onetaskts.end_ts.return_ts();
            // multi task
            // print fixed tasks ,waiting for being selected
            loop {
                // return begin ts
                println!("begin_ts is {}", task.onetaskts.begin_ts.return_ts());
                let end_ts = input_something(format!(
                    "input job end time(q:quit ; n: current_ts {})：",
                    curts
                ))?;

                if end_ts.chars().count() == 1 {
                    //   imply according to input n ,cur_ts as end_ts
                    //  set onetask_dur

                    if end_ts == "n" {
                        task.onetaskts.set_end_ts(cur_ts);
                    } else if end_ts == "q" {
                        break;
                    } else {
                        println!("wrong end ts commmand input");
                        return Err(CustomError::ValueNotFound(
                            "no such cmd in multi input".into(),
                        )
                        .into());
                    }
                } else {
                    task.onetaskts.set_end_ts(end_ts.into());
                }
                Task::task_detail_input(&mut task, &mut f)?;
                task.onetaskts.set_begin_ts(task.onetaskts.end_ts());
            }
        }

        Ok(())
    }

    pub fn set_onetaskts(&mut self, onetaskts: OneTaskTs) {
        self.onetaskts = onetaskts;
    }

    pub fn set_dayendts(&mut self, dayendts: DayEndTs) {
        self.dayendts = dayendts;
    }

    pub fn set_date(&mut self, date: Date) {
        self.date = date;
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn task(&self) -> &str {
        self.task.as_ref()
    }

    pub fn detail(&self) -> &str {
        self.detail.as_ref()
    }
}

#[test]
fn test_intlen() {
    let s = 00.to_string();
    println!("00 {}", s);
}
#[test]
fn test_week() {
    let weekday = Local::now().weekday().to_string();
    //    Sun
    println!("{}", weekday);
}
#[test]
fn test_yu() {
    let _s = 30;
    let r = 124 / 60;
    let x = 124 % 60;
    println!("{}{}", r, x);
}

/// \['f', 'o', 'e', 't', 'r', 'a', 'b']
///
/// \[1, 2, 1, 1, 1, 1, 1]
#[test]
fn teest_counter() {
    let v = vec!["a", "b", "a"];
    let char_counts = v.iter().collect::<Counter<_>>();
    let _counts_counts = char_counts.values().collect::<Counter<_>>();
    println!("{:?}", char_counts.keys());

    println!("{:?}", char_counts.values());
}

#[test]
fn test_mod() {
    // 2
    println!("{ }", 120 / 60);
    // 30
    println!("{ }", 30 % 60);
    println!("{ }", 120 % 60);
}
#[test]
fn test_percentage() {
    let sqlpath = "task.db";
    task_percentage_rounded(sqlpath).unwrap();
}

#[test]
fn test_index_task() {
    let l = vec!["我", "你", "我", "我", "你"]
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>();

    let word = "我";
    let o = index_task_vec(&word, &l);
    assert_eq!(vec![0, 2, 3], o);
}
