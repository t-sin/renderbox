use clap::{Args, Parser, Subcommand};

mod renderbox;
mod shader;
mod window;

use renderbox::{RenderboxOption, RenderboxState};
use window::Application;

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 840;
const WINDOW_TITLE: &str = "Renderbox";

#[derive(Parser)]
#[command(version)]
#[command(about = "Renderbox is a shadertoy like playground application.")]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Subcommand)]
enum CliCommands {
    /// Start Renderbox with a shader in FSHADER_PATH.
    Start(StartArgs),
    /// Show example fragment shader code.
    Example,
}

#[derive(Args)]
struct StartArgs {
    /// A path to the file includes fragment shader code.
    fshader_path: String,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        CliCommands::Example => {
            println!("{}", crate::shader::EXAMPLE_FRAGMENT_SHADER);
        }
        CliCommands::Start(args) => {
            let option = RenderboxOption {
                fshader_path: args.fshader_path,
            };
            Application::<RenderboxState, RenderboxOption>::start_app(option);
        }
    }
}
