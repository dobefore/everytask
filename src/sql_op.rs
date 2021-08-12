use std::path::Path;

use rusqlite::OptionalExtension;
pub use rusqlite::{params, types::FromSql, Connection, Result, Row, RowIndex};
pub struct Sqlite {
    pub db: Connection,
}
impl Sqlite {
    pub fn new_conn<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            db: Connection::open(path)?,
        })
    }
    /// query lastone row
    pub fn fetchone<T: FromSql>(&self, sql: &str) -> Result<T> {
        let s = self.db.query_row(sql, params![], |r| r.get(0))?;
        Ok(s)
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
    pub fn db_execute_many(&self, sql: &str, args: Vec<[String; 10]>) -> Result<()> {
        let mut stmt = self.db.prepare(sql)?;

        for param in args {
            stmt.execute(params![
                param[0], param[1], param[2], param[3], param[4], param[5], param[6], param[7],
                param[8], param[9]
            ])?;
        }

        Ok(())
    }
}

#[test]
fn test_table_field() {
    let q = "select * from everytask";
    let s = "SELECT sql FROM sqlite_master WHERE type = 'table' AND tbl_name = 'everytask';";
    let c = Sqlite::new_conn("task.db").unwrap();
    let a: Vec<usize> = c.fetchall(q, 7).unwrap();
    println!("{:?}", a);
}
