use crate::file_op::*;
use crate::sql_op::Sqlite;
use chrono::prelude::*;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::rc::Rc;
use std::{
    env, fs,
    io::{self, prelude::*},
};
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
impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}-{}-{})", self.year, self.month, self.day)
    }
}
impl Date {
    fn weekday(&self) -> String {
        let local = Local::now();
        let y = local.year();
        let local_dt = Local.ymd(y, self.month, self.day);
        local_dt.weekday().to_string()
    }
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
        *self = if ds == "" {
            println!("no date in file");
            let dt: String = Date::today_date().into();
            dt.into()
        } else {
            ds.into()
        }
    }
}
#[derive(Debug, Default, PartialEq, Clone, Copy)]
struct TimeStamp {
    hour: u32,
    minute: u32,
}
impl From<TimeStamp> for String {
    fn from(ts: TimeStamp) -> Self {
        format!("{:02}:{:02}", &ts.hour, &ts.minute)
    }
}
impl Into<TimeStamp> for String {
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
        format!("{:02}:{:02}", local.hour(), local.minute())
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
    fn set_onetask_dur(&mut self, dur: &String) {
        let i = dur.parse::<u32>().unwrap();
        self.one_task_duration = i
    }
    fn set_end_ts(&mut self, ts: &String) {
        self.end_ts = ts.to_owned().into()
    }
    fn set_begin_ts(&mut self, ts: &String) {
        self.begin_ts = ts.to_owned().into()
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
fn input_something<T: AsRef<str> + Display>(text_hint: T) -> io::Result<String> {
    print!("{}", text_hint);
    io::stdout().flush().unwrap();
    let mut bf = String::new();
    io::stdin().read_line(&mut bf)?;

    Ok(bf.trim().to_owned())
}
/// print out fixed tasks from file line by line
fn display_task() -> Vec<String> {
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
/// set task of task_instance  by input num or plain task
fn match_input_task(task_instance: &mut Task, task_str: String, fix_task_vec: Vec<String>) {
    let vlen = fix_task_vec.len() as u32;
    let v = (1..=vlen)
        .into_iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>();
    let vs = v.join("");
    let s = task_str.parse::<u32>();
    match s {
        Ok(x) => {
            if !vs.contains(&x.to_string()) {
                dbg!("out number range {}", x);
                return;
            }
            let n = x - 1;
            let item = fix_task_vec.get(n as usize).unwrap().to_owned();
            if item == "午餐" {
                input_something("have you take out pot from rice cooker").unwrap();
            }

            task_instance.set_task(&item.to_string());
        }
        Err(_e) => {
            let item = task_str;
            task_instance.set_task(&item.to_string());
        }
    }
}
fn summary_tasks(ntask: &NewTask) {
    let task_str = format!(
        "expected schedule {} detail {} \n
        task: {} detail: {} ,last for {}
        ",
        ntask.expected_behavior,
        ntask.expected_details,
        ntask.task.task,
        ntask.task.detail,
        ntask.task.onetaskts.dur_to_hm(),
    );
    append_line_into_file("summary.txt", task_str);
}
/// create file  if not exist once app starts
fn init_file() {
    create_ifnotexist("todo.txt");
    create_ifnotexist("date.txt");
    create_ifnotexist("extask.txt");
    create_ifnotexist("fix_task.txt");
    create_ifnotexist("expect_behavior.txt")
}
/// copy task.db and summary.txt to ./storage/shared/ if weekday is Sat
fn cp_taskdb_to_storage() {
    let db = "task.db";
    let sm = "summary.txt";
    let t = "../storage/shared/task.db";
    let smt = "../storage/shared/summary.txt";
    let weekday = Local::now().weekday().to_string();
    if weekday == "Sat".to_string() {
        println!("copy task.db summary.txt to phone storage");
        fs::copy(db, t).unwrap();
        fs::copy(sm, smt).unwrap();
    }
}
fn get_exptected_task_details() -> (String, String) {
    let mut v = read_alllines_from_file("expect_behavior.txt");
    if !v.is_empty() {
        let ext = v.remove(0);
        let exd = v.join(" ");
        (ext, exd)
    } else {
        ("".to_string(), "".to_string())
    }
}
/// append_backup_task_to_extask if date is saturday
fn write_backup_task_to_extask() {
    let mut f = OpenOptions::new().append(true).open("extask.txt").unwrap();
    let weekday = Local::now().weekday().to_string();
    if weekday == "Sat".to_string() {
        writeln!(f, "备份task.db,web source,read book").unwrap();
        println!("already write backup task to next task file");
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
        println!("check if you fullfil your expect");
        let v = read_alllines_from_file("expect_behavior.txt");
        if !v.is_empty() {
            for i in v {
                println!("{}", i);
            }
        } else {
            println!("No Schedule")
        }
        let detail = input_something("输入工作细节：").unwrap();
        tkits.set_detail(&detail);

        // here read ex task and detail from txt
        // only works on single mode
        let args = get_exptected_task_details();
        let newtask =
            NewTask::update_from_old_tasks(&*tkits, (args.0.as_str(), args.1.as_str())).unwrap();
        clear_contents("expect_behavior.txt");
        // pack task
        let string_ln = newtask.to_string();
        // write to todo
        append_line_into_file("todo.txt", string_ln);
    } else if get_task_mode == "m" {
        tkits.onetaskts.set_end_ts(&cur_ts);
        let curts = tkits.onetaskts.end_ts.return_ts();
        // multi task
        // print fixed tasks ,waiting for being selected
        loop {
            // return begin ts
            println!("begin_ts is {}", tkits.onetaskts.begin_ts.return_ts());
            let end_ts = input_something(format!(
                "input job end time(q:quit ; n: current_ts {})：",
                curts
            ))
            .unwrap();

            if end_ts.chars().count() == 1 {
                //   imply according to input n ,cur_ts as end_ts
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
            let newtask = NewTask::update_from_old_tasks(&*tkits, ("", "")).unwrap();
            let string_ln = newtask.to_string();
            clear_contents("expect_behavior.txt");
            // write to todo
            append_line_into_file("todo.txt", string_ln);
            //    set last end_ts as this time begin_ts
            tkits.onetaskts.begin_ts = tkits.onetaskts.end_ts.into();
        }
    }
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
    fn set_getup_ts(&mut self, hm_str: &String) {
        self.getup_ts = hm_str.to_owned().into()
    }
    fn set_bed_ts(&mut self, hm_str: String) {
        self.bed_ts = hm_str.into()
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
#[derive(Debug, Default)]
struct ExTask {
    idx: u64,
    date: Date,
    extask: String,
    state: u8,
    descptions: String,
}
impl ExTask {
    fn set_index(&mut self, idx: u64) {
        self.idx = idx;
    }
    fn set_date(&mut self) {
        self.date = Date::today_date();
    }
    fn set_extask(&mut self, task: &String) {
        self.extask = task.to_owned();
    }
    fn to_slice(&self) -> [String; 5] {
        let sl = [
            self.idx.to_string(),
            self.date.to_string(),
            self.extask.to_owned(),
            self.state.to_string(),
            self.descptions.to_owned(),
        ];
        sl
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
pub struct NewTask {
    expected_behavior: String,
    /// readlines to one line join by ' '
    expected_details: String,
    task: Task,
}
impl ToString for NewTask {
    fn to_string(&self) -> String {
        format!(
            "{};;{};;{}",
            self.task.psudo_pack(),
            self.expected_behavior,
            self.expected_details
        )
    }
}
impl NewTask {
    pub fn update_from_old_tasks(old_task: &Task, new_args: (&str, &str)) -> io::Result<Self> {
        Ok(Self {
            task: old_task.to_owned(),
            expected_behavior: new_args.0.to_owned(),
            expected_details: new_args.1.to_owned(),
        })
    }
    pub fn to_slice(&self) -> [String; 12] {
        let o_v = self.to_owned().task.to_slice();
        [
            (&o_v[0]).to_owned(),
            (&o_v[1]).to_owned(),
            (&o_v[2]).to_owned(),
            (&o_v[3]).to_owned(),
            (&o_v[4]).to_owned(),
            (&o_v[5]).to_owned(),
            (&o_v[6]).to_owned(),
            (&o_v[7]).to_owned(),
            (&o_v[8]).to_owned(),
            (&o_v[9]).to_owned(),
            self.to_owned().expected_behavior,
            self.to_owned().expected_details,
        ]
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

impl Task {
    fn is_task_instance_default(&self) -> bool {
        if self.date == Date::default()
            || self.dayendts == DayEndTs::default()
            || self.onetaskts == OneTaskTs::default()
            || self.task == String::new()
            || self.detail == String::new()
        {
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
    pub fn to_slice(self) -> [String; 10] {
        let v = [
            (self.index.to_string()),
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
    fn return_task_dbline(&self, id_num: u64) -> [String; 10] {
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
    fn set_index(&mut self, idx: u64) {
        self.index = idx;
    }
    pub fn start() {
        // init to create
        init_file();
        let env_args = env::args().collect::<Vec<_>>();
        if let Some(arg) = env_args.get(1) {
            //  make summary by add env arg "s"
            if arg == "s" {
                //    run create db
                let conn = Sqlite::new_conn("task.db").unwrap();

                conn.db.execute_batch(include_str!("create.sql")).unwrap();
                //    get an instance of Task
                let mut t = Task::new();
                //    summary
                let v_alll = read_alllines_from_file("todo.txt");

                //    set dayendts from todo
                t.date.load_date_from_file();
                // set getup bed ts
                t.psudo_unpack(v_alll.get(0).unwrap().to_owned());
                t.dayendts.set_getup_ts(&t.onetaskts.begin_ts.return_ts());
                t.psudo_unpack(v_alll.last().unwrap().to_owned());
                t.dayendts.set_bed_ts(t.onetaskts.end_ts.return_ts());
                //    calcu and set dayend_dur
                t.dayendts.calcu_set_day_dur();
                // write date,dayendts to summary file
                let date_str = format!("date {} {}", t.date.to_string(), t.date.weekday());
                let daydur_str = format!(
                    "the day is from {} to {} ,last for {}",
                    t.dayendts.getup_ts.return_ts(),
                    t.dayendts.bed_ts.return_ts(),
                    t.dayendts.dur_to_hm()
                );
                append_line_into_file("summary.txt", date_str);
                append_line_into_file("summary.txt", daydur_str);

                //    get db_last_index from db_query
                let sql = "SELECT id from everydaytask  ORDER BY id DESC";
                let path = "task.db";
                let mut idx = Sqlite::get_last_index(sql, path);
                //    generate a vec of tasks
                let mut v_alltk = vec![];

                for line in v_alll {
                    idx += 1;
                    t.set_index(idx);
                    let nt = t.psudo_unpack(line);
                    if t.is_task_instance_default() {
                        panic!("task instance stay init state");
                    }

                    let ar = nt.to_slice();
                    v_alltk.push(ar);
                    // write today's jobs tp summary.txt
                    summary_tasks(&nt);
                }
                append_line_into_file("summary.txt", "\n".to_owned());
                // print today's task linebyline
                let sql = "INSERT INTO everydaytask VALUES (?,?,?,?,?,?,?,?,?,?,?,?)";
                conn.db_execute_many(sql, v_alltk).unwrap();
                input_something("Have ypu charged wifi machine? enter").unwrap();
                write_backup_task_to_extask();
                cp_taskdb_to_storage();
                println!("clear file contents of todo.txt,date.txt");
                clear_contents("todo.txt");
                clear_contents("date.txt");
            } else if arg == "c" {
                // nearly discarded

                //   c : check out expect task
                // manually fill in extask.txt  in advance
                // loop txt and ask whether a task is finished or not and ask to input desc(through input)
                let mut v = vec![];
                //    get db_last_index from db_query
                let conn = Sqlite::new_conn("task.db").unwrap();
                conn.db
                    .execute_batch(include_str!("create_check_expect_task.sql"))
                    .unwrap();
                let sql = "SELECT id from check_expect_task ORDER BY id DESC";
                let path = "task.db";
                let mut idx = Sqlite::get_last_index(sql, path);

                for i in read_alllines_from_file("extask.txt") {
                    idx += 1;
                    let mut extask = ExTask::default();
                    extask.set_index(idx);
                    extask.set_date();
                    extask.set_extask(&i);
                    let state = loop {
                        println!("{}", i);
                        let t = input_something(format!("finished? 0:false,1:true：")).unwrap();
                        if t == "1" || t == "0" {
                            let st = t.parse::<u8>().unwrap();
                            break st;
                        }
                    };
                    extask.state = state;
                    let desc = input_something("leave comments about extask:").unwrap();
                    extask.descptions = desc;

                    let sl = extask.to_slice();
                    v.push(sl);
                }
                // write data,task,desc,id ,states,into db
                let sql = "INSERT INTO check_expect_task VALUES (?,?,?,?,?)";
                conn.db_execute_many_ex(sql, v).unwrap();
                println!("finished check extask out");
            } else if arg == "a" {

                // other relavent fn needs modification
            } else {
                //   help out
                println!("s :summary today's tasks and write to db");
                println!("c : check out whether expect task is done");
            }
        } else {
            //  create a new task instance
            let task_instance = Rc::new(RefCell::new(Task::new()));
            let tkits = &mut *(*task_instance).borrow_mut();
            // check if todo.txt is empty

            if file_contents_empty("todo.txt") {
                // write today's date
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
            let lline = read_lastline_from_file("todo.txt");
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
    fn psudo_unpack(&mut self, strs: String) -> NewTask {
        let v = strs.split(";;").collect::<Vec<&str>>();
        self.onetaskts.set_begin_ts(&v.get(0).unwrap().to_string());
        self.onetaskts.set_end_ts(&v.get(1).unwrap().to_string());
        self.onetaskts
            .set_onetask_dur(&v.get(2).unwrap().to_string());
        self.set_task(&v.get(3).unwrap().to_string());
        self.set_detail(&v.get(4).unwrap().to_string());
        let nt = NewTask::update_from_old_tasks(self, (v.get(5).unwrap(), v.get(6).unwrap()));
        nt.unwrap()
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
#[test]
fn test_week() {
    let weekday = Local::now().weekday().to_string();
    //    Sun
    println!("{}", weekday);
}
#[test]
fn test_yu() {
    let s = 30;
    let r = 124 / 60;
    let x = 124 % 60;
    println!("{}{}", r, x);
}

#[test]
fn test_lines() {
    let cursor = File::open("t.txt").unwrap();
    let bf = BufReader::new(&cursor);
    let mut v = vec![];
    for i in bf.lines() {
        v.push(i.unwrap())
    }
    println!("{:?}", v);
}
