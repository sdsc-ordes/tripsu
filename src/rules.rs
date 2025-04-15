use crate::rdf_types::*;
use ::std::collections::{HashMap, HashSet};
use curie::{Curie, PrefixMapping};
use serde::{Deserialize, Serialize};
use sophia_iri::Iri;

use crate::{index::TypeIndex, model::TripleMask};

/// Rules for pseudonymizing nodes
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NodeRules {
    // Replace values of nodes with a certain type.
    #[serde(default)]
    of_type: HashSet<String>,
}

/// Rules for pseudonymizing objects
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ObjectRules {
    // Replace values in matched `predicates`.
    #[serde(default)]
    on_predicate: HashSet<String>,
    // Replace values of predicates for specific types
    #[serde(default)]
    on_type_predicate: HashMap<String, HashSet<String>>,
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
    pub fn has_valid_curies_and_uris(&self) -> bool {
        match &self.prefixes {
            // If no prefix are set, check each URI for validity
            None => self.check_uris(&self.nodes, &self.objects),
            // If some prefix are set, check and expand each URI for validity
            Some(prefixes) => {
                let mut prefix_map = PrefixMapping::default();
                for p in prefixes {
                    if self.is_full_uri(p.1) {
                        match prefix_map.add_prefix(p.0, p.1) {
                            Ok(_) => continue,
                            Err(e) => {
                                eprintln!("Failed to add prefix: {:?}", e);
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }
                // Need to return checks for both cURIEs and full URIs
                return self.check_curies(
                    &self.filter_nodes(&self.nodes),
                    &self.filter_objects(&self.objects),
                    prefix_map,
                ) && self
                    .nodes
                    .of_type
                    .iter()
                    .all(|uri| self.check_string_iri(uri))
                    && self
                        .objects
                        .on_predicate
                        .iter()
                        .all(|uri| self.check_string_iri(uri))
                    && self.objects.on_type_predicate.iter().all(|(k, v)| {
                        self.check_string_iri(k) && v.iter().all(|uri| self.check_string_iri(uri))
                    });
            }
        }
    }

    fn to_curie<'a>(&self, uri: &'a str) -> Curie<'a> {
        let separator_idx = uri
            .chars()
            .position(|c| c == ':')
            .unwrap_or_else(|| panic!("No separator found in cURI string: {}", uri));
        let prefix = Some(&uri[..separator_idx]);
        let reference = &uri[separator_idx + 1..];
        return Curie::new(prefix, reference);
    }

    fn check_curies(
        &self,
        node_uris: &NodeRules,
        object_uris: &ObjectRules,
        prefixes: PrefixMapping,
    ) -> bool {
        // Use iterators to check if the cURIEs are valid
        return node_uris
            .of_type
            .iter()
            .all(|uri| self.try_expansion(uri, &prefixes))
            && object_uris
                .on_predicate
                .iter()
                .all(|uri| self.try_expansion(uri, &prefixes))
            && object_uris.on_type_predicate.iter().all(|(k, v)| {
                let key_valid = self.try_expansion(k, &prefixes);
                let value_valid = v.iter().all(|uri| self.try_expansion(uri, &prefixes));
                key_valid && value_valid
            });
    }

    fn try_expansion(&self, uri: &str, prefix_map: &PrefixMapping) -> bool {
        let curie = self.to_curie(uri);
        match prefix_map.expand_curie(&curie) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Failed to expand {:?} CURIE: {} ", e, uri);
                false
            }
        }
    }

    pub fn expand_rules_curie(&self) -> Rules {
        let prefix_map = match &self.prefix {
            None => PrefixMapping::default(),
            Some(prefixes) => {
                let mut prefix_map = PrefixMapping::default();
                prefixes.iter().for_each(|(k, v)| {
                    if self.is_full_uri(v) {
                        if let Err(e) = prefix_map.add_prefix(k, &v[1..v.len() - 1]) {
                            eprintln!("Failed to add prefix: {:?}", e);
                        }
                    }
                });
                prefix_map
            }
        };
        // for all rules we combine the full URIs with the expanded URIs
        let mut nodes_full_uris: HashSet<String> = self
            .nodes
            .of_type
            .iter()
            .filter(|uri| self.is_full_uri(uri))
            .cloned()
            .collect();
        nodes_full_uris.extend(self.expand_hashset(&self.filter(&self.nodes.of_type), &prefix_map));

        let mut objects_on_predicate_full_uris: HashSet<String> = self
            .objects
            .on_predicate
            .iter()
            .filter(|uri| self.is_full_uri(uri))
            .cloned()
            .collect();
        objects_on_predicate_full_uris
            .extend(self.expand_hashset(&self.filter(&self.objects.on_predicate), &prefix_map));

        let mut objects_on_type_predicate_full_uris: HashMap<String, HashSet<String>> =
            HashMap::new();
        for (k, v) in self.objects.on_type_predicate.iter() {
            let expanded_key = match self.is_full_uri(k) {
                false => format!("<{}>", prefix_map.expand_curie(&self.to_curie(k)).unwrap()),
                true => k.clone(),
            };
            let mut expanded_value: HashSet<String> = v
                .iter()
                .filter(|uri| self.is_full_uri(uri))
                .cloned()
                .collect();
            expanded_value.extend(self.expand_hashset(&self.filter(v), &prefix_map));
            objects_on_type_predicate_full_uris.insert(expanded_key, expanded_value);
        }

        Rules {
            invert: self.invert,
            prefixes: self.prefixes.clone(),
            nodes: NodeRules {
                of_type: nodes_full_uris,
            },
            objects: ObjectRules {
                on_predicate: objects_on_predicate_full_uris,
                on_type_predicate: objects_on_type_predicate_full_uris,
            },
        }
    }
    fn expand_hashset(&self, set: &HashSet<String>, prefix_map: &PrefixMapping) -> HashSet<String> {
        let mut expanded_set = HashSet::new();
        set.iter().for_each(|uri| {
            let expanded_uri = self.expand_string(uri, prefix_map);
            expanded_set.insert(expanded_uri);
        });
        expanded_set
    }
    fn expand_string(&self, uri: &str, prefix_map: &PrefixMapping) -> String {
        let separator_idx = uri
            .chars()
            .position(|c| c == ':')
            .unwrap_or_else(|| panic!("No separator found in cURI string: {}", uri));
        let prefix = Some(&uri[..separator_idx]);
        let reference = &uri[separator_idx + 1..];
        let curie = Curie::new(prefix, reference);
        match prefix_map.expand_curie(&curie) {
            Ok(expanded) => {
                format!("<{}>", expanded)
            }
            Err(e) => {
                eprintln!("Failed to expand {:?} CURIE: {} ", e, uri);
                uri.to_string()
            }
        }
    }
    fn check_uris(&self, nodes: &NodeRules, objects: &ObjectRules) -> bool {
        // Check if the URIs are valid and there are no cURIEs
        nodes
            .of_type
            .clone()
            .into_iter()
            .all(|uri| self.check_string_iri(&uri))
            && objects
                .on_predicate
                .clone()
                .into_iter()
                .all(|uri| self.check_string_iri(&uri))
            && objects.on_type_predicate.clone().into_iter().all(|(k, v)| {
                self.check_string_iri(&k) && v.into_iter().all(|uri| self.check_string_iri(&uri))
            })
    }
    fn check_string_iri(&self, uri: &str) -> bool {
        // We assume that a full URI starts with "<" and ends with ">"
        // We select the URI within the angle brackets
        Iri::new(&uri[1..uri.len() - 2]).is_ok()
    }

    fn filter(&self, hash_set: &HashSet<String>) -> HashSet<String> {
        // Filter out full URIs
        let filtered = hash_set
            .iter()
            .filter(|uri| !self.is_full_uri(uri))
            .cloned()
            .collect();
        filtered
    }
    fn is_full_uri(&self, uri: &str) -> bool {
        // Ensure that full URI starts with "<" and ends with ">"
        uri.starts_with('<') && uri.ends_with('>')
    }

    fn filter_objects(&self, object_uris: &ObjectRules) -> ObjectRules {
        ObjectRules {
            on_predicate: self.filter(&object_uris.on_predicate),
            on_type_predicate: object_uris
                .on_type_predicate
                .iter()
                .filter(|(k, _)| !self.is_full_uri(k))
                .map(|(k, v)| {
                    let filtered_values = self.filter(v);
                    (k.clone(), filtered_values)
                })
                .collect(),
        }
    }

    fn filter_nodes(&self, node_uris: &NodeRules) -> NodeRules {
        NodeRules {
            of_type: self.filter(&node_uris.of_type),
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
    fn valid_curies(
        #[case] prefixes: &str,
        #[case] prefixes_uri: &str,
        #[case] rule_type: &str,
        #[case] rule_predicate: &str,
        #[case] match_expected: bool,
    ) {
        let rules = parse_rules(&format!(
            "
            prefix:
              {prefixes}: {prefixes_uri}
            objects:
              on_type_predicate:
                {rule_type}:
                - {rule_predicate}
        "
        ));
        assert_eq!(rules.has_valid_curies_and_uris(), match_expected);
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
            prefix:
              {prefixes}: {prefixes_uri}
            objects:
              on_type_predicate:
                {rule_type}:
                - {rule_predicate}
        "
        ));
        let expanded = rules.expand_rules_curie();
        assert!(expanded.objects.on_type_predicate[expanded_rule_type]
            .contains(expanded_rule_predicate));
    }
}
