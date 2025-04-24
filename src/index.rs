use rio_api::parser::TriplesParser;
use rio_turtle::TurtleError;
use serde::{Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};

use crate::{
    io,
    rdf_types::{Triple, TripleView},
};

/// Stores a mapping from hashed instance uri to their types.
/// The type URIs are stored once as a vector of strings.
/// Each subject in map is stored as hash(subject_uri): u64
/// and refers to its types using their vector index.
#[derive(Serialize, Deserialize)]
pub struct TypeIndex {
    pub types: Vec<String>,
    map: HashMap<u64, SmallVec<[usize; 1]>>,
}

impl TypeIndex {
    fn hash(&self, s: &impl Hash) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish().to_le()
    }

    pub fn from_iter<'a>(type_map: impl Iterator<Item = (&'a str, &'a str)>) -> Self {
        let mut idx = TypeIndex::new();

        type_map.for_each(|(subject_uri, type_uri)| idx.insert(subject_uri, type_uri).unwrap());

        idx
    }

    pub fn new() -> Self {
        TypeIndex {
            types: Vec::new(),
            map: HashMap::new(),
        }
    }

    // Insert input subject-type mapping into the index.
    // The index will store the hash of the subject.
    pub fn insert(&mut self, subject_uri: &str, type_uri: &str) -> Result<(), std::io::Error> {
        let key = self.hash(&subject_uri.to_string());
        let type_idx: usize;

        // Get type index or add a new one.
        if self.types.contains(&type_uri.to_string()) {
            type_idx = self.types.iter().position(|x| *x == type_uri).unwrap();
        } else {
            type_idx = self.types.len();
            self.types.push(type_uri.to_string());
        }
        // Insert mapping into the index.
        match self.map.get_mut(&key) {
            Some(v) => {
                // Push index value only when new
                if !v.iter().any(|x| *x == type_idx) {
                    v.push(type_idx);
                }
            }
            None => {
                self.map.insert(key, smallvec![type_idx]);
            }
        }

        Ok(())
    }

    pub fn get(&self, subject_key: &str) -> Option<Vec<&str>> {
        let key = self.hash(&subject_key.to_string());
        self.map
            .get(&key)
            .map(|v| v.iter().map(|i| self.types[*i].as_ref()).collect())
    }
}

fn index_triple(t: Triple, index: &mut TypeIndex) {
    if t.predicate.iri.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
        let r = { index.insert(&t.subject.to_string(), &t.object.to_string()) };

        if let Err(e) = r {
            panic!("Error writting to out buffer: {e}");
        }
    }
}

pub fn create_type_index(input: &Path, output: &Path) {
    let buf_in = io::get_reader(input);
    let buf_out = io::get_writer(output);
    let mut triples = io::parse_ntriples(buf_in);
    let mut index = TypeIndex::new();

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
    let _ = serde_json::to_writer(buf_out, &index);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // Test the parsing of a triple.
    fn index_from_iter() {
        let vals = vec![
            ("<urn:Alice>", "<urn:Person>"),
            ("<urn:Alice>", "<urn:Employee>"),
            ("<urn:ACME>", "<urn:Organization>"),
        ]
        .into_iter();

        let idx = TypeIndex::from_iter(vals);

        assert_eq!(
            idx.get("<urn:Alice>").unwrap(),
            vec!["<urn:Person>", "<urn:Employee>"]
        );
        println!("{}", serde_json::to_string(&idx).unwrap());
    }
}
