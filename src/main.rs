// Define the module.
mod crypto;
mod io;
mod log;
mod model;
mod pass_first;
mod pass_second;
mod rdf_types;
mod rules;

// Define the imports.
use crate::{
    log::{create_logger, info},
    pass_first::create_type_map,
    pass_second::pseudonymize_graph,
};

use clap::{Args, Parser, Subcommand};
use log::Logger;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tripsu")]
#[command(version = "0.0.1")]
#[command(about ="A tool to pseudonymize URIs and values in RDF graphs.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Subcommands,

    /// Parallelize the convertion loops.
    /// Specifies the number of threads to use.
    /// A number of `0` will use all threads.
    #[arg(short, long)]
    parallel: Option<usize>,
}

#[derive(Args, Debug)]
struct IndexArgs {
    /// Output file descriptor to for the node-to-type index.
    #[arg(short, long, default_value = "-")]
    output: PathBuf,

    /// File descriptor to read triples from.
    /// Defaults to `stdin`.
    #[arg(short, long, default_value = "-")]
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

    /// The config file descriptor to use for defining RDF elements to pseudonymize.
    /// Format: yaml
    #[arg(short, long)]
    config: PathBuf,

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

fn install_thread_pool(log: &Logger, threads: usize) {
    let num_threads = if threads == 0 {
        std::thread::available_parallelism()
            .expect("Could not get available num. threads.")
            .get()
    } else {
        threads
    };

    info!(log, "Installing thread pool with '{num_threads}' threads.");
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Could not setup thread pool.");
}

fn main() {
    let log = create_logger(false);
    let cli = Cli::parse();

    let mut parallelize = false;
    if let Some(num_threads) = cli.parallel {
        parallelize = true;
        install_thread_pool(&log, num_threads);
    }

    match cli.command {
        Subcommands::Index(args) => {
            info!(log, "Args: {:?}", args);
            create_type_map(&args.input, &args.output)
        }
        Subcommands::Pseudo(args) => {
            info!(log, "Args: {:?}", args);
            pseudonymize_graph(
                &log,
                &args.input,
                &args.config,
                &args.output,
                &args.index,
                parallelize,
            )
        }
    }
}

#[cfg(test)]
mod tests {}
