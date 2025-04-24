use curie::{ExpansionError, PrefixMapping};
use sophia_iri::Iri;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

#[derive(Debug)]
pub struct PrefixMap(PrefixMapping);

#[derive(Debug)]
pub enum PrefixError {
    InvalidPrefix(String),
    MissingDefault(String),
}

impl fmt::Display for PrefixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix(curie) => {
                write!(f, "Invalid prefix: {curie}")
            }
            Self::MissingDefault(curie) => write!(f, "No default prefix provided for: {curie}"),
        }
    }
}

impl Error for PrefixError {}

impl Default for PrefixMap {
    fn default() -> Self {
        Self::new()
    }
}

impl PrefixMap {
    pub fn new() -> Self {
        PrefixMap(PrefixMapping::default())
    }

    pub fn import_hashmap(&mut self, hashmap: &HashMap<Option<String>, String>) -> &mut Self {
        for (key, value) in hashmap {
            if is_full_uri(value) {
                // We add prefixes full URIs without the brackets
                if let Some(prefix) = key.as_deref() {
                    if let Err(e) = self.0.add_prefix(prefix, &value[1..value.len() - 1]) {
                        eprintln!("Failed to add prefix: {:?}", e);
                    }
                } else {
                    self.0.set_default(&value[1..value.len() - 1])
                }
            }
        }
        self
    }

    pub fn expand_curie(&self, curie: &String) -> Result<String, PrefixError> {
        match self.0.expand_curie_string(curie) {
            Err(ExpansionError::Invalid) => Err(PrefixError::InvalidPrefix(curie.to_string())),
            Err(ExpansionError::MissingDefault) => {
                Err(PrefixError::MissingDefault(curie.to_string()))
            }
            Ok(s) => Ok(s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompactUriSet(HashSet<String>);

impl IntoIterator for CompactUriSet {
    type Item = String;
    type IntoIter = std::collections::hash_set::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a CompactUriSet {
    type Item = &'a String;
    type IntoIter = std::collections::hash_set::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Default for CompactUriSet {
    fn default() -> Self {
        Self::new()
    }
}

impl CompactUriSet {
    pub fn new() -> Self {
        CompactUriSet(HashSet::new())
    }

    pub fn insert(&mut self, hash_set: &HashSet<String>) {
        for c in hash_set {
            self.0.insert(c.clone());
        }
    }
    // Return result instead of HashSet
    pub fn expand_set(&self, prefix_map: &PrefixMap) -> Result<HashSet<String>, PrefixError> {
        let mut expanded_set = HashSet::new();
        for uri in &self.0 {
            match self.expand_uri(uri, prefix_map) {
                Ok(expanded_uri) => {
                    expanded_set.insert(format!("<{}>", expanded_uri));
                }
                Err(e) => return Err(e),
            }
        }
        Ok(expanded_set)
    }

    fn expand_uri(&self, uri: &str, prefix_map: &PrefixMap) -> Result<String, PrefixError> {
        match prefix_map.0.expand_curie_string(uri) {
            Err(ExpansionError::Invalid) => Err(PrefixError::InvalidPrefix(uri.to_string())),
            Err(ExpansionError::MissingDefault) => {
                Err(PrefixError::MissingDefault(uri.to_string()))
            }
            Ok(s) => Ok(s),
        }
    }
}

pub fn check_uri(uri: &str) -> Result<Iri<&str>, sophia_iri::InvalidIri> {
    // We assume that a full URI starts with "<" and ends with ">"
    // We select the URI within the angle brackets
    Iri::new(&uri[1..uri.len() - 2])
}

pub fn is_full_uri(uri: &str) -> bool {
    // Ensure that full URI starts with "<" and ends with ">"
    uri.starts_with('<') && uri.ends_with('>')
}

pub fn filter_out_full_uris(hash_set: &HashSet<String>) -> HashSet<String> {
    // Filter out full URIs
    let filtered = hash_set
        .iter()
        .filter(|uri| !is_full_uri(uri))
        .cloned()
        .collect();
    filtered
}

pub fn keep_full_uris(hash_set: &HashSet<String>) -> HashSet<String> {
    // Filter out compact URIs
    return hash_set
        .iter()
        .filter(|uri| is_full_uri(uri))
        .cloned()
        .collect();
}

pub fn check_uris(hash_set: &HashSet<String>) -> Result<(), sophia_iri::InvalidIri> {
    // Check if the URIs in the HashSet are valid

    for uri in hash_set {
        check_uri(uri)?;
    }
    Ok(())
}

pub fn check_curies(hash_set: &HashSet<String>, prefixes: &PrefixMap) -> Result<(), PrefixError> {
    for uri in hash_set {
        if !is_full_uri(uri) {
            try_expansion(uri, prefixes)?;
        }
    }
    Ok(())
}

pub fn try_expansion(uri: &str, prefix_map: &PrefixMap) -> Result<String, PrefixError> {
    prefix_map.expand_curie(&uri.to_string())
}
