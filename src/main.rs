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
    pass_second::encrypt,
};

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rdf-protect")]
#[command(version = "0.0.1")]
#[command(about ="A tool to anonymize nodes/edges in RDF graphs.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Args, Debug)]
struct IndexArgs {
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Args, Debug)]
struct PseudoArgs {
    /// The file which maps `node` ids to `type`s.
    /// This is used in `encrypt` as the second pass to encrypt RDF triples.
    #[arg(short, long)]
    index: PathBuf,

    /// The input file descriptor to use for outputting the RDF triples.
    /// Defaults to `stdin`.
    #[arg(default_value = "-")]
    input: PathBuf,

    /// The output file descriptor to use for outputting the RDF triples.
    // Defaults to `stdout`.
    #[arg(short, long, default_value = "-")]
    output: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    /// 1. Pass: Create the node-to-type index.
    // This is used in `encrypt` for the second pass to
    // encrypt RDF triples based on some rules.
    PrepareIndex(IndexArgs),

    /// 2. Pass: Pseudonymize RDF triples read from a file descriptor (default `stdin`)
    // This is based on rules and output them again on a file descriptor (default `stdout`)
    Pseudonymize(PseudoArgs),
}

fn main() {
    let log = create_logger(true);
    let cli = Cli::parse();

    match cli.command {
        Subcommands::PrepareIndex(args) => {
            info!(log, "Args: {:?}", args)
        }
        Subcommands::Pseudonymize(args) => {
            info!(log, "Args: {:?}", args);
            encrypt(&log, &args.input, &args.output, &args.index)
        }
    }
}

#[cfg(test)]
mod tests {}
