use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct CliArgs {
    /// Path to custom config file.
    #[clap(short, long)]
    pub config: Option<PathBuf>,

    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// Copy service file to systemd user config dir.
    #[clap(name = "copy-service")]
    CopyService,

    /// Watch libraries and sort added files.
    Watch,

    /// Sort a music directory.
    Sort {
        /// Path to music directory.
        path: Option<PathBuf>,

        /// Custom format string.
        #[clap(short, long)]
        format: Option<String>,

        /// Don't sort anything (simulated run).
        #[clap(short, long)]
        dryrun: bool,

        /// Sort files recursively.
        #[clap(short, long)]
        recursive: bool,

        /// Remove empty directories found while and after sorting.
        #[clap(name = "rm-empty", long)]
        remove_empty: bool,

        /// Mantain file names compatible with FAT32.
        #[clap(short, long)]
        exfat_compat: bool,
    },

    /// Goodies related to sync mode.
    #[cfg(feature = "sync")]
    Sync,
}
