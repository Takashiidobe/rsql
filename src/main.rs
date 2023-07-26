use std::collections::{BTreeMap, HashMap, HashSet};

use rsql::*;

pub type Columns = Vec<String>;
pub type Rows = Vec<String>;
pub type Table = Vec<Rows>;

fn main() {
    let sql = "SELECT id, name FROM users";

    let res = parse_sql(sql);

    let mut users = BTreeMap::default();

    users.insert(1, vec!["1", "Joe", "25"]);
    users.insert(2, vec!["2", "Jim", "30"]);
    users.insert(3, vec!["3", "Bob", "35"]);

    let mut tables: HashMap<String, BTreeMap<i32, Vec<&str>>> = HashMap::default();

    tables.insert("users".to_string(), users);

    let mut columns: HashMap<&str, Vec<&str>> = HashMap::default();
    columns.insert("users", vec!["id", "name", "age"]);

    match res {
        SqlQuery::Select(targets, table) => match targets {
            Fields::All => {
                if let Some(res) = tables.get(&table) {
                    println!("{:?}", res);
                }
            }
            Fields::Columns(columns) => {
                let column_indicies: HashSet<_> = columns.iter().enumerate().map(|c| c.0).collect();

                if let Some(res) = tables.get(&table) {
                    for value in res.values() {
                        for (index, val) in value.iter().enumerate() {
                            if column_indicies.contains(&index) {
                                print!("{} ", val);
                            }
                        }
                        println!();
                    }
                }
            }
        },
    }
}
