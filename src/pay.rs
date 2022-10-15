#![allow(dead_code)]
use std::{collections::HashMap, fmt::Display, path::PathBuf, result};

use rusqlite::{params, Connection};

use crate::{
    error::{CustomError, Result, TaskError},
    file_op::{append_line, clear_contents, open_as_append, read_lastline, read_lines},
    sql_op::{fetch_all, open_db},
    task::{self, Date, DATE_FILE, PAY_FILE, TASK_DB_FILE},
};
/// move this into a separate command line argument,e.g. task pay
//payments record
#[derive(Debug, Default)]
pub struct Pay {
    conn: Option<Connection>,
    payitem: Option<Vec<PayItem>>,
    payfile: Option<PathBuf>,
}

impl Pay {
    pub fn new(
        conn: Option<Connection>,
        payitem: Option<Vec<PayItem>>,
        payfile: Option<PathBuf>,
    ) -> Self {
        Self {
            conn,
            payitem,
            payfile,
        }
    }
    /// update date of pay db
    ///
    ///
    fn update_date(&self, old_date: &str, new_date: &str) -> Result<&Self> {
        // target_date is the date that will be added and another one will be replaced
        let sql = format!("UPDATE pay SET date_ = '{}' WHERE date_ = ?", new_date);
        // re-insert records to db
        self.conn
            .as_ref()
            .unwrap()
            .execute(&sql, params![old_date])?;
        Ok(self)
    }

    fn to_payitem(row: &rusqlite::Row) -> rusqlite::Result<PayItem, rusqlite::Error> {
        Ok(PayItem {
            date: row.get(0)?,
            item: row.get(1)?,
            price: row.get(2)?,
            amounts: row.get(3)?,
            category: row.get(4)?,
        })
    }

    ///basically,query field `item`.
    pub(crate) fn query_pay(
        conn: &Connection,
        words: &str,
    ) -> rusqlite::Result<Option<Vec<PayItem>>, rusqlite::Error> {
        let sql = format!(
            "SELECT date_,item,price,amounts,category FROM pay where item like '%{}%'",
            words
        );
        Ok(fetch_all(&sql, conn, Pay::to_payitem)?)
    }
    /// retrieve records on a specified day from db.
    pub(crate) fn retrieve_day(
        conn: &Connection,
        date: task::Date,
    ) -> rusqlite::Result<Option<Vec<PayItem>>, rusqlite::Error> {
        let sql = "SELECT date_,item,price,amounts,category FROM pay where date_=?";
        let date = date.to_string();

        let mut stmt = conn.prepare(&sql)?;
        // [Ok(TB { c: "c1", idx: 1 }), Ok(TB { c: "c2", idx: 2 })]
        let r = stmt
            .query_map([date], Pay::to_payitem)?
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>();
        Ok(if r.is_empty() { None } else { Some(r) })
    }
    /// retrieve records of a specified month from db.
    ///return None if no records in db
    /// # Errors
    ///
    /// This function will return an error if .
    pub(crate) fn retrieve_month(
        conn: &Connection,
        month: u8,
    ) -> rusqlite::Result<Option<Vec<PayItem>>, rusqlite::Error> {
        let sql = format!(
            "SELECT date_,item,price,amounts,category FROM pay where date_ like '_____{}%'",
            month
        );

        Ok(fetch_all(&sql, conn, Pay::to_payitem)?)
    }

    /// used to test or remove records all
    fn drop_table(&self) -> Result<()> {
        let sql = "DROP TABLE pay;";
        self.conn.as_ref().unwrap().execute(sql, [])?;
        Ok(())
    }
    /// add payitems [`PayItem`] to db
    pub fn add_to_db(&self) -> Result<&Self> {
        let sql = "INSERT INTO pay VALUES (?,?,?,?,?)";
        for pi in self.payitem.as_ref().unwrap() {
            self.conn.as_ref().unwrap().execute(
                sql,
                params![pi.date, pi.item, pi.price, pi.amounts, pi.category],
            )?;
        }
        Ok(self)
    }
    /// remove records from file in current day
    ///
    /// but keep lines start with #
    pub fn clear_file(&self) -> Result<()> {
        let path = format!("{}", self.payfile.as_ref().unwrap().display());
        let mut f = open_as_append(&path).unwrap();
        let old_lines = read_lines(&path).unwrap();
        clear_contents(&path)?;
        for l in old_lines {
            if l.starts_with("#") {
                append_line(&mut f, &l).unwrap();
            }
        }
        Ok(())
    }

    pub fn read_from_file(
        conn: Option<Connection>,
        path: &str,
        datestr: Option<String>,
    ) -> Result<Self> {
        let lines = read_lines(path)?;
        let mut ps = vec![];

        for l in lines {
            if l.starts_with("#") || l.trim().is_empty() {
                continue;
            }
            let payitem = PayItem::from_string(&l, datestr.clone())?;
            ps.push(payitem);
        }

        Ok(Self::new(conn, Some(ps), Some(path.into())))
    }
}

#[derive(Debug, Default)]
pub struct PayItem {
    item: Option<String>,
    date: Option<String>,
    price: Option<String>,
    amounts: Option<String>,
    category: Option<String>,
}
impl Display for PayItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, price:{},amounts: {}, category: {},date: {}",
            self.item.as_ref().unwrap(),
            self.price.as_ref().unwrap(),
            self.amounts.as_ref().unwrap(),
            self.category.as_ref().unwrap(),
            self.date.as_ref().unwrap()
        )
    }
}
impl PayItem {
    fn new(
        item: Option<String>,
        date: Option<String>,
        price: Option<String>,
        amounts: Option<String>,
        category: Option<String>,
    ) -> Self {
        Self {
            item,
            date,
            price,
            amounts,
            category,
        }
    }
    fn category<'a>() -> HashMap<u8, &'a str> {
        let mut m: HashMap<u8, &str> = HashMap::new();
        m.insert(1, "买菜");
        m.insert(2, "日用品");
        m.insert(3, "电子产品");
        m
    }

    /// impl from <string> ,separated by ','
    pub fn from_string(s: &str, date: Option<String>) -> Result<Self> {
        let r = s.split('，').collect::<Vec<&str>>();
        let categories = PayItem::category();
        let item = r.get(0).unwrap().trim().to_string();
        let price = r.get(1).unwrap().trim();

        let item = if item.is_empty() {
            Err(CustomError::ValueEmpty("item".into()))
        } else {
            Ok(item)
        };
        let price: result::Result<_, TaskError> = if price.is_empty() {
            Err(CustomError::ValueEmpty("price".into()).into())
        } else {
            Ok(price.parse::<f32>()?.to_string())
        };
        let category = if r.len() == 3 || r.len() == 4 {
            let category = r.get(2).unwrap().trim().to_string();
            if category.is_empty() {
                Ok("默认".into())
            } else {
                match categories.get(&category.parse::<u8>()?) {
                    Some(c) => Ok(c.to_string()),
                    None => Err(CustomError::ValueNotFound(format!(
                        "no key {} found in categories",
                        category
                    ))),
                }
            }
        } else {
            Ok("默认".into())
        };
        // amounts : parse string to u8 if len of fields is 4 ,else return 0 denoting default value
        let amounts = if r.len() == 4 {
            let amounts = r.get(3).unwrap().trim();
            if amounts.is_empty() {
                "默认"
            } else {
                amounts
            }
        } else {
            "默认"
        };
        // : merge 2,3,4
        let len_of_fields = r.len();
        if len_of_fields == 2 || len_of_fields == 3 || len_of_fields == 4 {
            // means fields are respectly items,price，category,amounts
            return Ok(Self::new(
                Some(item?),
                date,
                Some(price?),
                Some(amounts.into()),
                Some(category?),
            ));
        } else {
            // no available
            return Err(CustomError::ParsePayItemError(format!(
                "invalid numbers of fields {}",
                r.len()
            ))
            .into());
        };
    }
}

/// write payments to db.
pub fn write_pay() -> result::Result<(), TaskError> {
    let conn = open_db(TASK_DB_FILE)?;
    // read date from file
    let s = read_lastline(DATE_FILE)?.unwrap();
    let datestr = Date::load_date_from_str(&s);
    Pay::read_from_file(Some(conn), PAY_FILE, Some(datestr.to_string()))?
        .add_to_db()?
        .clear_file()?;
    Ok(())
}
// /// select an appointed date,and replace them with a specific one.
// ///
// /// 2022-8-30 to 2022-8-27
// fn update_pay_date() -> result::Result<(), TaskError> {
//     println!("{:?}", env::current_dir());
//     let conn = Sqlite::new_conn("target/debug/task.db")?;
//     conn.db.execute_batch(include_str!("create.sql"))?;
//     Pay::new(Some(conn.db), None, None).update_date("2022-8-30", "2022-8-27")?;
//     Ok(())
// }

#[test]
fn test_float_str() {
    let _f = 4.2f32;
    let f1 = "4.2".parse::<f32>().unwrap();
    println!("{}", f1.to_string())
}
