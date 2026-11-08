use clap::Parser;

mod renderbox;
mod window;

use renderbox::{RenderboxOption, RenderboxState};
use window::Application;

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 840;
const WINDOW_TITLE: &str = "Renderbox";

#[derive(Parser)]
#[command(version)]
#[command(about = "Renderbox is a shadertoy like playground application.")]
struct Args {
    image_path: String,
}

fn main() {
    let args = Args::parse();
    let option = RenderboxOption {
        image_path: args.image_path,
    };

    Application::<RenderboxState, RenderboxOption>::start_app(option);
}
