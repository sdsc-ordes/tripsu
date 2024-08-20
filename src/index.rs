use rio_api::parser::TriplesParser;
use rio_turtle::TurtleError;
use serde::{Deserialize, Serialize};
use serde_yml;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};

use crate::{
    io,
    rdf_types::{Triple, TripleView},
};

#[derive(Serialize, Deserialize)]
pub struct Index {
    pub types: Vec<String>,
    map: HashMap<[u8; 8], Vec<usize>>,

    #[serde(skip)]
    hasher: DefaultHasher,
}

impl Index {
    fn hash(&mut self, s: &str) -> [u8; 8] {
        s.hash(&mut self.hasher);
        self.hasher.finish().to_be_bytes()
    }

    pub fn new() -> Self {
        Index {
            types: Vec::new(),
            map: HashMap::new(),
            hasher: DefaultHasher::new(),
        }
    }

    // Insert input subject-type mapping into the index.
    // The index will store the hash of the subject.
    pub fn insert(&mut self, subject_key: &str, type_val: &str) -> Result<(), std::io::Error> {
        let key = self.hash(subject_key);
        let type_idx: usize;

        // Get type index or add a new one
        if self.types.contains(&type_val.to_string()) {
            type_idx = self.types.iter().position(|x| *x == type_val).unwrap();
        } else {
            type_idx = self.types.len();
            self.types.push(type_val.to_string());
        }
        // Insert mapping into the index
        match self.map.get_mut(&key) {
            Some(v) => {
                v.push(type_idx);
            }
            None => {
                self.map.insert(key, vec![type_idx]);
            }
        }

        Ok(())
    }

    pub fn get(&mut self, subject_key: &str) -> Option<Vec<&String>> {
        let key = self.hash(subject_key);
        let val = if let Some(v) = self.map.get(&key) {
            Some(v.iter().map(|i| &self.types[*i]).collect())
        } else {
            None
        };

        return val;
    }
}

fn index_triple(t: Triple, index: &mut Index) {
    if t.predicate.iri.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
        let r = || -> std::io::Result<()> {
            index.insert(&t.subject.to_string(), &t.object.to_string())
        }();

        if let Err(e) = r {
            panic!("Error writting to out buffer: {e}");
        }
    }
}

pub fn create_type_map(input: &Path, output: &Path) {
    let buf_in = io::get_reader(input);
    let buf_out = io::get_writer(output);
    let mut triples = io::parse_ntriples(buf_in);
    let mut index = Index::new();

    while !triples.is_end() {
        let _ = triples
            .parse_step(&mut |t: TripleView| {
                index_triple(t.into(), &mut index);
                Result::<(), TurtleError>::Ok(())
            })
            .inspect_err(|e| {
                panic!("Parsing error occured: {e}");
            });
    }
    let _ = serde_yml::to_writer(buf_out, &index);
}
