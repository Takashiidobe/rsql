use color_eyre::eyre::Result;

use rsql::*;

fn main() -> Result<()> {
    let mut v = vec![];
    let mut cv = vec![];

    let tables = load_db(&mut v)?;
    let columns = load_columns(&mut cv)?;

    // let (tables, columns) = load_data();

    repl(&tables, &columns)?;

    println!("Shutting down.");

    save_to_disk("columns.db", &columns)?;
    save_to_disk("data.db", &tables)?;

    println!("Successfully saved data to disk.");

    Ok(())
}
