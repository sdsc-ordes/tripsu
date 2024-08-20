use rio_api::parser::TriplesParser;
use rio_turtle::TurtleError;
use std::{io::Write, path::Path};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    io,
    rdf_types::{Triple, TripleView},
};

#[derive(Serialize, Deserialize)]
struct Index {
    types: Vec<String>,
    map: HashMap<[u8; 8], usize>,

    #[serde(skip)]
    hasher: DefaultHasher,
}

impl Index {

    fn hash(&mut self, s: &str) -> [u8; 8] {
        s.hash(&mut self.hasher);
        self.hasher.finish().to_be_bytes()
    }

    fn new() -> Self {
        Index {
            types: Vec::new(),
            map: HashMap::new(),
            hasher: DefaultHasher::new(),
        }
    }

    fn insert(&mut self, subject_key: &str, type_val: &str) {
        let key = self.hash(subject_key);
        let type_idx: usize;
        if self.types.contains(&type_val.to_string()) {
            type_idx = self.types.iter().position(|x| *x == type_val).unwrap();
        } else {
            type_idx = self.types.len();
            self.types.push(type_val.to_string());
        }
        self.map.insert(key, type_idx);
    }

    fn get(&mut self, subject_key: &str) -> Option<&str> {
        let key = self.hash(subject_key);
        self.map.get(&key).map(|i| self.types[*i].as_str())
    }
}

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
