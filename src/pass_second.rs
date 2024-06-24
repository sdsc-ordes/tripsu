use rio_api::{model::Triple, parser::TriplesParser};
use rio_turtle::TurtleError;
use std::{
    collections::HashMap, io::{BufRead, Write}, path::Path
};

use crate::{
    io,
    log::Logger,
    model::{pseudonymize_triple, TripleMask},
};

fn mask_triple(triple: &Triple) -> TripleMask {
    return TripleMask::SUBJECT
}

// mask and encode input triple
// NOTE: This will need the type-map to perform masking
fn process_triple(triple: &Triple, out: &mut impl Write) -> Result<(), TurtleError> {
    let mask = mask_triple(triple);
    let pseudo_triple = pseudonymize_triple(&triple, mask);
    let _ = out.write(
                &format!("{} .\n", &pseudo_triple.to_string()).into_bytes()
    );
    Ok(())
}

// Create a index mapping node -> type from an input ntriples buffer 
fn load_index(input: impl BufRead) -> HashMap<String, String> {
   let mut node2type: HashMap<String, String> = HashMap::new();
    let mut triples = io::parse_ntriples(input);

    while !triples.is_end() {
        let _ = triples.parse_step(&mut |t| {
            node2type.insert(t.subject.to_string(), t.object.to_string());
            Ok(()) as Result<(), TurtleError>
        }
    );
    }
    node2type
}

pub fn pseudonymize_graph(log: &Logger, input: &Path, output: &Path, index: &Path) {
    let buf_input = io::get_reader(input);
    let buf_index = io::get_reader(index);
    let mut buf_output = io::get_writer(output);

    let node2type: HashMap<String, String> = load_index(buf_index);
    let mut triples = io::parse_ntriples(buf_input);
    while !triples.is_end() {
        triples.parse_step(&mut |t| process_triple(&t, &mut buf_output)).unwrap();
    }
}
