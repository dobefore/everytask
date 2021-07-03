// 1.use sqlite.insert,seleect(latest one,all),
// row(date:2021-06-07,getup_time(07:00),bed_time(22:30),begin_timestamp(08:10),
// end_timestamp(08:40),timespan(300)min,task("早餐"),details("面包"))

// 2.write summary.txt ,sql_cache.txt(deserial to row),read task_choice.txt,

// before insert into sqlite, format check of everyline task

// two modes of task input :single-task/multi-tasks

// overflow:
// 1:input getup_time(when this input,write current date to txt)
// 2. choose task_mode(read keyed tasks from txt)
// 2.1 single:
//
// input/choose task
// input details

// 2,2 multi:
//
// input end_time
// input/choose task,details
// ...
// input c as end_time=cur_time

// write task info to todo.txt( line by line
// serialize and deserialize:struct as bytes into file)

// summary:deserialize todo.txt into struct ,
// format and write to summary.txt
// format and write to sqlite db

mod task;
use task::Task;
mod file_op;
mod sql_op;
fn main() {
    Task::start()
}
