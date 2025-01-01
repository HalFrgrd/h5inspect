use crate::app::App;
use color_eyre::Result;
use std::error::Error;

mod app;
mod h5_utils;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    crate::h5_utils::read_hdf5()?;

    color_eyre::install()?;
    let terminal = ratatui::init();
    let _ = App::new().run(terminal);
    ratatui::restore();

    Ok(())
}
