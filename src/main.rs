#![windows_subsystem = "windows"]

use druid::widget::prelude::*;
use druid::{AppLauncher, WindowDesc, piet};

mod lib;
use lib::canvas::{FractalBuilder, FractalCanvas};

const WINDOW_WIDTH: usize = 16*60;
const WINDOW_HEIGHT: usize = 9*60;

pub fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget)
        .title("Dimers")
        .window_size((WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64));

	let initial_state = ();

    // start the application. Here we pass in the application state.
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<()> {
    let builder = FractalBuilder::new(WINDOW_WIDTH, WINDOW_HEIGHT, piet::ImageFormat::RgbaSeparate);
	FractalCanvas::new(builder)
}
