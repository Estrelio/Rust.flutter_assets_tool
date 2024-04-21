use clap::Subcommand;

use crate::core::flutter::pubspec_yaml::FlutterGenAssetsOutputsStyle;

pub mod asset_gen;
pub mod flutter_gen;

#[derive(Subcommand, Debug, Clone)]
pub enum MigrateCommand {
    /// Migrate from asset_gen.
    #[command(name = "asset_gen")]
    AssetGen,

    /// Migrate flutter_gen style.
    ///
    /// Picks up the style configuration from the pubspec.yaml file and migrate the existing syntax
    /// from `previous_style`.
    #[command(name = "flutter_gen")]
    FlutterGen {
        /// Paths to ignore when searching for unused translations.
        ///
        /// If the specified path is a directory, ignore all files in the directory; otherwise,
        /// ignore the specified file.
        ///
        /// Make sure that you specify the directory path exactly how you specify it in your
        /// pubspec.yaml flutter.assets section.
        #[clap(long = "ignore-path")]
        ignore_paths: Option<Vec<String>>,

        #[clap(long, short, default_value = "snake-case")]
        #[arg(value_enum)]
        previous_style: FlutterGenAssetsOutputsStyle,
    },
}
