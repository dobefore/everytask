use crate::file_op::*;
use crate::sql_op::Sqlite;
use chrono::prelude::*;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use std::{
    env, fs,
    io::{self, prelude::*},
};
/// record today's date
#[derive(Debug, Default, PartialEq, Clone, Copy)]
struct Date {
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
impl Date {
    fn write_date() {
        let dt: String = Date::today_date().into();
        let path = "date.txt";
        fs::write(path, dt).unwrap()
    }

    fn today_date() -> Self {
        let local = Local::now();
        Self {
            year: local.year() as u32,
            month: local.month(),
            day: local.day(),
        }
    }
    fn load_date_from_file(&mut self) {
        let fp = "date.txt";
        let ds = fs::read_to_string(fp).unwrap().trim().to_owned();
        if ds == "" {
            println!("no date in file");
            return;
        }

        *self = ds.into();
    }
}
#[derive(Debug, Default, PartialEq, Clone, Copy)]
struct TimeStamp {
    hour: u32,
    minute: u32,
}
impl From<TimeStamp> for String {
    fn from(ts: TimeStamp) -> Self {
        let h_str = if ts.hour < 10 {
            format!("0{}", &ts.hour)
        } else if ts.hour == 10 {
            10.to_string()
        } else {
            format!("{}", &ts.hour)
        };
        let m_str = if ts.minute < 10 {
            format!("0{}", &ts.minute)
        } else if ts.minute == 10 {
            10.to_string()
        } else {
            format!("{}", &ts.minute)
        };
        format!("{}:{}", h_str, m_str)
    }
}
impl Into<TimeStamp> for &String {
    fn into(self) -> TimeStamp {
        let v: Vec<u32> = self.split(":").map(|r| r.parse::<u32>().unwrap()).collect();
        TimeStamp {
            hour: v.get(0).unwrap().to_owned(),
            minute: v.get(1).unwrap().to_owned(),
        }
    }
}
impl TimeStamp {
    fn return_ts(&self) -> String {
        self.to_owned().into()
    }
    fn current_ts() -> String {
        let local = Local::now();
        format!("{}:{}", local.hour(), local.minute())
    }
}
#[derive(Debug, Default, PartialEq, Clone, Copy)]
struct OneTaskTs {
    begin_ts: TimeStamp,
    end_ts: TimeStamp,
    one_task_duration: u32,
}

impl OneTaskTs {
    fn set_onetask_dur(&mut self, dur: &String) {
        let i = dur.parse::<u32>().unwrap();
        self.one_task_duration = i
    }
    fn set_end_ts(&mut self, ts: &String) {
        self.end_ts = ts.into()
    }
    fn set_begin_ts(&mut self, ts: &String) {
        self.begin_ts = ts.into()
    }
    fn calcu_set_onetask_dur(&mut self) {
        if self.begin_ts != TimeStamp::default() || self.end_ts != TimeStamp::default() {
            let g_h = self.begin_ts.hour;
            let g_m = self.begin_ts.minute;
            let b_h = self.end_ts.hour;
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
            self.one_task_duration = sum_min;
            return;
        }
        println!("DayEndTs stay init state")
    }
    fn return_ts_dur_line(&self) -> String {
        format!(
            "{};;{};;{}",
            self.begin_ts.return_ts(),
            self.end_ts.return_ts(),
            self.one_task_duration.to_string()
        )
    }
}
fn input_something(text_hint: &str) -> io::Result<String> {
    print!("{}", text_hint);
    io::stdout().flush().unwrap();
    let mut bf = String::new();
    io::stdin().read_line(&mut bf)?;

    Ok(bf.trim().to_owned())
}
/// print out fixed tasks from file line by line
fn display_task() -> Vec<String> {
    create_ifnotexist("fix_task.txt");
    let v = fs::read_to_string("fix_task.txt").unwrap();
    let mut n = 0;
    let mut vv = vec![];
    for l in v.lines() {
        n += 1;
        println!("{}  {}", n, &l);
        vv.push(l.to_owned());
    }
    vv
}
fn match_input_task(task_instance: &mut Task, task_str: String, fix_task_vec: Vec<String>) {
    let vlen = fix_task_vec.len() as u32;
    let mut v = vec![];
    for i in 1..=vlen {
        v.push(i.to_string())
    }
    let vs = v.join("");
    let s = task_str.parse::<u32>();
    match s {
        Ok(x) => {
            if !vs.contains(&x.to_string()) {
                println!("out number range{}", x);
                return;
            }
            let n = x - 1;
            let item = fix_task_vec.get(n as usize).unwrap().to_owned();
            task_instance.set_task(&item.to_string());
        }
        Err(_e) => {
            let item = task_str;
            task_instance.set_task(&item.to_string());
        }
    }
}

fn work_flow(task_instance: &mut Task) {
    // get current_ts
    let cur_ts = TimeStamp::current_ts();
    let tkits = task_instance;

    // choose task_mode
    let get_task_mode = input_something("input task-mode s:single,m:multi：").unwrap();
    if get_task_mode == "s" {
        //    cur_ts as end_ts
        tkits.onetaskts.set_end_ts(&cur_ts);
        // set onedaydur
        tkits.onetaskts.calcu_set_onetask_dur();
        // single task;
        // print fixed tasks ,waiting for being selected
        let v = display_task();
        let task = input_something(
            format!(
                "{}-{}做了什么？",
                tkits.onetaskts.begin_ts.return_ts(),
                tkits.onetaskts.end_ts.return_ts()
            )
            .as_str(),
        )
        .unwrap();
        match_input_task(tkits.borrow_mut(), task, v);
        let detail = input_something("输入工作细节：").unwrap();
        tkits.set_detail(&detail);
        // pack task
        let pack_ln = tkits.psudo_pack();
        // write to todo
        append_line_into_todo(pack_ln);
    } else if get_task_mode == "m" {
        tkits.onetaskts.set_end_ts(&cur_ts);
        let curts = tkits.onetaskts.end_ts.return_ts();
        // multi task
        // print fixed tasks ,waiting for being selected
        loop {
            // return begin ts

            let end_ts = input_something(
                format!("input job end time(q:quit ; n: current_ts {})：", curts).as_str(),
            )
            .unwrap();

            if end_ts.chars().count() == 1 {
                //   inply input n ,cur_ts as end_ts
                //  set onetask_dur

                if end_ts == "n" {
                    tkits.onetaskts.set_end_ts(&cur_ts);
                } else if end_ts == "q" {
                    break;
                } else {
                    println!("wrong end ts commmand input");
                    return;
                }
            } else {
                tkits.onetaskts.set_end_ts(&end_ts);
            }

            // get task and detail
            let v_fixtask = display_task();
            let task = input_something("input tasknum or plain task：").unwrap();
            match_input_task(tkits.borrow_mut(), task, v_fixtask);
            let detail = input_something("输入工作细节：").unwrap();
            tkits.set_detail(&detail);
            tkits.onetaskts.calcu_set_onetask_dur();
            // pack task
            let pack_ln = tkits.psudo_pack();
            // write to todo
            append_line_into_todo(pack_ln);
            //    set last end_ts as this time begin_ts
            tkits.onetaskts.begin_ts = tkits.onetaskts.end_ts.into();
        }
    }
}
/// ts:timestamp
#[derive(Debug, Default, PartialEq, Clone, Copy)]
struct DayEndTs {
    getup_ts: TimeStamp,
    bed_ts: TimeStamp,
    day_duration: u32,
}
impl DayEndTs {
    fn set_getup_ts(&mut self, hm_str: &String) {
        self.getup_ts = hm_str.into()
    }
    fn set_bed_ts(&mut self, hm_str: String) {
        self.bed_ts = (&hm_str).into()
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
}
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Task {
    /// for db insert use
    index: u32,
    task: String,
    date: Date,
    dayendts: DayEndTs,
    onetaskts: OneTaskTs,
    detail: String,
}

impl Task {
    fn is_task_instance_default(&self) -> bool {
        if self.date==Date::default() || self.dayendts==DayEndTs::default()
         || self.onetaskts==OneTaskTs::default() || self.task==String::new() 
          || self.detail==String::new(){
            return true;
        }
        false
    }
    fn set_date(&mut self) {
        self.date = Date::today_date()
    }
    fn set_detail(&mut self, detail: &String) {
        self.detail = detail.to_owned();
    }
    fn set_task(&mut self, task: &String) {
        self.task = task.to_owned();
    }
    fn return_task_dbline(&self, id_num: u32) -> [String; 10] {
        let v = [
            (id_num.to_string()),
            (self.date.into()),
            (self.dayendts.getup_ts.return_ts().to_owned()),
            (self.dayendts.bed_ts.return_ts().to_owned()),
            (self.dayendts.day_duration.to_string()),
            (self.onetaskts.begin_ts.return_ts().to_owned()),
            (self.onetaskts.end_ts.return_ts().to_owned()),
            (self.onetaskts.one_task_duration.to_string()),
            (self.task.to_owned()),
            (self.detail.to_owned()),
        ];
        v
    }
    fn new() -> Task {
        Self::default()
    }
    fn set_index(&mut self, idx: u32) {
        self.index = idx;
    }
    pub fn start() {
        let env_args = env::args();
        let mut v_arg = vec![];
        //  make summary by add env arg "s"
        if env_args.len() == 2 {
            for i in env_args {
                v_arg.push(i)
            }
            let arg = v_arg.pop().unwrap();
            if arg == "s" {
                //    run create db
                let conn = Sqlite::new_conn("task.db").unwrap();

                conn.db.execute_batch(include_str!("create.sql")).unwrap();
                //    get an instance of Task
                let mut t = Task::new();
                //    summary
                let v_alll = read_alllines_from_todo();

                //    set dayendts from todo
                t.date.load_date_from_file();
                // set getup bed ts
                t.psudo_unpack(v_alll.get(0).unwrap().to_owned());
                t.dayendts.set_getup_ts(&t.onetaskts.begin_ts.return_ts());
                t.psudo_unpack(v_alll.last().unwrap().to_owned());
                t.dayendts.set_bed_ts(t.onetaskts.end_ts.return_ts());
                //    calcu and set dayend_dur
                t.dayendts.calcu_set_day_dur();

                //    get db_last_index from db_query
                let mut idx = Sqlite::get_last_index();
                //    generate a vec of tasks
                let mut v_alltk = vec![];

                for line in v_alll {
                    idx += 1;
                    t.set_index(idx);
                    t.psudo_unpack(line);

                    if t.is_task_instance_default() {
                        println!("task instance stay init state")
                    }

                    let ar = t.return_task_dbline(idx);
                    v_alltk.push(ar);
                }
                println!("{:?}", &v_alltk);
                let sql = "INSERT INTO everytask VALUES (?,?,?,?,?,?,?,?,?,?)";
                conn.db_execute_many(sql, v_alltk).unwrap();
            }
            println!("clear file contents of todo.txt,date.txt");
            clear_contents("todo.txt");
            clear_contents("date.txt");

            return;
        }
        //  create a new task instance
        let task_instance = Rc::new(RefCell::new(Task::new()));
        let tkits = &mut *(*task_instance).borrow_mut();
        // check if todo.txt is empty
        create_ifnotexist("todo.txt");
        if file_contents_empty("todo.txt") {
            // write today's date
            create_ifnotexist("date.txt");
            if file_contents_empty("date.txt") {
                Date::write_date();
            }

            tkits.set_date();
            // true   ,start from input getup_ts,this as begin_ts
            let get_getup_ts = input_something("when did you getup：").unwrap();
            // return begin - end ts
            // set getup_ts,begin_ts

            tkits.dayendts.set_getup_ts(&get_getup_ts);
            tkits.onetaskts.set_begin_ts(&get_getup_ts);
            work_flow(tkits);

            return;
        }

        // false ,start from load task from last line of todo.txt
        let lline = read_lastline_from_todo();
        // load to struct from lline
        tkits.psudo_unpack(lline);
        // load date from txt
        tkits.date.load_date_from_file();
        //    set last end_ts as this time begin_ts
        tkits
            .onetaskts
            .set_begin_ts(&tkits.onetaskts.end_ts.return_ts());
        work_flow(tkits);
    }
    // format task to todo.txt line by line
    fn psudo_pack(&self) -> String {
        // not finished
        format!(
            "{};;{};;{}",
            self.onetaskts.return_ts_dur_line(),
            self.task,
            self.detail
        )
    }
    /// split string by seo ";;",process data from todo.txt
    fn psudo_unpack(&mut self, strs: String) {
        let v = strs.split(";;").collect::<Vec<&str>>();
        self.onetaskts.set_begin_ts(&v.get(0).unwrap().to_string());
        self.onetaskts.set_end_ts(&v.get(1).unwrap().to_string());
        self.onetaskts
            .set_onetask_dur(&v.get(2).unwrap().to_string());
        self.set_task(&v.get(3).unwrap().to_string());
        self.set_detail(&v.get(4).unwrap().to_string());
    }
}

#[test]
fn test_unpack() {
    let mut tk = Task::new();
    let s = "8:0;;10:54;;174;;吃饭;;小葱";
    tk.psudo_unpack(s.to_owned());
    println!("{:?}", tk);
}
#[test]
fn test_intlen() {
    let s = 00.to_string();
    println!("00 {}", s);
}
