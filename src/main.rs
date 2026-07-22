use std::os::unix::prelude::*;
use std::path::PathBuf;

use clap::Parser;

mod backend;
mod config;

#[derive(Debug, Clone, Parser)]
#[command(trailing_var_arg = true)]
struct Args {
    #[arg(long, default_value = "~/.config/seolgi/config.yaml")]
    config: PathBuf,
    #[arg(short, long, default_value = "false")]
    verbose: bool,
    #[arg(num_args = 0.., allow_hyphen_values = true)]
    command: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(
                match args.verbose {
                    true => "seolgi=debug".to_string(),
                    false => "seolgi=info".to_string(),
                }
                .parse()?,
            ),
        )
        .init();

    tracing::debug!(
        "Initializing sandbox environment with config: {}",
        args.config.to_string_lossy()
    );

    let config = config::Config::load(&args.config).map_err(|e| {
        tracing::error!("Failed to load config! {e:?}");
        e
    })?;

    if args.command.is_empty() {
        tracing::error!("Command not specified (`--` followed by command)");
        return Err(anyhow::anyhow!("Command not specified"));
    }

    let mut cmd = std::process::Command::new(&args.command[0]);
    cmd.args(&args.command[1..]);

    #[cfg(feature = "landlock")]
    {
        let backend = backend::landlock::LandlockBackend::new(config.landlock);
        backend.sandbox_cmd(&mut cmd)?;
    }

    // Exec into the command.
    tracing::debug!("Executing command: {:?}", cmd);
    let err = cmd.exec();
    tracing::error!("Failed to exec command! {err:?}");

    Err(err.into())
}
