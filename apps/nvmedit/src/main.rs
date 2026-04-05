use amber_connect::{
    codec::{PbReceiver, PbSocketError},
    control,
};
use anyhow::{Context, anyhow};
use clap::{Parser, Subcommand};
use proto::sensor::{
    self,
    nvm::{self, Parameters},
};
use std::{
    io::{Read, Write},
    time::Duration,
};
use tokio::time::timeout;
use zeromq::{Socket, SubSocket};

use crate::user::UserParameters;

mod user;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    command: Command,

    #[arg(short, long)]
    out: Option<String>,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    Read,
    Write { file: String },
    ResetCamera,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut control = control::Client::try_acquire("nvmedit")
        .await
        .context("Failed to acquire exlusive control")?;

    let mut status_socket = SubSocket::new();
    status_socket.connect(amber_connect::endpoint::STATUS).await?;
    status_socket.subscribe("").await?;

    let r = tokio::select! {
        r = run_operation(&mut control, &mut status_socket, args) => r,
        _ = tokio::signal::ctrl_c() => Err(anyhow!("Interrupted"))
    };

    let mut reset = sensor::Command::default();
    reset.set_action(sensor::Action::Monitor);

    let _ = control.send(reset).await;
    control.release().await;

    r
}

async fn run_operation(control: &mut control::Client, status: &mut SubSocket, args: Args) -> anyhow::Result<()> {
    const TIMEOUT: Duration = Duration::from_secs(1);

    let mut cmd = sensor::Command::default();
    cmd.set_action(sensor::Action::Monitor);
    control.send(cmd).await?;
    timeout(TIMEOUT, wait_until(status, |s| s.state() != sensor::State::Manual))
        .await
        .context("Timed out trying to reset the sensor")??;

    let mut cmd = sensor::Command::default();
    cmd.set_action(sensor::Action::Manual);
    control.send(cmd).await?;
    timeout(TIMEOUT, wait_until(status, |s| s.state() == sensor::State::Manual))
        .await
        .context("Timed out trying to enter manual mode")??;

    let mut cmd = sensor::Command::default();
    let nvm_cmd = cmd.nvm.get_or_insert_default();
    match args.command {
        Command::Read => nvm_cmd.set_action(nvm::Action::Read),
        Command::Write { file } => {
            nvm_cmd.set_action(nvm::Action::Write);

            let new_user_params: UserParameters = read_file(file)?;
            let new_params: Parameters = new_user_params.try_into()?;
            nvm_cmd.new_parameters = Some(new_params);
        }
        Command::ResetCamera => cmd.nvm.get_or_insert_default().set_action(nvm::Action::ResetCamera),
    }

    control.send(cmd).await?;
    let new_params = timeout(
        TIMEOUT,
        wait_until(status, |s| {
            s.nvm.as_ref().is_some_and(|n| n.current_parameters.is_some())
        }),
    )
    .await
    .context("Timed out trying to reset parameters")??;

    let mut cmd = sensor::Command::default();
    cmd.set_action(sensor::Action::Monitor);
    control.send(cmd).await?;
    timeout(TIMEOUT, wait_until(status, |s| s.state() != sensor::State::Manual))
        .await
        .context("Timed out trying to reset the sensor")??;

    let new_params = new_params.nvm.unwrap().current_parameters.unwrap();
    let new_params: UserParameters = new_params.into();

    if let Some(out_file) = args.out {
        let mut file = std::fs::File::create(&out_file)?;
        let contents = serde_json::to_string_pretty(&new_params)?;
        let _ = file.write(contents.as_bytes())?;
        println!("Sensor parameters saved to {out_file}");
    } else {
        let json = serde_json::to_string_pretty(&new_params)?;
        println!("{json}");
    }

    Ok(())
}

/// Return the first status meeting `condition`. Drop all statuses until then.
async fn wait_until<S: PbReceiver>(
    socket: &mut S,
    condition: impl Fn(&sensor::Status) -> bool,
) -> Result<sensor::Status, PbSocketError> {
    loop {
        let msg = socket.recv_msg::<sensor::Status>().await?;

        if condition(&msg) {
            return Ok(msg);
        }
    }
}

fn read_file(file: String) -> anyhow::Result<UserParameters> {
    let mut file = std::fs::File::open(file)?;
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents)?;

    serde_json::from_str(&contents).map_err(anyhow::Error::from)
}
