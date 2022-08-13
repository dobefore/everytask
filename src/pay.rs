use std::{collections::HashMap, path::PathBuf, result, fmt::Display};

use rusqlite::{params, Connection};

use crate::{
    error::{CustomError, Result, TaskError},
    file_op::{
        append_line_into_file, clear_contents, read_alllines_from_file, read_lastline_from_file,
    },
    task::{self}, sql_op::Sqlite,
};
/// move this into a separate command line argument,e.g. task pay
//payments record
#[derive(Debug, Default)]
pub struct Pay {
    conn: Option< Connection>,
    payitem: Option<Vec<PayItem>>,
    payfile: Option<PathBuf>,
}

impl Pay {
    pub fn new(
        conn: Option< Connection>,
        payitem: Option<Vec<PayItem>>,
        payfile: Option<PathBuf>,
    ) -> Self {
        Self {
            conn,
            payitem,
            payfile,
        }
    }
    pub fn retrieve_records_form_last_day(&mut self,db:Option<&str>)->Result<()> {
         // retrive payment records of last day from db
         let conn = Sqlite::new_conn(db.unwrap()).unwrap();
         conn.db.execute_batch(include_str!("create.sql")).unwrap();
         self.set_conn(Some(conn.db));
      let res=   self.retrieve_records(true, false).unwrap();
    if let Some(r) =res  {
        for pi in  r{
            println!("{}",pi?);
        }
    }

    Ok(())
    }
    pub fn retrieve_records_form_last_month(&mut self,db:Option<&str>)->Result<()> {
        // retrive payment records of last day from db
        let conn = Sqlite::new_conn(db.unwrap()).unwrap();
        conn.db.execute_batch(include_str!("create.sql")).unwrap();
        self.set_conn(Some(conn.db));
     let res=   self.retrieve_records(false, true).unwrap();
   if let Some(r) =res  {
       for pi in  r{
           println!("{}",pi?);
       }
   }
   Ok(())

   }
    /// query records of either last day or last month  from db
    pub fn retrieve_records(
        &self,
        last_day: bool,
        last_month: bool,
    ) -> Result<Option<Vec<result::Result<PayItem, rusqlite::Error>>>> {
        if last_day {
            let ld = self.last_day();
            let sql = "SELECT date_,item,price,amounts,category FROM pay where date_=?";
            let mut stmt = self.conn.as_ref().unwrap().prepare(sql)?;
            let pis = stmt
                .query_map([ld], |row| {
                    Ok(PayItem {
                        date: row.get(0)?,
                        item: row.get(1)?,
                        price: row.get(2)?,
                        amounts: row.get(3)?,
                        category: row.get(4)?,
                    })
                })?
                .collect::<Vec<_>>();
            return Ok(Some(pis));
        }
        if last_month {
            // 模糊查询 带有 月份的date
            // 2022-8,查找第5位开始为某些特定数值的值
            // _____{}%,8
            // e.g._____18%，查找第五位第六位为18的任意值
            let lm = self.last_month();
            let sql = format!(
                "SELECT date_,item,price,amounts,category FROM pay where date_ like '_____{}%'",
                lm
            );
            let mut stmt = self.conn.as_ref().unwrap().prepare(&sql)?;
            let pis = stmt
                .query_map([], |row| {
                    Ok(PayItem {
                        date: row.get(0)?,
                        item: row.get::<_, String>(1)?.into(),
                        price: row.get(2)?,
                        amounts: row.get::<_, u16>(3)?.into(),
                        category: row.get(4)?,
                    })
                })?
                .collect::<Vec<_>>();
            return Ok(Some(pis));
        }
        Ok(None)
    }
    /// read date file and get date of today
    ///
    ///then get last month by minus 1 from this month
    fn last_month(&self) -> String {
        let dt = Pay:: load_date();
        (dt.month - 1).to_string()
    }
    // return date of today
  pub  fn load_date() -> task::Date {
        let s = read_lastline_from_file("date.txt").unwrap();
        task::Date::load_date_from_str(s.trim())
    }
    /// read date file and get date of today
    ///
    ///then return date str of last day by minus 1 from today
    fn last_day(&self) -> String {
        let mut dt = Pay:: load_date();
        dt.set_day(dt.day - 1);
        dt.to_string()
    }
    /// used to test or remove records all
    fn drop_table(&self) -> Result<()> {
        let sql = "DROP TABLE pay;";
        self.conn.as_ref().unwrap().execute(sql, [])?;
        Ok(())
    }
    /// return payitems from db
    fn payitems(&self) {}
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
    pub fn clear_file(&self) {
        let path = format!("{}", self.payfile.as_ref().unwrap().display());
        let old_lines = read_alllines_from_file(&path);
        clear_contents(&path);
        for l in old_lines {
            if l.starts_with("#") {
                append_line_into_file(&path, l);
            }
        }
    }

    pub fn read_from_file(
        conn: Option< Connection>,
        path: &str,
        datestr: Option<String>,
    ) -> Result<Self> {
        let lines = read_alllines_from_file(path);
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

    pub fn set_conn(&mut self, conn: Option< Connection>) {
        self.conn = conn;
    }
}

#[derive(Debug, Default)]
pub struct PayItem {
    item: Option<String>,
    date: Option<String>,
    price: Option<f32>,
    amounts: Option<u16>,
    category: Option<String>,
}
impl Display for PayItem{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, price:{},amounts: {}, category: {},date: {}", self.item.as_ref().unwrap(), self.price.unwrap(),self.amounts.unwrap(),self.category.as_ref().unwrap(),self.date.as_ref().unwrap())
    }
}
impl PayItem {
    fn new(
        item: Option<String>,
        date: Option<String>,
        price: Option<f32>,
        amounts: Option<u16>,
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
    /// update date of pay db
    fn update_date(&self) {}
    /// impl from <string> ,separated by ','
    pub fn from_string(s: &str, date: Option<String>) -> Result<Self> {
        let r = s.split(',').collect::<Vec<&str>>();
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
            Ok(price.parse::<_>()?)
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
        let amounts: result::Result<_, TaskError> = if r.len() == 4 {
            let amounts = r.get(3).unwrap().trim();
            if amounts.is_empty() {
                Ok(0u16)
            } else {
                Ok(amounts.parse::<_>()?)
            }
        } else {
            Ok(0)
        };
        // : merge 2,3,4
        let len_of_fields = r.len();
        if len_of_fields == 2 || len_of_fields == 3 || len_of_fields == 4 {
            // means fields are respectly items,price，category,amounts
            return Ok(Self::new(
                Some(item?),
                date,
                Some(price?),
                Some(amounts?),
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

// from db
// #[test]
// fn test_drop_table() {
//     let s=Sqlite::new_conn("task.db").unwrap();
//     Pay::new(Some(&s.db), None, None).drop_table().unwrap();
// }