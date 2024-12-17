// Define the module.
mod crypto;
mod index;
mod io;
mod log;
mod model;
mod pseudo;
mod rdf_types;
mod rules;

// Define the imports.
use crate::{
    index::create_type_index,
    log::{create_logger, info},
    pseudo::pseudonymize_graph,
};

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tripsu")]
#[command(version = "0.0.1")]
#[command(about ="A tool to pseudonymize URIs and values in RDF graphs.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Args, Debug)]
struct IndexArgs {
    /// Output file descriptor to for the node-to-type index.
    #[arg(short, long, default_value = "-")]
    output: PathBuf,

    /// File descriptor to read triples from.
    /// Defaults to `stdin`.
    #[arg(default_value = "-")]
    input: PathBuf,
}

#[derive(Args, Debug)]
struct PseudoArgs {
    /// Index file produced by prepare-index.
    /// Required for pseudonymization.
    #[arg(short = 'x', long)]
    index: PathBuf,

    /// File descriptor to read input triples from.
    /// Defaults to `stdin`.
    #[arg(default_value = "-")]
    input: PathBuf,

    /// File defining which RDF elements to pseudonymize.
    /// Format: yaml
    #[arg(short, long)]
    rules: PathBuf,

    /// Output file descriptor for pseudonymized triples.
    /// Defaults to `stdout`.
    #[arg(short, long, default_value = "-")]
    output: PathBuf,

    /// File containing the secret used to generate pseudonyms.
    /// Default is to use a random key.
    #[arg(short, long, default_value=None)]
    secret: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    /// 1. Pass: Create a node-to-type index from input triples.
    // This is used in `pseudonymize` for the second pass to
    // pseudonymize RDF triples based on a configuration.
    Index(IndexArgs),

    /// 2. Pass: Pseudonymize input triples.
    // A config file defines pseudonymization rules. The deidentified triples are sent to the
    // output file descriptor. (default `stdout`)
    Pseudo(PseudoArgs),
}

fn main() {
    let log = create_logger(false);
    let cli = Cli::parse();

    match cli.command {
        Subcommands::Index(args) => {
            info!(log, "Args: {:?}", args);
            create_type_index(&args.input, &args.output)
        }
        Subcommands::Pseudo(args) => {
            info!(log, "Args: {:?}", args);
            pseudonymize_graph(
                &log,
                &args.input,
                &args.rules,
                &args.output,
                &args.index,
                &args.secret,
            )
        }
    }
}

#[cfg(test)]
mod tests {}
