#![allow(unused_variables)]

use std::collections::{BTreeMap, HashMap, HashSet};

use color_eyre::Result;
use prettytable::Table;
use serde::ser;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{self, Ident, ObjectName, SelectItem, SetExpr, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::fs::File;
use std::io::prelude::*;

pub type TableName = String;
pub type QueryResult = Vec<Vec<SqlDataType>>;
pub type Columns = HashMap<String, Vec<String>>;
pub type Tables = HashMap<String, BTreeMap<i32, Vec<SqlDataType>>>;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SqlDataType {
    String(String),
    Usize(usize),
    Bool(bool),
    None,
}

impl Default for SqlDataType {
    fn default() -> Self {
        Self::Usize(0)
    }
}

use std::fmt;

impl fmt::Display for SqlDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlDataType::String(s) => write!(f, "{}", s),
            SqlDataType::Usize(num) => write!(f, "{}", num),
            SqlDataType::Bool(b) => write!(f, "{}", b),
            SqlDataType::None => write!(f, "Null"),
        }
    }
}

impl From<String> for SqlDataType {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for SqlDataType {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<bool> for SqlDataType {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<usize> for SqlDataType {
    fn from(value: usize) -> Self {
        Self::Usize(value)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SqlData {
    #[default]
    None,
    Some(SqlDataType),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Fields {
    #[default]
    All,
    Columns(Vec<String>),
}

#[derive(Debug)]
pub enum SqlQuery {
    Select(Fields, TableName),
}

impl Default for SqlQuery {
    fn default() -> Self {
        Self::Select(Fields::default(), String::default())
    }
}

pub fn get_result(res: SqlQuery, tables: &Tables, columns: &Columns) -> (Vec<String>, QueryResult) {
    let mut result = vec![];
    let mut fetched_columns = vec![];
    match res {
        SqlQuery::Select(targets, table) => match targets {
            Fields::All => {
                if let Some(res) = tables.get(&table) {
                    let values: Vec<_> = res
                        .values()
                        .map(|x| x.iter().map(|x| x.to_owned()).collect())
                        .collect();
                    result = values;
                    fetched_columns = columns
                        .get(&table)
                        .expect("column did not exist on table")
                        .to_owned();
                }
            }
            Fields::Columns(column) => {
                let columns_map: HashMap<_, _> = columns
                    .get(&table)
                    .unwrap()
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (c, i))
                    .collect();
                // get columns and column as set
                let columns: HashSet<String> = columns
                    .get(&table)
                    .unwrap()
                    .iter()
                    .map(|x| x.to_string())
                    .collect();

                let column: HashSet<String> = column.iter().map(|x| x.to_string()).collect();

                let column_intersection: HashSet<String> = columns
                    .intersection(&column)
                    .map(|x| x.to_string())
                    .collect();

                let mut intersection: HashSet<usize> = HashSet::default();

                for c in column_intersection {
                    let index = columns_map.get(&c.to_string()).unwrap();
                    fetched_columns.push(columns.get(&c.to_string()).unwrap().to_string());
                    intersection.insert(*index);
                }

                if let Some(res) = tables.get(&table) {
                    for value in res.values() {
                        let mut row = vec![];
                        for (index, val) in value.iter().enumerate() {
                            if intersection.contains(&index) {
                                row.push(val.clone());
                            }
                        }
                        result.push(row);
                    }
                }
            }
        },
    }

    (fetched_columns, result)
}

pub fn parse_sql(sql: &str) -> Vec<SqlQuery> {
    let dialect = GenericDialect {};

    let mut result = vec![];

    let ast = Parser::parse_sql(&dialect, sql).unwrap();
    for node in ast {
        let mut targets = Fields::default();
        let mut table = TableName::default();
        let mut fields = vec![];
        match node {
            Statement::Query(query) => {
                let body = query.body.clone();
                match *body {
                    SetExpr::Select(s) => {
                        let projections = s.projection;
                        let from = s.from;
                        // Implement Between, i.e. select * from table where id
                        // > 5;
                        if let Some(selection) = s.selection {
                            match selection {
                                ast::Expr::Between {
                                    expr,
                                    negated,
                                    low,
                                    high,
                                } => {}
                                _ => todo!(),
                            }
                        }
                        for t in from {
                            let a = t;
                            match a.relation {
                                ast::TableFactor::Table { name, .. } => match name {
                                    ObjectName(a) => {
                                        for name in a {
                                            let Ident { value, .. } = name;

                                            table = value;
                                        }
                                    }
                                },
                                _ => todo!(),
                            }
                        }
                        for projection in projections {
                            match projection {
                                SelectItem::UnnamedExpr(identifier) => match identifier {
                                    ast::Expr::Identifier(Ident { value, .. }) => {
                                        fields.push(value);
                                    }
                                    _ => todo!(),
                                },
                                SelectItem::Wildcard(_) => {
                                    targets = Fields::All;
                                }
                                _ => todo!(),
                            }
                        }
                    }
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }

        if !fields.is_empty() {
            result.push(SqlQuery::Select(Fields::Columns(fields), table))
        } else {
            result.push(SqlQuery::Select(targets, table))
        }
    }

    result
}

pub fn save_to_disk<T: ser::Serialize>(file_name: &str, tables: &T) -> Result<()> {
    let serialized_tables = bincode::serialize(&tables)?;
    let mut file = File::create(file_name)?;

    file.write_all(&serialized_tables)?;
    Ok(())
}

pub fn load_db(v: &mut Vec<u8>) -> Result<Tables> {
    let mut f = File::open("./data.db")?;
    f.read_to_end(v)?;

    Ok(bincode::deserialize(v)?)
}

pub fn load_columns(v: &mut Vec<u8>) -> Result<Columns> {
    let mut f = File::open("./columns.db")?;
    f.read_to_end(v)?;

    Ok(bincode::deserialize(v)?)
}

pub fn repl(tables: &Tables, columns: &Columns) -> Result<()> {
    use linefeed::{Interface, ReadResult};

    let reader = Interface::new("Rsql")?;

    reader.set_prompt("rsql> ")?;

    while let ReadResult::Input(input) = reader.read_line()? {
        let parsed_sql = parse_sql(&input);
        for parsed_sql_node in parsed_sql {
            let (fetched_columns, results) = get_result(parsed_sql_node, tables, columns);
            let mut table = Table::new();

            table.add_row(fetched_columns.into());

            for result in results {
                table.add_row(result.into());
            }

            table.printstd();
        }
    }

    Ok(())
}

pub fn load_data() -> (Tables, Columns) {
    let mut users = BTreeMap::default();

    users.insert(
        1,
        vec![
            SqlDataType::from(1),
            SqlDataType::from("Joe"),
            SqlDataType::from(25),
            SqlDataType::None,
        ],
    );
    users.insert(
        2,
        vec![
            SqlDataType::from(2),
            SqlDataType::from("Jim"),
            SqlDataType::from(30),
            SqlDataType::None,
        ],
    );
    users.insert(
        3,
        vec![
            SqlDataType::from(3),
            SqlDataType::from("Bob"),
            SqlDataType::from(35),
            SqlDataType::None,
        ],
    );
    users.insert(
        4,
        vec![
            SqlDataType::from(4),
            SqlDataType::from("Jom"),
            SqlDataType::from(40),
            SqlDataType::Bool(true),
        ],
    );
    users.insert(
        5,
        vec![
            SqlDataType::from(5),
            SqlDataType::from("Boy"),
            SqlDataType::from(45),
            SqlDataType::Bool(false),
        ],
    );

    let mut restaurants = BTreeMap::default();

    restaurants.insert(
        1,
        vec![SqlDataType::from(1), SqlDataType::from("Soup Shack")],
    );
    restaurants.insert(
        2,
        vec![SqlDataType::from(2), SqlDataType::from("Hei La Moon")],
    );
    restaurants.insert(
        3,
        vec![SqlDataType::from(3), SqlDataType::from("Maruichi Select")],
    );
    restaurants.insert(
        4,
        vec![SqlDataType::from(4), SqlDataType::from("Shake Shack")],
    );

    let mut tables: Tables = HashMap::default();

    tables.insert("users".to_string(), users);
    tables.insert("restaurants".to_string(), restaurants);

    let mut columns: Columns = HashMap::default();
    columns.insert(
        "users".to_string(),
        vec![
            "id".to_string(),
            "name".to_string(),
            "age".to_string(),
            "is_person".to_string(),
        ],
    );
    columns.insert(
        "restaurants".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    (tables, columns)
}
