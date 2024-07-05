use rio_api::{model::Triple, parser::TriplesParser};
use rio_turtle::TurtleError;
use std::{
    io::{stdin, BufRead, BufReader, Write},
    path::Path,
};

use crate::io;

fn index_triple(t: Triple, out: &mut impl Write) -> Result<(), TurtleError> {
    match t.predicate.iri {
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" => {
            let _ = out.write(&format!("{} .\n", &t.to_string()).into_bytes());
        }
        _ => {}
    }

    Ok(())
}

pub fn create_type_map(input: &Path, output: &Path) {
    let buf_in = io::get_reader(input);
    let mut buf_out = io::get_writer(output);
    let mut triples = io::parse_ntriples(buf_in);
    while !triples.is_end() {
        triples
            .parse_step(&mut |t| index_triple(t, &mut buf_out))
            .unwrap();
    }
}
