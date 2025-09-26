use crate::app::App;
use clap::{Arg, Command};
use color_eyre::Result;
use std::error::Error;

use ratatui;
use tui_logger;

mod analysis;
mod app;
mod events;
mod h5_utils;
mod hist_plot;
mod num_utils;
mod tree;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    crate::h5_utils::generate_dummy_file()?;
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
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    let h5_file_name: &String = matches.get_one("h5file").expect("h5file is required");
    let h5_file_path = std::path::PathBuf::from(h5_file_name);
    tui_logger::init_logger(log::LevelFilter::Trace)?;
    tui_logger::set_default_level(log::LevelFilter::Trace);
    tui_logger::set_level_for_target("plotters_ratatui_backend::widget", log::LevelFilter::Off);
    tui_logger::set_level_for_target("mio::poll", log::LevelFilter::Off);
    log::info!("Starting app");

    let app = App::new(h5_file_path);
    let runtime = build_runtime();

    color_eyre::install()?;
    let terminal = ratatui::init();
    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;
    let res = runtime.block_on(app.run(terminal));
    crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)?;
    ratatui::restore();
    res
}

fn build_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_app_startup() -> Result<(), Box<dyn Error>> {
        h5_utils::generate_dummy_file()?;
        let h5_file_path = std::path::PathBuf::from("dummy.h5");
        run_app(h5_file_path)
    }

    #[test]
    #[should_panic(expected = "File path doesn't exist")]
    fn run_app_on_non_existent_file() {
        let h5_file_path = std::path::PathBuf::from("non_existent.h5");
        run_app(h5_file_path).unwrap();
    }

    #[test]
    #[should_panic(expected = "Couldn't open file")]
    fn run_app_on_non_h5_file() {
        let h5_file_path = std::path::PathBuf::from("src/main.rs");
        run_app(h5_file_path).unwrap();
    }

    #[test]
    fn run_app_on_split_file() -> Result<(), Box<dyn Error>> {
        h5_utils::generate_dummy_split_file()?;
        let h5_file_path = std::path::PathBuf::from("dummy_split.h5");
        run_app(h5_file_path)
    }

    fn run_app(h5_file_path: std::path::PathBuf) -> Result<(), Box<dyn Error>> {
        let app = App::new(h5_file_path);

        let backend = ratatui::backend::TestBackend::new(200, 120);
        let terminal = ratatui::Terminal::new(backend).unwrap();

        let runtime = build_runtime();
        let res = runtime.block_on(async {
            tokio::select! {
                res = app.run(terminal) => {
                    res
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
                    println!("Timer expired before app returned, nice.");
                    Ok(())
                }
            }
        });

        res
    }
}
