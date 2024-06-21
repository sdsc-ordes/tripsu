use std::path::Path;
use std::io::{BufRead, BufReader, stdin, Write};
use rio_api::{
    parser::TriplesParser,
    model::Triple,
};
use rio_turtle::TurtleError;

use crate::io;

fn index_triple(t: Triple, out: &mut impl Write) -> Result<(), TurtleError> {
    match t.predicate.iri {
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" => {
            let _ = out.write(t.predicate.iri.as_bytes());
        }
        _ => {}
            
    }
    Ok(())
}

pub fn create_index(input: &Path, output: &Path) {

    let buf_in: Box<dyn BufRead> = match input.to_str().unwrap() {
        "-" => Box::new(BufReader::new(stdin())),
        _ => Box::new(io::get_reader(input)),
    };
    let mut buf_out = io::get_writer(output);
    let mut triples = io::parse_ntriples(buf_in);
    while !triples.is_end() {
        triples.parse_step(&mut |t| {
            index_triple(t, &mut buf_out)
            }
        ).unwrap();
    }
}
