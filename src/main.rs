use crate::app::App;
use clap::{Arg, Command};
use color_eyre::Result;
use std::error::Error;

use tui_logger;

mod app;
mod events;
mod h5_utils;
mod tree;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    // crate::h5_utils::generate_dummy_file()?;
    // crate::h5_utils::generate_dummy_split_file()?;
    
    let matches = Command::new("h5inspect")
    .author("Hal Frigaard")
    .about("Simple TUI to inspect h5 files")
    .arg(
        Arg::new("h5file")
        .value_name("FILE")
        .help("Name of hdf5 file to inspect")
        .value_hint(clap::ValueHint::FilePath)
        .required(true),
    )
    .get_matches();

    let h5_file_name: &String = matches.get_one("h5file").expect("h5file is required");
    let h5_file_path = std::path::PathBuf::from(h5_file_name);
    run(h5_file_path)
}


fn run(h5_file_path: std::path::PathBuf) -> Result<(), Box<dyn Error>> {
    tui_logger::init_logger(log::LevelFilter::Trace)?;
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let app = App::new(h5_file_path)?;

    color_eyre::install()?;
    let terminal = ratatui::init();

    // #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let _ = app.run(terminal).await;
    });

    ratatui::restore();
    Ok(())
}