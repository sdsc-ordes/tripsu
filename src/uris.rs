use anyhow::anyhow;
use curie::{ExpansionError, InvalidPrefixError, PrefixMapping};
use regex::Regex;
use sophia_iri::Iri;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::{Display, self},
};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Uri {
    FullUri(String),
    CompactUri(String),
}

impl Uri {
    pub fn is_full(&self) -> bool {
        match self {
            Uri::FullUri(_) => true,
            Uri::CompactUri(_) => false,
        }
    }
    pub fn is_compact(&self) -> bool {
        match self {
            Uri::FullUri(_) => false,
            Uri::CompactUri(_) => true,
        }
    }

    pub fn expand(&self, prefix_map: &PrefixMap) -> Result<Self, PrefixError> {

        let uri = if let Uri::CompactUri(uri) = self {
            uri
        } else {
            return Ok(self.clone());
        };

        match prefix_map.0.expand_curie_string(uri) {
            Err(ExpansionError::Invalid) => Err(PrefixError::InvalidPrefix(uri.to_string())),
            Err(ExpansionError::MissingDefault) => {
                Err(PrefixError::MissingDefault(uri.to_string()))
            }
            Ok(s) => Ok(Self::FullUri(s)),
        }
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Uri::FullUri(uri) => write!(f, "<{}>", uri),
            Uri::CompactUri(uri) => write!(f, "{}", uri),
        }
    }
}

impl TryFrom<&str> for Uri {
    type Error = sophia_iri::InvalidIri;
    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        let curie_re = Regex::new(r"([A-Za-z_][A-Za-z0-9_.\-]*)\:([^\s:/][^\s]*)").unwrap();

        if uri.starts_with('<') && uri.ends_with('>') {
            let trimmed = &uri[1..uri.len() - 2];
            Iri::new(trimmed)?;
            Ok(Self::FullUri(trimmed.to_string()))
        } else if curie_re.is_match(uri) {
            Ok(Self::CompactUri(uri.to_string()))
        } else {
            Err(sophia_iri::InvalidIri(
                format!("Input should be either a URI enclosed in '<>' or a CURIE. Found: {}", uri
            )))
        }
    }
}

/// Render URI as string with angle brackets
impl Into<String> for Uri {
    fn into(self) -> String {
        match self {
            Uri::CompactUri(uri) => uri.to_string(),
            Uri::FullUri(uri) => format!("<{}>", uri),
        }
    }
}

impl TryInto<Iri<String>> for Uri {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Iri<String>, Self::Error> {
        match self {
            Uri::FullUri(uri) => Iri::new(uri.clone()).map_err(|_| anyhow!("Invalid URI: {}", uri)),
            Uri::CompactUri(uri) => Err(anyhow!(
                "Cannot convert CURIE to IRI: {}",
                uri
            )),
        }
    }
}


/// Errors related to CURIE prefixes
#[derive(Debug)]
pub enum PrefixError {
    InvalidPrefix(String),
    MissingDefault(String),
    PrefixNotAllowed(String),
    InvalidPrefixURI(String),
}

impl fmt::Display for PrefixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix(curie) => {
                write!(f, "Invalid prefix: {curie}")
            }
            Self::MissingDefault(curie) => write!(f, "No default prefix provided for: {curie}"),
            Self::PrefixNotAllowed(uri) => write!(f, "Prefix \"_\" not allowed: {uri}"),
            Self::InvalidPrefixURI(uri) => write!(f, "Invalid URI provided for prefix: {uri}"),
        }
    }
}

impl Error for PrefixError {}

impl From<sophia_iri::InvalidIri> for PrefixError {
    fn from(err: sophia_iri::InvalidIri) -> Self {
        PrefixError::InvalidPrefixURI(err.to_string())
    }
}

impl From<InvalidPrefixError> for PrefixError {
    fn from(err: InvalidPrefixError) -> Self {
        PrefixError::PrefixNotAllowed(format!("{:?}", err))
    }
}

/// A mapping of prefixes to URIs
pub struct PrefixMap(PrefixMapping);

impl Default for PrefixMap {
    fn default() -> Self {
        Self::new()
    }
}

impl PrefixMap {
    pub fn new() -> Self {
        PrefixMap(PrefixMapping::default())
    }

    pub fn from_hashmap(
        hashmap: &HashMap<Option<String>, String>,
    ) -> Result<PrefixMap, PrefixError> {
        let mut prefix_map = PrefixMap::new();
        for (key, value) in hashmap {
            Uri::try_from(value.as_str())?;
            // We add prefixes full URIs without the brackets
            if let Some(prefix) = key.as_deref() {
                prefix_map
                    .0
                    .add_prefix(prefix, &value)
                    .map_err(PrefixError::from)?
            } else {
                prefix_map.0.set_default(&value)
            }
        }
        Ok(prefix_map)
    }

    pub fn expand_curie(&self, curie: &str) -> Result<String, PrefixError> {
        match self.0.expand_curie_string(curie) {
            Err(ExpansionError::Invalid) => Err(PrefixError::InvalidPrefix(curie.to_string())),
            Err(ExpansionError::MissingDefault) => {
                Err(PrefixError::MissingDefault(curie.to_string()))
            }
            Ok(s) => Ok(s),
        }
    }
}

/// A collection of distinct URIs
#[derive(Debug, Clone)]
pub struct UriSet(HashSet<Uri>);

impl IntoIterator for UriSet {
    type Item = Uri;
    type IntoIter = std::collections::hash_set::IntoIter<Uri>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}


impl Default for UriSet {
    fn default() -> Self {
        Self::new()
    }
}

impl UriSet {
    pub fn new() -> Self {
        UriSet(HashSet::new())
    }

    pub fn insert(&mut self, hash_set: &HashSet<Uri>) {
        for c in hash_set {
            self.0.insert(c.clone());
        }
    }
    // Return result instead of HashSet
    pub fn expand_set(&self, prefix_map: &PrefixMap) -> Result<HashSet<String>, PrefixError> {
        let mut expanded_set = HashSet::new();
        for uri in &self.0 {
            match self.expand_uri(&uri.to_string(), prefix_map) {
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

pub fn check_full_uri(uri: &str) -> Result<(), anyhow::Error> {
    // Ensure that full URI starts with "<" and ends with ">"
    if !(uri.starts_with('<') && uri.ends_with('>')) {
        return Err(anyhow!("Full URI in rules must start and end with angle brackets <...>. Please format {} into <{}>", uri, uri));
    }
    Ok(())
}

pub fn is_full_uri(uri: &str) -> bool {
    // Ensure that full URI starts with "<" and ends with ">"
    uri.starts_with('<') && uri.ends_with('>')
}

pub fn filter_out_full_uris(hash_set: &HashSet<String>) -> HashSet<String> {
    // Filter out full URIs
    let filtered = hash_set
        .iter()
        .filter(|uri| Uri::try_from(uri.as_str()).unwrap().is_full())
        .cloned()
        .collect();
    filtered
}

pub fn keep_full_uris(hash_set: &HashSet<String>) -> HashSet<String> {
    // Filter out compact URIs
    return hash_set
        .iter()
        .filter(|uri| Uri::try_from(uri.as_str()).unwrap().is_full())
        .cloned()
        .collect();
}

pub fn check_uris(hash_set: &HashSet<String>) -> Result<(), anyhow::Error> {
    // Check if the URIs in the HashSet are valid

    for uri in hash_set {
        check_full_uri(uri)?;
        check_uri(uri)?;
    }
    Ok(())
}

pub fn check_curies(hash_set: &HashSet<String>, prefixes: &PrefixMap) -> Result<(), PrefixError> {
    for uri in hash_set {
        if get_uri(uri).is_err() {
            try_expansion(uri, prefixes)?;
        }
    }
    Ok(())
}

pub fn try_expansion(uri: &str, prefix_map: &PrefixMap) -> Result<String, PrefixError> {
    prefix_map.expand_curie(&uri.to_string())
}
