use std::collections::{BTreeMap, HashMap};

use rsql::*;

pub type Columns = Vec<String>;
pub type Rows = Vec<String>;
pub type Table = Vec<Rows>;

fn main() {
    let sql = "SELECT * FROM users";

    let res = parse_sql(sql);

    let mut users = BTreeMap::default();

    users.insert(1, vec!["Takashi", "27"]);

    let mut tables: HashMap<String, BTreeMap<i32, Vec<&str>>> = HashMap::default();

    tables.insert("users".to_string(), users);

    let mut columns: HashMap<&str, Vec<&str>> = HashMap::default();
    columns.insert("users", vec!["name", "age"]);

    match res {
        SqlQuery::Select(targets, table) => match targets {
            Fields::All => {
                if let Some(res) = tables.get(&table) {
                    println!("{:?}", res);
                }
            }
            Fields::Columns(columns) => {
                for column in columns {
                    println!("{}", column);
                }
            }
        },
    }
}
