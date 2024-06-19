use rio_api::{model::Triple, parser::TriplesParser};
use rio_turtle::TurtleError;
use std::{
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{
    io,
    log::Logger,
    model::{pseudonymize_triple, TripleMask},
};

// mask and encode input triple
// NOTE: This will need the type-map to perform masking
fn process_triple(triple: &Triple) -> Result<(), TurtleError> {
    let mask = TripleMask::new();
    println!("{}", pseudonymize_triple(&triple, mask).to_string());
    Ok(())
}

pub fn encrypt(log: &Logger, input: &Path, output: &Path, type_map_file: &Path) {
    // Construct the buffer either from `stdio` or from an input file.
    // This object is constructed on the heap: `Box` and is a `trait object` (a dynamic dispatch)
    let buffer: Box<dyn BufRead> = match input.to_str().unwrap() {
        "-" => Box::new(BufReader::new(std::io::stdin())),
        _ => Box::new(io::get_buffer(input)),
    };

    let mut triples = io::parse_ntriples(buffer);
    while !triples.is_end() {
        triples.parse_step(&mut |t| process_triple(&t)).unwrap();
    }
}
