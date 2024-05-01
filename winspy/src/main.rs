use std::path::Path;

use argh::FromArgs;
use miette::{miette, Result};

mod models;

#[derive(FromArgs)]
/// A simple Windows 10/11 event parser and vizualizer
pub struct CmdArguments {
    /// path to the EventTrancript.db
    #[argh(option, short = 'i')]
    pub database_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmd_arguments: CmdArguments = argh::from_env();

    let database_path = Path::new(&cmd_arguments.database_path);
    if !database_path.exists() {
        return Err(miette!(
            "Provided path (`{}`) does not exist",
            cmd_arguments.database_path
        ));
    }

    if !database_path.is_file() {
        return Err(miette!(
            "Provided path (`{}`) is not a valid file",
            cmd_arguments.database_path
        ));
    }

    //let database = EventDatabase::events_from_file(&cmd_arguments.database_path).await?;
    //database.print_detected_events();

    //println!("{:?}", database);

    Ok(())
}
