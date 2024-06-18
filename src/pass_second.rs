use std::{
    io::{BufRead, BufReader},
    path::Path,
};
use rio_api::parser::TriplesParser;
use rio_turtle::TurtleError;

use crate::{info, io, log::Logger, model::{MaskedTriple, Pseudonymize, TripleMask}};

pub fn encrypt(log: &Logger, input: &Path, output: &Path, type_map_file: &Path) {
    // Construct the buffer either from `stdio` or from an input file.
    // This object is constructed on the heap: `Box` and is a `trait object` (a dynamic dispatch)
    let buffer: Box<dyn BufRead> = match input.to_str().unwrap() {
        "-" => Box::new(BufReader::new(std::io::stdin())),
        _ => Box::new(io::get_buffer(input)),
    };

    let mut triples = io::parse_ntriples(buffer);
    while !triples.is_end() {
        triples.parse_step(
            &mut |t|{
                let pseudo_triple: MaskedTriple;
                pseudo_triple = MaskedTriple::new(t, TripleMask::SUBJECT).pseudo();
                info!(log, "{:?}", pseudo_triple);
                Ok(()) as Result<(), TurtleError>
            }
        ).unwrap();
    }
}
