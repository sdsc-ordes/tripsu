use ::std::collections::{HashMap, HashSet};
use curie::{Curie, ExpansionError, PrefixMapping};
use sophia_iri::Iri;

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

    pub fn import_hashmap(&mut self, hashmap: &HashMap<String, String>) -> &mut Self {
        for (key, value) in hashmap {
            if is_full_uri(value) {
                // We add prefixes full URIs without the brackets
                match self.0.add_prefix(key, &value[1..value.len() - 1]) {
                    Ok(_) => continue,
                    Err(e) => {
                        eprintln!("Failed to add prefix: {:?}", e);
                    }
                }
            } else {
                eprintln!("Invalid URI: {}", value);
            }
        }
        self
    }

    pub fn expand_curie(&self, curie: &Curie) -> Result<String, curie::ExpansionError> {
        self.0.expand_curie(curie)
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

    pub fn expand_set(&self, prefix_map: &PrefixMap) -> HashSet<String> {
        let mut expanded_set = HashSet::new();
        self.0.iter().for_each(|uri| {
            let expanded_uri = self.expand_uri(uri, prefix_map).map_or_else(
                |_| format!("Failed to unwrap cURI: {}", uri),
                |expanded| format!("<{}>", expanded),
            );
            // Once formatted we need to add brackets for full URIs
            expanded_set.insert(expanded_uri);
        });
        expanded_set
    }

    fn expand_uri(&self, uri: &str, prefix_map: &PrefixMap) -> Result<String, ExpansionError> {
        prefix_map.0.expand_curie_string(uri)
    }
}

pub fn check_string_iri(uri: &str) -> Result<Iri<&str>, sophia_iri::InvalidIri> {
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
        check_string_iri(uri)?;
    }
    Ok(())
}

pub fn check_curies_hashset(
    hash_set: &HashSet<String>,
    prefixes: &PrefixMap,
) -> Result<(), curie::ExpansionError> {
    for uri in hash_set {
        if !is_full_uri(uri) {
            try_expansion(uri, prefixes)?;
        }
    }
    Ok(())
}

pub fn try_expansion(uri: &str, prefix_map: &PrefixMap) -> Result<String, curie::ExpansionError> {
    let curie = to_curie(uri);
    prefix_map.expand_curie(&curie)
}

pub fn to_curie(uri: &str) -> Curie<'_> {
    let separator_idx = uri
        .chars()
        .position(|c| c == ':')
        .unwrap_or_else(|| panic!("No separator found in cURI string: {}", uri));
    let prefix = Some(&uri[..separator_idx]);
    let reference = &uri[separator_idx + 1..];
    return Curie::new(prefix, reference);
}
