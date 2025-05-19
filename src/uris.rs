use anyhow::anyhow;
use curie::{ExpansionError, InvalidPrefixError, PrefixMapping};
use regex::Regex;
use sophia_iri;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    iter::FromIterator,
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
        return !self.is_full();
    }

    pub fn expand(&self, prefix_map: &PrefixMap) -> Result<Self, PrefixError> {

        prefix_map.expand_curie(self)
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

impl TryFrom<String> for Uri {
    type Error = sophia_iri::InvalidIri;
    fn try_from(uri: String) -> Result<Self, Self::Error> {
        let curie_re = Regex::new(r"([A-Za-z_][A-Za-z0-9_.\-]*)?\:([^\s:/][^\s]*)").unwrap();

        if uri.starts_with('<') && uri.ends_with('>') {
            let trimmed = &uri[1..uri.len() - 1];
            sophia_iri::Iri::new(trimmed)?;
            Ok(Self::FullUri(trimmed.to_string()))
        } else if curie_re.is_match(&uri) {
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

impl TryInto<sophia_iri::Iri<String>> for Uri {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<sophia_iri::Iri<String>, Self::Error> {
        match self {
            Uri::FullUri(uri) => sophia_iri::Iri::new(uri.clone()).map_err(|_| anyhow!("Invalid URI: {}", uri)),
            Uri::CompactUri(uri) => Err(anyhow!(
                "CURIEs cannot be converted IRIs: {}",
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
#[derive(Debug)]
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
            Uri::try_from(value.clone())?;
            // We add prefixes full URIs without the brackets
            if let Some(prefix) = key.as_deref() {
                prefix_map
                    .0
                    .add_prefix(prefix, &value[1..&value.len()-1])
                    .map_err(PrefixError::from)?
            } else {
                prefix_map.0.set_default(&value)
            }
        }
        Ok(prefix_map)
    }

    /// Expand input curie into a full uri. If the input uri is already full,
    /// it is returned a-is.
    pub fn expand_curie(&self, uri: &Uri) -> Result<Uri, PrefixError> {
        let curie = match uri {
            Uri::CompactUri(val) => val,
            _ => return Ok(uri.clone()),
        };
          
        match self.0.expand_curie_string(&curie) {
            Err(ExpansionError::Invalid) => Err(PrefixError::InvalidPrefix(curie.to_string())),
            Err(ExpansionError::MissingDefault) => {
                Err(PrefixError::MissingDefault(curie.to_string()))
            }
            Ok(s) => Ok(Uri::FullUri(s)),
        }
    }

    pub fn expand_curies(&self, curies: &UriSet) -> Result<UriSet, PrefixError> {
        let mut expanded_set = UriSet::new();

        for curie in curies.clone() {
            match self.expand_curie(&curie) {
                Ok(expanded_uri) => {
                    expanded_set.insert(expanded_uri);
                }
                Err(e) => return Err(e),
            }
        }

        Ok(expanded_set.try_into().unwrap())
    }

    pub fn check_curies(&self, curies: &UriSet) -> Result<(), PrefixError> {
        for curie in curies.clone() {
            match self.expand_curie(&curie) {
                Ok(_) => {},
                Err(e) => return Err(e),
            }
        }
        Ok(())
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


impl TryFrom <HashSet<String>> for UriSet {
    type Error = anyhow::Error;
    fn try_from(hash_set: HashSet<String>) -> Result<Self, Self::Error> {
        let mut uri_set = HashSet::new();
        for str_uri in hash_set.clone() {
            let uri = Uri::try_from(str_uri)?;
            uri_set.insert(uri);
        }
        Ok(UriSet(uri_set))
    }
}

impl Into<HashSet<String>> for UriSet {
    fn into(self) -> HashSet<String> {
        let mut hash_set = HashSet::new();
        for uri in self.0 {
            hash_set.insert(uri.to_string());
        }
        hash_set
    }
}

impl FromIterator<Uri> for UriSet {
    fn from_iter<I: IntoIterator<Item=Uri>>(iter: I) -> Self {
        let mut hash_set = HashSet::new();
        for item in iter {
            hash_set.insert(item);
        }
        UriSet(hash_set)
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

    pub fn insert(&mut self, uri: Uri) {
        self.0.insert(uri);
    }

    pub fn extend(&mut self, hash_set: &UriSet) {
        for c in hash_set.clone() {
            self.insert(c);
        }
    }

    /// Returns a new UriSet containing the URIs expanded with the given prefix map
    pub fn expand(&self, prefix_map: &PrefixMap) -> Result<UriSet, PrefixError> {
        let mut expanded_set = UriSet::new();
        for uri in self.clone().into_iter() {
            let expanded_uri = prefix_map.expand_curie(&uri)?;
            expanded_set.insert(expanded_uri);
        }
        Ok(expanded_set)
    }

    /// Returns a new UriSet containing only full URIs
    pub fn full_uris(&self) -> Self {
        self.clone()
            .into_iter()
            .filter(|uri| uri.is_full())
            .collect()
    }

    /// Returns a new UriSet containing only compact URIs
    pub fn compact_uris(&self) -> Self {
        self.clone()
            .into_iter()
            .filter(|uri| uri.is_compact())
            .collect()
    }
}

