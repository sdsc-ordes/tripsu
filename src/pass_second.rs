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
    let mask = TripleMask::SUBJECT;
    println!("{}", pseudonymize_triple(&triple, mask).to_string());
    Ok(())
}

pub fn encrypt(log: &Logger, input: &Path, output: &Path, type_map_file: &Path) {
    // Construct the buffer either from `stdio` or from an input file.
    //
    // This object is constructed on the stack and is a `trait object`.
    // The wide-pointer `buffer` will have a pointer to the vtable
    // and pointer to data on the stack.
    // Normally that would be done with `Box::new(std::io::stdin())` on the heap, but since the
    // newest version in Rust that also works on the stack (life-time extensions).
    let buffer: &mut dyn BufRead = match input.to_str().unwrap() {
        "-" => &mut BufReader::new(std::io::stdin()),
        _ => &mut io::get_buffer(input),
    };

    let mut triples = io::parse_ntriples(buffer);
    while !triples.is_end() {
        triples.parse_step(&mut |t| process_triple(&t)).unwrap();
    }
}
