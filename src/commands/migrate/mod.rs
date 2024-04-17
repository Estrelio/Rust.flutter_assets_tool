pub mod asset_gen;

use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum MigrateCommand {
    /// Migrate from asset_gen.
    #[command(name = "asset_gen")]
    AssetGen,
}
