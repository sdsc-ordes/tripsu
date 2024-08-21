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

/// Stores a mapping from hashed instance uri to their types
#[derive(Serialize, Deserialize)]
pub struct TypeIndex {
    pub types: Vec<String>,
    map: HashMap<String, Vec<usize>>,
}

impl TypeIndex {
    fn hash(&mut self, s: &str) -> String {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn from_iter(type_map: impl Iterator<Item = (String, String)>) -> Self {
        let mut idx = TypeIndex {
            types: type_map
                .map(|(_, &t)| t.clone())
                .collect::<std::collections::HashSet<String>>()
                .into_iter()
                .collect(),
            map: HashMap::new(),
        };

            type_map.for_each(|(subject, type_uri)| idx.insert(&subject.to_string(), &type_uri.to_string()).unwrap());

        return idx;
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
        let key = self.hash(subject_uri);
        let type_idx: usize;

        // Get type index or add a new one
        if self.types.contains(&type_uri.to_string()) {
            type_idx = self.types.iter().position(|x| *x == type_uri).unwrap();
        } else {
            type_idx = self.types.len();
            self.types.push(type_uri.to_string());
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

fn index_triple(t: Triple, index: &mut TypeIndex) {
    if t.predicate.iri.as_str() == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
        let r = || -> std::io::Result<()> {
            index.insert(&t.subject.to_string(), &t.object.to_string())
        }();

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
    let _ = serde_yml::to_writer(buf_out, &index);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // Test the parsing of a triple.
    fn index_from_map() {
        let vals = vec![
            ("urn:Alice", "urn:Person"),
            ("urn:Alice", "urn:Employee"),
            ("urn:ACME", "urn:Organization"),
        ]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()));

        let map = HashMap::from_iter(vals);
        let mut idx = TypeIndex::from_map(map);

        assert_eq!(idx.get("urn:Alice").unwrap(), vec!["urn:Person", "urn:Employee"]);
        println!("{}", serde_yml::to_string(&idx).unwrap());
    }
}
