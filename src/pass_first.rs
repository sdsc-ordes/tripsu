use std::path::Path;
use std::io::{BufRead, BufReader, stdin};
use rio_api::{
    parser::TriplesParser,
    model::Triple,
};
use rio_turtle::TurtleError;

use crate::io;

fn index_triple(t: Triple) -> Result<(), TurtleError> {
    match t.predicate.iri {
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" => {
            println!("{}", t.to_string());
        }
        _ => {}
            
    }
    Ok(())
}

pub fn create_index(input: &Path, output: &Path) {

    let buffer: Box<dyn BufRead> = match input.to_str().unwrap() {
        "-" => Box::new(BufReader::new(stdin())),
        _ => Box::new(io::get_buffer(input)),
    };
    let mut triples = io::parse_ntriples(buffer);
    while !triples.is_end() {
        triples.parse_step(&mut |t| {
            index_triple(t)
            }
        ).unwrap();
    }
}
