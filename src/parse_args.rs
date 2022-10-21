use clap::{arg, command, Parser, Subcommand, ValueEnum};
#[derive(Parser, Debug)]
#[clap( version, long_about = None)]
pub(crate) struct Args {
    /// retrieve records from database e.g. pay.db
    #[command(subcommand)]
    pub(crate) cmd: Option<RetrieveCommand>,
}
#[derive(Parser, Debug)]
pub(crate) struct Dt {
    /// retrieve records of a specified month ,possible values: 1-12. 0 means this month
    #[arg(short, long, value_parser)]
    month: u8,
    /// retrieve records of a specified day,must prefix it with a month value, possible values:1-31
    #[arg(short, long, value_parser)]
    day: Option<u8>,
}
#[derive(Subcommand, Debug)]
pub(crate) enum RetrieveCommand {
    /// payment-related operations
    Pay {
        /// Retrieve payment records from database by a specified date,e.g. pay -r 9,3  
        #[clap(short, long, value_delimiter('.'))]
        retrieve: Vec<u8>,
        /// insert records about payments in file pay.txt into database
        #[arg(short, long, action)]
        insert: bool,
    },
    /// task-related operations
    Task {
        // /// Retrieve task records from database by a specified date
        // #[arg(short, long, value_delimiter(','))]
        // retrieve: Vec<u8>,
        /// summary all today's tasks,write them to a file
        #[arg(short, long, action)]
        summary: bool,
        /// rearrange orders of tasks in files like today_target.txt
        #[arg(short, long, action)]
        order: bool,
        /// upload all files in everydaytask/ to aliyun drive
        #[arg(short, long, action)]
        upload: bool,
    },
    /// search and retrieve records which include a certain words.
    Search {
        ///  choose which db to perform query
        #[arg(value_enum, short, long)]
        dbname: DBOption,
        /// words in table field task and detail.
        #[arg(short, long, value_parser)]
        words: String,
    },
}
#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum DBOption {
    Pay,
    Task,
}
