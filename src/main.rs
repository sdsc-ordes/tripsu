// Define the module.
mod crypto;
mod io;
mod log;
mod model;
mod pass_first;
mod pass_second;
mod rules;

// Define the imports.
use crate::{
    log::{create_logger, info},
    pass_first::create_index,
    pass_second::pseudonymize_graph,
};

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rdf-protect")]
#[command(version = "0.0.1")]
#[command(about ="A tool to pseudonymize URIs and values in RDF graphs.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Args, Debug)]
struct IndexArgs {
    /// The output file descriptor to use for outputting the node-to-type index.
    #[arg(short, long)]
    output: PathBuf,
    
    /// The input file descriptor to use for outputting the RDF triples.
    /// Defaults to `stdin`.
    #[arg(default_value = "-")]
    input: PathBuf,
}

#[derive(Args, Debug)]
struct PseudoArgs {
    /// Index file produced by prepare-index.
    /// Required for pseudonymization. 
    #[arg(short, long)]
    index: PathBuf,

    /// File descriptor to read input triples from.
    /// Defaults to `stdin`.
    #[arg(default_value = "-")]
    input: PathBuf,

    /// Output file descriptor for pseudonymized triples.
    /// Defaults to `stdout`.
    #[arg(short, long, default_value = "-")]
    output: PathBuf,
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
    let log = create_logger(true);
    let cli = Cli::parse();

    match cli.command {
        Subcommands::Index(args) => {
            info!(log, "Args: {:?}", args);
            create_index(&args.input, &args.output)
        }
        Subcommands::Pseudo(args) => {
            info!(log, "Args: {:?}", args);
            pseudonymize_graph(&log, &args.input, &args.output, &args.index)
        }
    }
}

#[cfg(test)]
mod tests {}
