use std::{error::Error};
use color_eyre::Result;

mod app;
mod ui;
use crate::app::App;


mod h5_utils;

fn main() -> Result<(), Box<dyn Error>> {
    crate::h5_utils::read_hdf5()?;

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();

    Ok(())
}
