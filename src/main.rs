use std::{path::Path, sync::mpsc};

use clap::{Args, Parser, Subcommand};

mod renderbox;
mod shader;
mod window;

use notify::Watcher;
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
    /// GLSL version
    #[arg(default_value = "330")]
    glsl_version: String,
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
    /// Watch updates of FSHADER_PATH.
    #[arg(short = 'w')]
    watch_path: bool,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        CliCommands::Example => {
            let example_fshader = crate::shader::generate_shader(
                crate::shader::EXAMPLE_FRAGMENT_SHADER,
                &cli.glsl_version,
            );
            println!("{}", example_fshader);
        }
        CliCommands::Start(args) => {
            let fshader_path = args.fshader_path;
            let glsl_version = cli.glsl_version;

            let mut _file_event_watcher = None;
            let mut file_event_receiver = None;

            if args.watch_path {
                let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
                let mut watcher = notify::recommended_watcher(tx)
                    .expect("cannot create notify::recommended_watcher");
                watcher
                    .watch(
                        Path::new(&fshader_path),
                        notify::RecursiveMode::NonRecursive,
                    )
                    .expect("cannot start watcher");

                println!("start to watch a file = {}", &fshader_path);

                _file_event_watcher = Some(watcher);
                file_event_receiver = Some(rx);
            }

            let option = RenderboxOption {
                fshader_path,
                glsl_version,
                file_event_receiver,
            };
            Application::<RenderboxState, RenderboxOption>::start_app(option);
        }
    }
}
