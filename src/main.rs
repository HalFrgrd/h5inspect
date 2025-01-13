use crate::app::App;
use clap::{Arg, Command};
use color_eyre::Result;
use std::error::Error;

use ratatui;
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

fn build_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}

fn run(h5_file_path: std::path::PathBuf) -> Result<(), Box<dyn Error>> {
    tui_logger::init_logger(log::LevelFilter::Trace)?;
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let app = App::new(h5_file_path);

    color_eyre::install()?;
    let terminal = ratatui::init();

    let runtime = build_runtime();

    runtime.block_on(async {
        let _ = app.run(terminal).await;
    });

    ratatui::restore();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_startup() -> Result<(), Box<dyn Error>> {
        h5_utils::generate_dummy_file()?;
        let h5_file_path = std::path::PathBuf::from("dummy.h5");
        run_app(h5_file_path)
    }

    #[test]
    #[should_panic(expected = "unable to open file: unable to open file")]
    fn test_app_on_non_existent_file() {
        let h5_file_path = std::path::PathBuf::from("non_existent.h5");
        run_app(h5_file_path).unwrap();
    }

    #[test]
    #[should_panic(expected = "unable to open file: file signature not found")]
    fn test_app_on_non_h5_file() {
        let h5_file_path = std::path::PathBuf::from("src/main.rs");
        run_app(h5_file_path).unwrap();
    }

    #[test]
    fn test_app_on_split_file() -> Result<(), Box<dyn Error>> {
        h5_utils::generate_dummy_split_file()?;
        let h5_file_path = std::path::PathBuf::from("dummy_split.h5");
        run_app(h5_file_path)
    }

    fn run_app(h5_file_path: std::path::PathBuf) -> Result<(), Box<dyn Error>> {
        let app = App::new(h5_file_path);

        let backend = ratatui::backend::TestBackend::new(200, 120);
        let terminal = ratatui::Terminal::new(backend).unwrap();

        let runtime = build_runtime();

        runtime.block_on(async {
            tokio::select! {
                res = app.run(terminal) => {
                    res.unwrap();
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
                    println!("Timer expired before app returned, nice.");
                }
            }
        });

        Ok(())
    }
}
