mod task;
use clap::Parser;
use task::Task;
mod error;
mod file_op;
mod parse_args;
mod pay;
mod sql_op;
mod task_handle;
fn main() {
    let arg = parse_args::Args::parse();
    match Task::start(arg) {
        Ok(_) => {}
        Err(e) => eprintln!("{e}"),
    }
    // Task::start();
}
