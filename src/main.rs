// Define the module.
mod common;

use std::path::PathBuf;

// Define the imports.
use clap::{Args, Parser};
use common::log::{create_logger, info};

#[derive(Parser)]
#[command(name = "rdf-protect")]
#[command(version = "0.0.1")]
#[command(about ="A tool to anonymize nodes/edges in RDF graphs.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Args, Debug)]
struct TypeMapArgs {
    #[arg(short, long)]
    output_file: PathBuf,
}

#[derive(Args, Debug)]
struct EncryptArgs {
    // The file which maps `node` ids to `type`s.
    // This is used in `encrypt` as the second pass to encrypt RDF triples.
    #[arg(short, long)]
    type_map_file: PathBuf,

    // The input file descriptor to use for outputting the RDF triples.
    // Defaults to `stdin`.
    #[arg(short, long, default_value = "-")]
    input: PathBuf,

    // The output file descriptor to use for outputting the RDF triples.
    // Defaults to `stdout`.
    #[arg(short, long, default_value = "-")]
    output: PathBuf,
}

#[derive(Parser, Debug)]
enum Subcommands {
    // Create the type map which is used in `encrypt` for the second pass to
    // encrypt RDF triples based on some rules.
    CreateTypeMap(TypeMapArgs),
    // Encrypt RDF triples read from a file descriptor (default `stdin`) and
    // based on rules and output them again on a file descriptor (default `stdout`)
    Encrypt(EncryptArgs),
}

fn main() {
    let log = create_logger(true);
    let cli = Cli::parse();

    info!(log, "This is {}", 3);

    match cli.command {
        Subcommands::CreateTypeMap(args) => {
            info!(log, "Args: {:?}", args)
        }
        Subcommands::Encrypt(args) => {
            info!(log, "Args: {:?}", args);
        }
    }
}
