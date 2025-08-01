/*
 *     Copyright 2024 The Dragonfly Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use clap::{Parser, Subcommand};
use dragonfly_client::grpc::dfdaemon_download::DfdaemonDownloadClient;
use dragonfly_client::grpc::health::HealthClient;
use dragonfly_client::tracing::init_tracing;
use dragonfly_client_config::VersionValueParser;
use dragonfly_client_config::{dfcache, dfdaemon};
use dragonfly_client_core::Result;
use std::path::PathBuf;
use tracing::Level;

pub mod export;
pub mod import;
pub mod stat;

#[derive(Debug, Parser)]
#[command(
    name = dfcache::NAME,
    author,
    version,
    about = "dfcache is a cache command line based on P2P technology in Dragonfly.",
    long_about = "A cache command line based on P2P technology in Dragonfly that can import file and export file in P2P network, \
    and it can copy multiple replicas during import. P2P cache is effectively used for fast read and write cache.",
    disable_version_flag = true
)]
struct Args {
    #[arg(
        short = 'V',
        long = "version",
        help = "Print version information",
        default_value_t = false,
        action = clap::ArgAction::SetTrue,
        value_parser = VersionValueParser
    )]
    version: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
#[command(args_conflicts_with_subcommands = true)]
pub enum Command {
    #[command(
        name = "import",
        author,
        version,
        about = "Import a file into Dragonfly P2P network",
        long_about = "Import a local file into Dragonfly P2P network and copy multiple replicas during import. If import successfully, it will return a task ID."
    )]
    Import(import::ImportCommand),

    #[command(
        name = "export",
        author,
        version,
        about = "Export a file from Dragonfly P2P network",
        long_about = "Export a file from Dragonfly P2P network by task ID. If export successfully, it will return the local file path."
    )]
    Export(export::ExportCommand),

    #[command(
        name = "stat",
        author,
        version,
        about = "Stat a file in Dragonfly P2P network",
        long_about = "Stat a file in Dragonfly P2P network by task ID. If stat successfully, it will return the file information."
    )]
    Stat(stat::StatCommand),
}

/// Implement the execute for Command.
impl Command {
    #[allow(unused)]
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Import(cmd) => cmd.execute().await,
            Self::Export(cmd) => cmd.execute().await,
            Self::Stat(cmd) => cmd.execute().await,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments.
    let args = Args::parse();

    // Execute the command.
    args.command.execute().await?;
    Ok(())
}

/// Creates and validates a dfdaemon download client with health checking.
///
/// This function establishes a connection to the dfdaemon service via Unix domain socket
/// and performs a health check to ensure the service is running and ready to handle
/// download requests. Only after successful health verification does it return the
/// download client for actual use.
pub async fn get_dfdaemon_download_client(endpoint: PathBuf) -> Result<DfdaemonDownloadClient> {
    // Check dfdaemon's health.
    let health_client = HealthClient::new_unix(endpoint.clone()).await?;
    health_client.check_dfdaemon_download().await?;

    // Get dfdaemon download client.
    let dfdaemon_download_client = DfdaemonDownloadClient::new_unix(endpoint).await?;
    Ok(dfdaemon_download_client)
}
