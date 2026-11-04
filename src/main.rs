mod renderbox;
mod window;

use renderbox::RenderboxState;
use window::Application;

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 840;

fn main() {
    Application::<RenderboxState>::start_app();
}
