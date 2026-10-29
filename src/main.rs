mod waveform;
use bazauti::WAVParser;
use bazauti_cli::info::summary::summary;
use clap::{Parser, Subcommand};
use waveform::plot::WaveformRenderer;

#[derive(Subcommand, Debug, Clone)]
enum BazautiCliCommands {
    /// Get information about the audio data/files
    Info {
        /// Path to the audio file
        path: String,
    },
    /// Draw/Plot the waveform of the audio file.
    Waveform {
        /// Path to the audio file
        path: String,
    },
    /// Play the audio file
    Playback {
        /// Path to the audio file
        path: String,
    },
}

#[derive(Parser, Debug)]
#[command(name = "bazauti")]
#[command(version, about = "A Cli frontend for the bazauti audio library.", long_about=None)]
struct BazautiCliArgs {
    /// The Cli subcommand that detects the command to run under the cli.
    #[command(subcommand)]
    command: BazautiCliCommands,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = BazautiCliArgs::try_parse()?;
    match args.command {
        BazautiCliCommands::Info { path } => {
            let mut parser = WAVParser::new(&path);
            let _ = parser.parse()?;
            summary(&parser);
        }
        BazautiCliCommands::Waveform { path } => {
            let mut parser = WAVParser::new(&path);
            let _ = parser.parse()?;
            let renderer = WaveformRenderer::new(parser.context());
            renderer.render_text(0.0, 1.5)?;
            renderer.render(0.0, 1.5);
        }
        BazautiCliCommands::Playback { path: _ } => {}
    }
    Ok(())
}
