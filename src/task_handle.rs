use crate::{
    error::Result,
    file_op::{clear_contents, open_as_append, open_as_read},
    task,
};
use regex::RegexBuilder;
use std::{
    fmt::Display,
    io::{Read, Write},
};
/// move root tasks with all branch tasks marked from `today`(from) task file to `all`(to) task file
pub(crate) fn move_task(from: &str, to: &str) -> Result<()> {
    let ts = capture_task(from)?;
    let roots = ts.iter().map(|e| RootTask::from_str(e)).collect::<Vec<_>>();
    let roots_marked = roots_marked(&roots);
    let roots_not_marked = roots_not_marked(&roots);

    let mut f = open_as_append(to)?;
    //  write date first
    let mut date = task::Date::new();
    date.load_set_date()?;
    f.write(format!("date {} {}\n", date.to_string(), date.weekday()).as_bytes())?;

    for r in roots_marked {
        f.write(format!("{}\n", r.to_string()).as_bytes())?;
    }

    clear_contents(from)?;
    let mut fr = open_as_append(from)?;
    for r in roots_not_marked {
        fr.write(format!("{}\n", r.to_string()).as_bytes())?;
    }
    Ok(())
}
/// return root tasks whose inner branch tasks are marked either OK or BAD.
pub(crate) fn roots_marked(roots: &Vec<RootTask>) -> Vec<&RootTask> {
    roots.iter().filter(|e| e.all_marked()).collect::<Vec<_>>()
}
/// return root tasks whose inner branch tasks are either partly marked or not marked altogether.
pub(crate) fn roots_not_marked(roots: &Vec<RootTask>) -> Vec<&RootTask> {
    roots.iter().filter(|e| !e.all_marked()).collect::<Vec<_>>()
}
pub(crate) fn capture_task(fpatth: &str) -> Result<Vec<String>> {
    let mut f = open_as_read(fpatth)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let re = RegexBuilder::new(r"\[(?P<task>.*?)\]")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()?;

    let ts = re
        .captures_iter(&buf)
        .map(|c| c["task"].trim().to_string())
        .collect::<Vec<_>>();
    Ok(ts)
}
#[derive(Debug, Default, std::cmp::PartialEq)]
pub(crate) struct RootTask {
    item: String,
    branch: Vec<Branch>,
}
impl Display for RootTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let bs = self.branch();
        let bs_marked = bs.iter().filter(|b| b.is_marked()).collect::<Vec<_>>();
        let bs_not_marked = bs.iter().filter(|b| !b.is_marked()).collect::<Vec<_>>();
        let mut n = 0;
        let mut b_str = String::new();
        for b in bs_not_marked {
            n += 1;
            b_str.push_str(&format!("    {}.{}\n", n, b.to_string()));
        }
        for b in bs_marked {
            n += 1;
            b_str.push_str(&format!("    {}.{}\n", n, b.to_string()));
        }
        let format = format!(
            "[
    {}{{
{}
        }}    
]",
            self.item, b_str
        );

        s.push_str(&format);

        write!(f, "{}", s)
    }
}
impl RootTask {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_item(&mut self, item: String) {
        self.item = item;
    }

    /// check if all branch tasks are marked.
    pub(crate) fn all_marked(&self) -> bool {
        let bs = self.branch();
        bs.iter().all(|b| b.is_marked())
    }
    ///#steps:
    /// 1. split root task at `{`
    ///
    /// 2. parse branch tasks
    /// # example.
    /// a sample root task
    /// ```
    ///   task1{
    /// 1.23 OK
    /// 2.32
    /// }
    /// ```
    pub(crate) fn from_str(s: &str) -> Self {
        let v = s.split("{").collect::<Vec<_>>();
        let root_item = v.get(0).as_ref().unwrap().to_string();
        let b = v.get(1).as_ref().unwrap().to_string();

        let mut root = RootTask::new();
        let mut bs = vec![];
        for l in b
            .lines()
            .filter(|e| !e.trim().is_empty())
            .filter(|e| !e.contains("}"))
        {
            let branch = Branch::from_str(l.trim());
            bs.push(branch);
        }
        root.set_item(root_item);
        root.set_branch(bs);
        root
    }

    pub(crate) fn set_branch(&mut self, branch: Vec<Branch>) {
        self.branch = branch;
    }

    pub(crate) fn branch(&self) -> &[Branch] {
        self.branch.as_ref()
    }
}

#[derive(Debug, Default, std::cmp::PartialEq)]
pub(crate) struct Branch {
    item: String,
    marked: String,
}
impl Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}    {}", self.item.trim(), self.marked.trim())
    }
}
impl Branch {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    fn is_marked(&self) -> bool {
        if self.marked.contains("OK") || self.marked.contains("BAD") {
            true
        } else {
            false
        }
    }
    ///# steps:
    /// 1. remove prefix serial number by splitting at `.`
    ///
    /// 2. get branch item and marked info by splitting at `" "`
    ///  # example.
    /// a sample branch string
    /// ```
    /// 1.b1 OK
    /// 2.b2
    /// 3. b3 BAD
    /// ```
    fn from_str(line: &str) -> Self {
        let v = line.split(".").collect::<Vec<_>>();
        // remove prefix serial number
        let s_without_prefix = v[1..].join(".").trim().to_string();
        let vn = s_without_prefix.split(" ").collect::<Vec<_>>();
        let mark = vn.last();
        let m = mark.as_ref().unwrap();
        let mut b = Branch::new();
        if m.contains("BAD") || m.contains("OK") {
            b.set_item(vn[..vn.len() - 1].join(" "));
            b.set_marked(m.to_string());
        } else {
            b.set_item(s_without_prefix.to_string());
            b.set_marked("".into());
        }

        b
    }

    pub(crate) fn set_item(&mut self, item: String) {
        self.item = item;
    }

    pub(crate) fn set_marked(&mut self, marked: String) {
        self.marked = marked;
    }
}

#[test]
fn test_regex_multiline() {
    use regex::RegexBuilder;
    let text = "[
        task1{
            1.b1     OK
            2.b2
        }
    ]";
    let re = RegexBuilder::new(r"\[(?P<task>.*?)\]")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();
    let ts = re.captures(text).map(|c| c["task"].trim().to_string());
    let root_from_str = RootTask::from_str(&ts.as_ref().unwrap());

    let mut b1 = Branch::new();
    b1.set_item("b1".into());
    b1.set_marked("OK".into());
    let mut b2 = Branch::new();
    b2.set_item("b2".into());
    b2.set_marked("".into());
    let mut root = RootTask::new();
    root.set_item("task1".into());
    root.set_branch(vec![b1, b2]);

    assert_eq!(root_from_str, root)
}
#[test]
fn test_root_string() {
    // [
    //     task1{
    //     1.b2
    //     2.b1    OK

    //     }
    //     ]
    let mut b1 = Branch::new();
    b1.set_item("b1".into());
    b1.set_marked("OK".into());
    let mut b2 = Branch::new();
    b2.set_item("b2".into());
    b2.set_marked("".into());
    let mut root = RootTask::new();
    root.set_item("task1".into());
    root.set_branch(vec![b1, b2]);
}
