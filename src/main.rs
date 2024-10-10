//! Solana Feature Status CLI.

use {
    clap::{Parser, Subcommand},
    status_report::FeatureStatusReport,
    std::{fs::File, io::Write},
    tokio::process::Command,
};

mod emit;
mod status_report;

const EMIT: &str = "dir/emit.rs.txt";

enum Network {
    Devnet,
    Testnet,
    MainnetBeta,
}

impl Network {
    fn moniker(&self) -> &str {
        match self {
            Network::Devnet => "-ud",
            Network::Testnet => "-ut",
            Network::MainnetBeta => "-um",
        }
    }

    fn filename(&self) -> &str {
        match self {
            Network::Devnet => "dir/devnet.json",
            Network::Testnet => "dir/testnet.json",
            Network::MainnetBeta => "dir/mainnet-beta.json",
        }
    }

    async fn download_features(self: &Network) -> std::io::Result<()> {
        let solana_command = format!(
            "solana feature status --display-all --output json {}",
            self.moniker()
        );
        Command::new("sh")
            .arg("-c")
            .arg(&solana_command)
            .output()
            .await
            .and_then(|output| {
                let mut file = File::create(self.filename())?;
                file.write_all(&output.stdout)?;
                Ok(())
            })
    }
}

#[derive(Subcommand)]
enum SubCommand {
    Status {
        #[arg(short, long)]
        agave_version: String,
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        no_fetch: bool,
    },
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    pub command: SubCommand,
}

#[tokio::main]
async fn main() {
    match Cli::parse().command {
        SubCommand::Status {
            agave_version,
            no_fetch,
        } => {
            let devnet = Network::Devnet;
            let devnet_report = load_status_report(&devnet, no_fetch).await;

            let testnet = Network::Testnet;
            let testnet_report = load_status_report(&testnet, no_fetch).await;

            let mainnet_beta = Network::MainnetBeta;
            let mainnet_beta_report = load_status_report(&mainnet_beta, no_fetch).await;

            emit::emit(
                &agave_version,
                &[
                    (devnet, devnet_report),
                    (testnet, testnet_report),
                    (mainnet_beta, mainnet_beta_report),
                ],
                EMIT,
            )
            .await
            .unwrap();
        }
    }
}

async fn load_status_report(network: &Network, no_fetch: bool) -> FeatureStatusReport {
    if !no_fetch {
        network.download_features().await.unwrap();
    }
    FeatureStatusReport::from_json_file(network.filename()).unwrap()
}
