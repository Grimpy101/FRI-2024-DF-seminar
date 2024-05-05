use std::path::Path;

use argh::FromArgs;
use detectors::{AllDetectors, EventTranscriptProcessor};
use miette::{Context, Result};
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

    let cli_arguments: CmdArguments = argh::from_env();


    let sqlite_database_path = Path::new(&cli_arguments.database_path);
    let database = EventTranscriptReader::new(sqlite_database_path)
        .await
        .wrap_err("Failed to initialize EventTranscriptReader.")?;

    // for event in database.load_all_events().await? {
    //     println!("{:?}", event);
    // }

    let processor = EventTranscriptProcessor::new_from_event_transcript_reader(database)
        .await
        .wrap_err("Failed to initialize EventTranscriptProcessor.")?;

    let all_detectors = AllDetectors::new();
    let processed_events = processor.process_events(all_detectors);

    for processed_event in processed_events {
        println!("{processed_event:?}");
        println!();
    }


    drop(guard);
    Ok(())
}
