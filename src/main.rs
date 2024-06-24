use std::path::PathBuf;

use clap::{Parser, Subcommand};
use essential_types::{
    intent::{Intent, SignedSet},
    solution::Solution,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Which solution data index to debug
    #[arg(short, long, default_value_t = 0)]
    solution_data_index: usize,
    /// Which intent to debug
    #[arg(short, long, default_value_t = 0)]
    intent_index: usize,
    /// Which constraint to debug
    #[arg(short, long, default_value_t = 0)]
    constraint_index: usize,
    /// Path to the solution file encoded in JSON
    solution: PathBuf,
    /// Select a subcommand to run
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Intent {
        /// Path to the individual intent file encoded in JSON
        intent: PathBuf,
    },
    Intents {
        /// Path to a list of intents file encoded in JSON
        intents: PathBuf,
    },
    SignedIntent {
        /// Path to a signed intents file encoded in JSON
        intents: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    if let Err(e) = run(args).await {
        eprintln!("Command failed because: {}", e);
    }
}

async fn run(args: Cli) -> anyhow::Result<()> {
    // TODO state.
    let Cli {
        solution_data_index,
        intent_index,
        constraint_index,
        solution,
        command,
    } = args;
    let intents: Vec<Intent> = match command {
        Command::Intent { intent } => {
            vec![serde_json::from_slice(&tokio::fs::read(intent).await?)?]
        }
        Command::Intents { intents } => serde_json::from_slice(&tokio::fs::read(intents).await?)?,
        Command::SignedIntent { intents } => {
            let intents: SignedSet = serde_json::from_slice(&tokio::fs::read(intents).await?)?;
            intents.set
        }
    };

    let solution: Solution = serde_json::from_slice(&tokio::fs::read(solution).await?)?;
    let intent = intents
        .get(intent_index)
        .ok_or_else(|| anyhow::anyhow!("Intent not found"))?
        .clone();
    essential_debugger::run(
        solution,
        solution_data_index as u16,
        intent,
        constraint_index,
        Default::default(),
    )
    .await
}
