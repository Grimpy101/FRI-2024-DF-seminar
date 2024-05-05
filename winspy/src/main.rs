use std::path::Path;

use argh::FromArgs;
use miette::Result;
use tracing_subscriber::EnvFilter;

use crate::{logging::initialize_tracing, reader::EventTranscriptReader};

mod detectors;
mod logging;
mod models;
mod reader;

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


    let sqlite_database_path = Path::new(&cmd_arguments.database_path);
    let mut database = EventTranscriptReader::new(sqlite_database_path).await?;

    for event in database.load_all_events().await? {
        println!("{:?}", event);
    }


    drop(guard);
    Ok(())
}
