use std::{
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{info, io, log::Logger, model::TriplePart};

pub fn encrypt(log: &Logger, input: &Path, output: &Path, type_map_file: &Path) {
    // Construct the buffer either from `stdio` or from an input file.
    // This object is constructed on the heap: `Box` and is a `trait object` (a dynamic dispatch)
    let buffer: Box<dyn BufRead> = match input.to_str().unwrap() {
        "-" => Box::new(BufReader::new(std::io::stdin())),
        _ => Box::new(io::get_buffer(input)),
    };

    let triples = io::parse_ntriples(buffer);

    for triple in triples {
        info!(log, "{:?}", triple.hash_parts(TriplePart::SUBJECT));
    }
}
