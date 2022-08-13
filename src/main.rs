mod task;
use task::Task;
mod error;
mod file_op;
mod pay;
mod sql_op;
fn main() {
    Task::start()
}
