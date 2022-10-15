#![allow(dead_code)]
pub use rusqlite::{params, types::FromSql, Connection, Result, Row, RowIndex};
/// drop unused colunms from table
///
/// 1. create a new temp table from old one ,but not unused fields
///
/// 2. rename new table  
fn drop_column(conn: Connection) -> Result<(), rusqlite::Error> {
    let sql="create table temp as select id ,date_ ,getup_ts ,bed_ts ,day_dur ,begin_ts ,end_ts ,one_task_dur,task,detail from everydaytask where 1 = 1;";
    let sql1 = "drop table everydaytask; 
drop table everytask;  
alter table temp rename to everydaytask;";
    conn.execute(sql, [])?;
    conn.execute_batch(sql1)?;
    Ok(())
}

// open db connection and create table
pub(crate) fn open_db<P: AsRef<std::path::Path>>(
    db_path: P,
) -> Result<Connection, rusqlite::Error> {
    let db_path = db_path.as_ref();
    let conn = Connection::open(db_path)?;
    conn.execute(include_str!("create.sql"), [])?;
    Ok(conn)
}
/// fetch first record if there are more than one.
pub(crate) fn fetch_one<T: FromSql>(conn: &Connection, sql: &str) -> Result<T, rusqlite::Error> {
    let s = conn.query_row(sql, params![], |r| r.get(0))?;

    Ok(s)
}
pub(crate) fn fetch_all<T, F>(
    sql: &str,
    conn: &Connection,
    to_struct: F,
) -> rusqlite::Result<Option<Vec<T>>, rusqlite::Error>
where
    F: FnMut(&Row) -> Result<T, rusqlite::Error>,
{
    let mut stmt = conn.prepare(&sql)?;
    // [Ok(TB { c: "c1", idx: 1 }), Ok(TB { c: "c2", idx: 2 })]
    let r = stmt
        .query_map([], to_struct)?
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    Ok(if r.is_empty() { None } else { Some(r) })
}
