use std::path::Path;

use argh::FromArgs;
use miette::Result;
use models::EventReader;
use tracing_subscriber::EnvFilter;

use crate::logging::initialize_tracing;

mod detectors;
mod logging;
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
    let guard = initialize_tracing(
        EnvFilter::from_default_env(),
        EnvFilter::from_default_env(),
        "logs",
        "df-winspy",
    )?;

    let cmd_arguments: CmdArguments = argh::from_env();


    let database_path = Path::new(&cmd_arguments.database_path);

    let mut database = EventReader::new(database_path).await?;
    let events = database.load_all_events().await?;

    for event in events.iter() {
        println!("{:?}", event);
    }

    println!(
        "\n-------------------\nRetrieved {} events",
        events.len()
    );

    drop(guard);
    Ok(())
}
