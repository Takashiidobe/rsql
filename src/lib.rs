#![allow(unused_variables)]

use sqlparser::ast::Ident;
use sqlparser::ast::ObjectName;
use sqlparser::ast::SelectItem;
use sqlparser::ast::Statement;
use sqlparser::ast::{self, SetExpr};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

pub type Table = String;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Fields {
    #[default]
    All,
    Columns(Vec<String>),
}

#[derive(Debug)]
pub enum SqlQuery {
    Select(Fields, Table),
}

impl Default for SqlQuery {
    fn default() -> Self {
        Self::Select(Fields::default(), String::default())
    }
}

pub fn parse_sql(sql: &str) -> SqlQuery {
    let dialect = GenericDialect {}; // or AnsiDialect

    let mut targets = Fields::default();
    let mut table = Table::default();

    let ast = Parser::parse_sql(&dialect, sql).unwrap();
    match ast.first().unwrap() {
        Statement::Query(query) => {
            let body = query.body.clone();
            match *body {
                SetExpr::Select(s) => {
                    let s = s.clone();
                    let projections = s.projection;
                    let from = s.from;
                    for t in from {
                        match t {
                            a => match a.relation {
                                ast::TableFactor::Table {
                                    name,
                                    alias,
                                    args,
                                    with_hints,
                                } => match name {
                                    ObjectName(a) => {
                                        for name in a {
                                            let Ident { value, .. } = name;

                                            table = value;
                                        }
                                    }
                                },
                                _ => todo!(),
                            },
                        }
                    }
                    for projection in projections {
                        match projection {
                            SelectItem::UnnamedExpr(u) => {
                                dbg!(u);
                            }
                            SelectItem::ExprWithAlias { expr, alias } => {
                                dbg!(expr, alias);
                            }
                            SelectItem::QualifiedWildcard(a, b) => {
                                dbg!(a, b);
                            }
                            SelectItem::Wildcard(_) => {
                                targets = Fields::All;
                            }
                        }
                    }
                }
                SetExpr::Query(_) => todo!(),
                SetExpr::SetOperation {
                    op,
                    set_quantifier,
                    left,
                    right,
                } => todo!(),
                SetExpr::Values(q) => todo!(),
                SetExpr::Insert(q) => todo!(),
                SetExpr::Update(q) => {
                    dbg!(q);
                }
                SetExpr::Table(q) => {
                    dbg!(q);
                }
            }
        }
        _ => todo!(),
    }

    SqlQuery::Select(targets, table)
}
