mod serve;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "turborepo-server")]
#[command(about = "A fictional versioning CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Serve(crate::serve::Serve),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("error"));

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(serve) => serve.run().await?,
    }

    Ok(())
}
