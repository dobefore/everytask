use std::path::Path;

use rusqlite::OptionalExtension;
pub use rusqlite::{params, types::FromSql, Connection, Result, Row, RowIndex};

use crate::{
    error::TaskError,
    task::{DayEndTs, NewTask, PercentageTasks, Task, TimeStamp},
};
pub struct Sqlite {
    pub db: Connection,
}
impl Sqlite {
    pub fn new_conn<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            db: Connection::open(path)?,
        })
    }
    /// create table if not exists
    pub fn create_table_pay(&self) -> Result<()> {
        self.db.execute_batch(include_str!("create.sql"))?;
        Ok(())
    }
    /// query lastone row
    pub fn fetchone<T: FromSql>(&self, sql: &str) -> Result<T> {
        let s = self.db.query_row(sql, params![], |r| r.get(0))?;
        Ok(s)
    }
    pub fn to_percentage_tasks(&self, sql: &str) -> Result<Vec<PercentageTasks>, TaskError> {
        let mut s = self.db.prepare(sql)?;
        let mut rows = s.query(params![])?;
        let mut items: Vec<PercentageTasks> = vec![];
        while let Some(row) = rows.next()? {
            let r = PercentageTasks::new()
                .set_one_task_dur(row.get::<usize, u16>(0)?.into())
                .set_task(&row.get::<usize, String>(1)?);
            items.push(r);
        }
        Ok(items)
    }
    pub fn fetchall<I: RowIndex + Copy, T: FromSql>(
        &self,
        sql: &str,
        index_of_column: I,
    ) -> Result<Vec<T>> {
        let mut s = self.db.prepare(sql)?;
        let mut rows = s.query(params![])?;
        let mut items: Vec<T> = vec![];
        while let Some(row) = rows.next()? {
            items.push(row.get(index_of_column).unwrap());
        }
        Ok(items)
    }
    /// get last db index number
    pub fn get_last_index(sql: &str, db: &str) -> u64 {
        // test result if table no data
        let conn = Sqlite::new_conn(db).unwrap();
        let s: Option<u64> = conn.fetchone(sql).optional().unwrap();
        if s.is_none() {
            return 0;
        }
        s.unwrap()
    }
    /// eg: excu insert manytimes check_expect_task
    pub fn db_execute_many_ex(&self, sql: &str, args: Vec<[String; 5]>) -> Result<()> {
        let mut stmt = self.db.prepare(sql)?;

        for param in args {
            stmt.execute(params![param[0], param[1], param[2], param[3], param[4]])?;
        }

        Ok(())
    }
    /// eg: excu insert manytimes
    pub fn db_execute_many(&self, sql: &str, args: Vec<[String; 12]>) -> Result<()> {
        let mut stmt = self.db.prepare(sql)?;

        for param in args {
            stmt.execute(params![
                param[0], param[1], param[2], param[3], param[4], param[5], param[6], param[7],
                param[8], param[9], param[10], param[11]
            ])?;
        }

        Ok(())
    }
}
// row from table everytask de-construct to old task struct
fn to_task(row: &Row) -> Result<Task> {
    Ok(Task {
        index: row.get(0).unwrap(),
        date: row.get::<usize, String>(1).unwrap().into(),
        dayendts: (
            row.get(2).unwrap(),
            row.get(3).unwrap(),
            row.get(4).unwrap(),
        )
            .into(),
        onetaskts: (
            row.get(5).unwrap(),
            row.get(6).unwrap(),
            row.get(7).unwrap(),
        )
            .into(),
        task: row.get(8).unwrap(),
        detail: row.get(9).unwrap(),
    })
}
#[test]
fn test_table_field() {
    let q = "select * from everytask";
    let s = "SELECT sql FROM sqlite_master WHERE type = 'table' AND tbl_name = 'everytask';";
    let c = Sqlite::new_conn("task.db").unwrap();
    let a: Vec<usize> = c.fetchall(q, 7).unwrap();
    println!("{:?}", a);
}
///fetch all old tasks rec write to newtask
#[test]
fn test_fetchall_write_newtask() -> Result<()> {
    let db = Connection::open("task.db").unwrap();
    let mut s = db.prepare("select * from everytask").unwrap();
    let rows = s.query_and_then([], |row| to_task(row)).unwrap();

    db.execute_batch(include_str!("create.sql")).unwrap();
    let sql = "INSERT INTO everydaytask VALUES (?,?,?,?,?,?,?,?,?,?,?,?)";

    // update recs to new task
    for i in rows {
        // complete new task update fn
        // write newtasks to db
        let ex_t = "";
        let ex_d = "";
        let nt = NewTask::update_from_old_tasks(&i.unwrap(), (ex_t, ex_d)).unwrap();
        let v = nt.to_slice();
        println!("{:?}", v);
        db.execute(sql, v).unwrap();
    }
    Ok(())
}
