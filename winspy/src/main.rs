use std::path::Path;

use argh::FromArgs;
use miette::Result;
use models::EventDatabase;

mod detectors;
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

    let mut database = EventDatabase::new(database_path).await?;
    let events = database.load_all_events().await?;

    for event in events.iter() {
        println!("{:?}", event);
    }

    println!("\n-------------------\nRetrieved {} events", events.len());

    Ok(())
}
