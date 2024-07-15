use rio_api::parser::TriplesParser;
use rio_turtle::TurtleError;
use std::{io::Write, path::Path};

use crate::{
    io,
    rdf_types::{Triple, TripleView},
};

fn index_triple(t: Triple, out: &mut impl Write) {
    if t.predicate.iri.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
        let r = || -> std::io::Result<()> {
            out.write_all(t.to_string().as_bytes())?;
            out.write_all(b" .\n")
        }();

        if let Err(e) = r {
            panic!("Error writting to out buffer: {e}");
        }
    }
}

pub fn create_type_map(input: &Path, output: &Path) {
    let buf_in = io::get_reader(input);
    let mut buf_out = io::get_writer(output);
    let mut triples = io::parse_ntriples(buf_in);

    while !triples.is_end() {
        let _ = triples
            .parse_step(&mut |t: TripleView| {
                index_triple(t.into(), &mut buf_out);
                Result::<(), TurtleError>::Ok(())
            })
            .inspect_err(|e| {
                panic!("Parsing error occured: {e}");
            });
    }
}
