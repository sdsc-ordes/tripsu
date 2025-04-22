use crate::{rdf_types::*, uris::*};
use ::std::collections::{HashMap, HashSet};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::{index::TypeIndex, model::TripleMask};

/// Rules for pseudonymizing nodes
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NodeRules {
    // Replace values of nodes with a certain type.
    #[serde(default)]
    of_type: HashSet<String>,
}

impl NodeRules {
    /// Validate each full URI specified in the rules for nodes
    pub fn check_uris(&self) -> Result<(), sophia_iri::InvalidIri> {
        let node_uris = keep_full_uris(&self.of_type);
        check_uris(&node_uris)
    }

    /// Validates the CURIEs in the `of_type` fields
    /// against the provided prefix map, ensuring they can be expanded correctly.
    pub fn check_curies(&self, prefixes: &PrefixMap) -> Result<(), PrefixError> {
        check_curies(&self.of_type, prefixes)
    }

    /// Checks if the provided cURIs for nodes can be expanded given the prefixes provided
    pub fn expand_curies(&self, prefixes: &PrefixMap) -> Result<NodeRules, PrefixError> {
        let mut nodes_full_uris: HashSet<String> = keep_full_uris(&self.of_type);
        let nodes_curies = filter_out_full_uris(&self.of_type);
        let mut nodes_curies_set = CompactUriSet::new();

        nodes_curies_set.insert(&nodes_curies);

        let valid_nodes_curies = nodes_curies_set.expand_set(prefixes)?;
        nodes_full_uris.extend(valid_nodes_curies);

        Ok(NodeRules {
            of_type: nodes_full_uris,
        })
    }
}

/// Rules for pseudonymizing objects
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ObjectRules {
    /// Replace values in matched `predicates`.
    #[serde(default)]
    on_predicate: HashSet<String>,
    /// Replace values of predicates for specific types
    #[serde(default)]
    on_type_predicate: HashMap<String, HashSet<String>>,
}

impl ObjectRules {
    /// Validate each full URI specified in the rules for objects
    pub fn check_uris(&self) -> Result<(), sophia_iri::InvalidIri> {
        let on_predicate_uris = keep_full_uris(&self.on_predicate);
        check_uris(&on_predicate_uris)?;
        for (k, v) in self.on_type_predicate.iter() {
            // Check if a string is a full URI and if it is check iri
            if is_full_uri(k) {
                check_uri(k)?;
            }
            let on_type_predicate_uris = keep_full_uris(v);
            check_uris(&on_type_predicate_uris)?;
        }
        Ok(())
    }

    /// Validates the CURIEs in the `on_predicate` and `on_type_predicate` fields
    /// against the provided prefix map, ensuring they can be expanded correctly.
    pub fn check_curies(&self, prefixes: &PrefixMap) -> Result<(), PrefixError> {
        check_curies(&self.on_predicate, prefixes)?;
        for (k, v) in &self.on_type_predicate {
            if !is_full_uri(k) {
                try_expansion(k, prefixes)?;
            }
            check_curies(v, prefixes)?;
        }
        Ok(())
    }
    /// Takes an object rule and extracts all compact URIs
    pub fn keep_curies(&self) -> ObjectRules {
        ObjectRules {
            on_predicate: filter_out_full_uris(&self.on_predicate),
            on_type_predicate: self
                .on_type_predicate
                .iter()
                .filter(|(k, _)| !is_full_uri(k))
                .map(|(k, v)| {
                    let filtered_values = filter_out_full_uris(v);
                    (k.clone(), filtered_values)
                })
                .collect(),
        }
    }

    /// Checks if the provided cURIs for objects can be expanded given the prefixes provided
    pub fn expand_curies(&self, prefixes: &PrefixMap) -> Result<ObjectRules, anyhow::Error> {
        let mut objects_on_predicate_full_uris = keep_full_uris(&self.on_predicate);
        let objects_predicate_curies = filter_out_full_uris(&self.on_predicate);
        let mut objects_predicate_curies_set = CompactUriSet::new();

        objects_predicate_curies_set.insert(&objects_predicate_curies);

        let valid_object_predicate_curies = objects_predicate_curies_set.expand_set(prefixes)?;

        objects_on_predicate_full_uris.extend(valid_object_predicate_curies);

        let mut objects_on_type_predicate_full_uris: HashMap<String, HashSet<String>> =
            HashMap::new();

        for (k, v) in self.on_type_predicate.iter() {
            let expanded_keys: Result<String, PrefixError> = match is_full_uri(k) {
                false => {
                    let valid_key = prefixes.expand_curie(k)?;
                    Ok(format!("<{}>", valid_key))
                }
                true => Ok(k.clone()),
            };

            let mut expanded_values = keep_full_uris(v);
            let objects_type_predicate_curies = filter_out_full_uris(v);
            let mut objects_type_predicate_curies_set = CompactUriSet::new();

            objects_type_predicate_curies_set.insert(&objects_type_predicate_curies);

            let valid_objects_type_predicate_curies =
                objects_type_predicate_curies_set.expand_set(prefixes)?;

            expanded_values.extend(valid_objects_type_predicate_curies);
            if let Ok(expanded_key) = expanded_keys {
                objects_on_type_predicate_full_uris.insert(expanded_key, expanded_values);
            } else {
                return Err(anyhow::anyhow!("Failed to expand key: {:?}", expanded_keys));
            }
        }

        Ok(ObjectRules {
            on_predicate: objects_on_predicate_full_uris,
            on_type_predicate: objects_on_type_predicate_full_uris,
        })
    }
}

/// Rules for pseudonymizing triples
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Rules {
    // Invert all matchings
    #[serde(default)]
    pub invert: bool,

    #[serde(default)]
    prefixes: Option<HashMap<String, String>>,

    #[serde(default)]
    pub nodes: NodeRules,

    #[serde(default)]
    pub objects: ObjectRules,
}

/// Check if rules are setup correctly
impl Rules {
    pub fn validate_uris(&self) -> Result<(), anyhow::Error> {
        self.prefixes.as_ref().map_or_else(
            // If no prefix are set, check each URI for validity
            || {
                check_uris(&self.nodes.of_type).map_err(Error::from)?;
                check_uris(&self.objects.on_predicate).map_err(Error::from)?;
                for (k, v) in &self.objects.on_type_predicate {
                    check_uri(k).map_err(Error::from)?;
                    check_uris(v).map_err(Error::from)?;
                }
                Ok::<(), anyhow::Error>(())
            },
            // If prefixes are set, build prefix map, try expanding
            // and check both compact URIs and full URIs
            |p| {
                let mut prefix_map = PrefixMap::new();
                prefix_map.import_hashmap(p);
                self.nodes.check_curies(&prefix_map).map_err(Error::from)?;
                self.objects
                    .keep_curies()
                    .check_curies(&prefix_map)
                    .map_err(Error::from)?;
                self.nodes.check_uris().map_err(Error::from)?;
                self.objects.check_uris().map_err(Error::from)?;
                Ok(())
            },
        )?;
        Ok(())
    }

    pub fn expand_rules_curie(&self) -> Result<Rules, anyhow::Error> {
        self.validate_uris()?;
        match self.prefixes.as_ref() {
            // If there's no prefixes return Rules as they are
            None => {
                return Ok(Rules {
                    invert: self.invert,
                    prefixes: self.prefixes.clone(),
                    nodes: NodeRules {
                        of_type: self.nodes.of_type.clone(),
                    },
                    objects: ObjectRules {
                        on_predicate: self.objects.on_predicate.clone(),
                        on_type_predicate: self.objects.on_type_predicate.clone(),
                    },
                });
            }
            // If there's prefixes, return expanded cURIs and full URIs
            Some(p) => {
                let mut prefix_map = PrefixMap::new();
                prefix_map.import_hashmap(p);
                return Ok(Rules {
                    invert: self.invert,
                    prefixes: self.prefixes.clone(),
                    nodes: self.nodes.expand_curies(&prefix_map)?,
                    objects: self.objects.expand_curies(&prefix_map)?,
                });
            }
        }
    }
}

/// Check all parts of the triple against rules.
pub fn match_rules(triple: &Triple, rules: &Rules, type_map: &mut TypeIndex) -> TripleMask {
    let mut mask =
        match_node_rules(triple, rules, type_map) | match_object_rules(triple, rules, type_map);

    if rules.invert {
        mask = mask.invert();
    }

    mask
}

/// Check triple against node-pseudonymization rules.
pub fn match_node_rules(triple: &Triple, rules: &Rules, type_map: &mut TypeIndex) -> TripleMask {
    let pseudo_subject = match &triple.subject {
        Subject::NamedNode(n) => match_type(&n.to_string(), rules, type_map),
        Subject::BlankNode(_) => false,
    };
    let pseudo_object = match &triple.object {
        Term::NamedNode(n) => match_type(&n.to_string(), rules, type_map),
        Term::BlankNode(_) => false,
        Term::Literal(_) => false,
    };

    let mut mask = TripleMask::default();
    if pseudo_subject {
        mask |= TripleMask::SUBJECT;
    };
    if pseudo_object {
        mask |= TripleMask::OBJECT;
    };

    mask
}

/// Checks triple against object-pseudonymization rules
pub fn match_object_rules(triple: &Triple, rules: &Rules, type_map: &mut TypeIndex) -> TripleMask {
    if match_predicate(&triple.predicate.to_string(), rules) {
        return TripleMask::OBJECT;
    }

    let pseudo_object = match &triple.subject {
        Subject::NamedNode(n) => match_type_predicate(
            &n.to_string(),
            &triple.predicate.to_string(),
            type_map,
            rules,
        ),
        Subject::BlankNode(b) => match_type_predicate(
            &b.to_string(),
            &triple.predicate.to_string(),
            type_map,
            rules,
        ),
    };

    if pseudo_object {
        return TripleMask::OBJECT;
    }

    TripleMask::default()
}

/// Check if the type of input instance URI is in the rules.
fn match_type(subject: &str, rules: &Rules, type_map: &mut TypeIndex) -> bool {
    if let Some(v) = type_map.get(subject) {
        v.iter().any(|&i| rules.nodes.of_type.contains(i))
    } else {
        false
    }
}

/// Check if the predicate URI is in the rules.
fn match_predicate(predicate: &str, rules: &Rules) -> bool {
    rules.objects.on_predicate.contains(predicate)
}

/// Check if the combination of subject type and predicate URIs is in the rules.
fn match_type_predicate(
    subject: &str,
    predicate: &str,
    type_map: &mut TypeIndex,
    rules: &Rules,
) -> bool {
    let Some(instance_types) = type_map.get(subject) else {
        return false;
    };

    for typ in instance_types {
        if let Some(preds) = rules.objects.on_type_predicate.get(typ) {
            if preds.contains(predicate) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use rio_api::parser::TriplesParser;
    use rio_turtle::{TurtleError, TurtleParser};
    use rstest::rstest;

    // Instance used in tests
    const NODE_IRI: &str = "<Alice>";
    const PREDICATE_IRI: &str = "<hasName>";

    // Helper macro to create a HashMap from pairs
    #[macro_export]
    macro_rules! index {
    () => {
            TypeIndex::new()
        };

        ($($key:expr => $value:expr),+ $(,)?) => {
            TypeIndex::from_iter(
                vec![
                $(($key, $value)),*
            ].into_iter())
        };
    }

    fn parse_rules(yml: &str) -> Rules {
        serde_yml::from_str(yml).unwrap()
    }

    #[rstest]
    // Subject is in the rules & type index
    #[case(index! { NODE_IRI => "<Person>" }, "<Person>", true)]
    // Subject is in the type index, not in the rules
    #[case(index! { NODE_IRI => "<Person>" }, "<Bank>", false)]
    // Subject is not in the type index
    #[case(index! { "<BankName>" => "<Bank>" }, "<Bank>", false)]
    fn type_rule(
        #[case] mut index: TypeIndex,
        #[case] rule_type: &str,
        #[case] match_expected: bool,
    ) {
        let rules = parse_rules(&format!(
            "
            nodes:
              of_type:
              - {rule_type}
        "
        ));

        assert_eq!(match_type(NODE_IRI, &rules, &mut index), match_expected);
    }

    #[rstest]
    // Predicate is in the rules
    #[case(PREDICATE_IRI, true)]
    // Predicate is not in the rules
    #[case("hasAge", false)]
    fn predicate_rule(#[case] rule_predicate: &str, #[case] match_expected: bool) {
        let rules = parse_rules(&format!(
            "
            objects:
              on_predicate:
              - {rule_predicate}
        "
        ));
        assert_eq!(match_predicate(PREDICATE_IRI, &rules), match_expected);
    }

    #[rstest]
    // Subject predicate in config
    #[case("<Person>", "<hasName>", index! { NODE_IRI => "<Person>" }, true)]
    // Subject in config, predicate not
    #[case("<Person>", "<hasAge>", index! { NODE_IRI => "<Person>" }, false)]
    // Subject predicate not in config
    #[case("<Bob>", "<hasAge>", index! { NODE_IRI => "<Person>" }, false)]
    // Subject not in type index
    #[case("<Bob>", "<hasAge>", index! { "<Bob>" => "<Person>" }, false)]
    fn type_predicate_rule(
        #[case] rule_type: &str,
        #[case] rule_predicate: &str,
        #[case] mut index: TypeIndex,
        #[case] match_expected: bool,
    ) {
        let rules = parse_rules(&format!(
            "
            objects:
              on_type_predicate:
                {rule_type}:
                - {rule_predicate}
        "
        ));

        assert_eq!(
            match_type_predicate(NODE_IRI, PREDICATE_IRI, &mut index, &rules),
            match_expected
        );
    }

    #[rstest]
    // sensitive subject, on-type sensitive object
    #[case(r#"<urn:Alice> <urn:hasAge> "42" ."#, 0b101)]
    // sensitive subject, sensitive literal object
    #[case(r#"<urn:Alice> <urn:hasLastName> "Foobar" ."#, 0b101)]
    // sensitive subject, sensitive named node object
    #[case(r#"<urn:Alice> <urn:hasFriend> <urn:Bob> ."#, 0b101)]
    // non-sensitive subject, sensitive named node object
    #[case(r#"<urn:ACME> <urn:hasEmployee> <urn:Bob> ."#, 0b001)]
    // non-sensitive subject, non-sensitive object
    #[case(r#"<urn:ACME> <urn:hasAge> "200" ."#, 0b000)]
    // Test the parsing of different triples against fixed rules/index.
    fn individual_triple(#[case] triple: &str, #[case] expected_mask: u8) {
        let rules: Rules = parse_rules(
            r#"
            nodes:
              of_type: ["<urn:Person>"]
            objects:
              on_predicate: ["<urn:hasLastName>"]
              on_type_predicate:
                "<urn:Person>": ["<urn:hasAge>"]
            "#,
        );
        let mut index = index! {
            "<urn:Alice>" => "<urn:Person>",
            "<urn:Bob>" => "<urn:Person>",
            "<urn:ACME>" => "<urn:Organization>"
        };
        println!("{}", serde_yml::to_string(&rules).unwrap());
        println!("{}", serde_json::to_string(&index).unwrap());
        TurtleParser::new(triple.as_ref(), None)
            .parse_all(&mut |t| {
                let mask = match_rules(&t.into(), &rules, &mut index);
                assert_eq!(mask.bits(), expected_mask);
                Ok(()) as Result<(), TurtleError>
            })
            .unwrap();
    }
    #[rstest]
    // Prefix provided with matching cURIes
    #[case("ex", "<http://example.org/>", "ex:Person", "ex:hasName>", true)]
    // Prefix provided with non-matching cURIes
    #[case("ex", "<http://example.org/>", "foaf:Person", "foaf:hasAge>", false)]
    // Prefix provided with full URIs in config
    #[case("ex", "<http://example.org/>", "<http:Person>", "<http:hasName>", true)]
    // Prefix badly defined
    #[case("ex", "http://example.org/", "ex:Person", "ex:hasName>", false)]
    // Bad full URIs provided
    #[case("ex", "<http://example.org/>", "<Person>", "<http:hasName>", false)]
    // No default prefix provided
    #[case("ex", "<http://example.org/>", "Person", "<http:hasName>", false)]
    // Valid default provided
    #[case("default", "<http://example.org/>", "Person", "<http:hasName>", true)]
    fn valid_curies(
        #[case] prefixes: &str,
        #[case] prefixes_uri: &str,
        #[case] rule_type: &str,
        #[case] rule_predicate: &str,
        #[case] match_expected: bool,
    ) {
        let rules = parse_rules(&format!(
            "
            prefixes:
              {prefixes}: {prefixes_uri}
            objects:
              on_type_predicate:
                {rule_type}:
                - {rule_predicate}
        "
        ));
        assert_eq!(rules.validate_uris().is_ok(), match_expected);
    }
    #[rstest]
    // Prefix provided with matching cURIes
    #[case(
        "ex",
        "<http://example.org/>",
        "ex:Person",
        "ex:hasName",
        "<http://example.org/Person>",
        "<http://example.org/hasName>"
    )]
    // Prefix provided with full URIs
    #[case(
        "ex",
        "<http://example.org/>",
        "<http://example.org/Person>",
        "<http://example.org/hasName>",
        "<http://example.org/Person>",
        "<http://example.org/hasName>"
    )]
    fn expand_rules(
        #[case] prefixes: &str,
        #[case] prefixes_uri: &str,
        #[case] rule_type: &str,
        #[case] rule_predicate: &str,
        #[case] expanded_rule_type: &str,
        #[case] expanded_rule_predicate: &str,
    ) {
        let rules = parse_rules(&format!(
            "
            prefixes:
              {prefixes}: {prefixes_uri}
            objects:
              on_type_predicate:
                {rule_type}:
                - {rule_predicate}
        "
        ));
        let expanded = rules.expand_rules_curie();
        println!("Expanded rules: {:?} ", expanded.as_ref().unwrap());
        assert!(
            expanded.unwrap().objects.on_type_predicate[expanded_rule_type]
                .contains(expanded_rule_predicate)
        );
    }
}
