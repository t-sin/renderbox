mod renderbox;
mod window;

use renderbox::{RenderboxOption, RenderboxState};
use window::Application;

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 840;

// a png file for GIMP splash screen
const IMAGE_PATH: &str = "/usr/share/gimp/3.0/images/gimp-splash.png";

fn main() {
    let option = RenderboxOption {
        image_path: IMAGE_PATH.into(),
    };

    Application::<RenderboxState, RenderboxOption>::start_app(option);
}
