use crate::app::App;
use clap::{Arg, Command};
use color_eyre::Result;
use std::error::Error;

mod app;
mod events;
mod h5_gen;
mod h5_utils;
mod tree;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    crate::h5_gen::generate_dummy_file()?;
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

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App::new(h5_file_path)?;
    let _ = app.run(terminal).await;

    ratatui::restore();
    Ok(())
}
