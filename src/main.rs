use std::path::PathBuf;

use clap::{Parser, Subcommand};
use essential_types::{
    contract::{Contract, SignedContract},
    predicate::Predicate,
    solution::Solution,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Which solution data index to debug
    #[arg(short, long, default_value_t = 0)]
    solution_data_index: usize,
    /// Which predicate to debug
    #[arg(short, long, default_value_t = 0)]
    predicate_index: usize,
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
    Predicate {
        /// Path to the individual predicate file encoded in JSON
        predicate: PathBuf,
    },
    Contract {
        /// Path to a contract file encoded in JSON
        contract: PathBuf,
    },
    SignedContract {
        /// Path to a signed contract file encoded in JSON
        contract: PathBuf,
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
        predicate_index,
        constraint_index,
        solution,
        command,
    } = args;
    let predicates: Vec<Predicate> = match command {
        Command::Predicate { predicate } => {
            vec![serde_json::from_slice(&tokio::fs::read(predicate).await?)?]
        }
        Command::Contract { contract } => {
            let contract: Contract = serde_json::from_slice(&tokio::fs::read(contract).await?)?;
            contract.predicates
        }
        Command::SignedContract { contract } => {
            let contract: SignedContract =
                serde_json::from_slice(&tokio::fs::read(contract).await?)?;
            contract.contract.predicates
        }
    };

    let solution: Solution = serde_json::from_slice(&tokio::fs::read(solution).await?)?;
    let predicate = predicates
        .get(predicate_index)
        .ok_or_else(|| anyhow::anyhow!("Predicate not found"))?
        .clone();
    essential_debugger::run(
        solution,
        solution_data_index as u16,
        predicate,
        constraint_index,
        Default::default(),
    )
    .await
}
