use crate::rdf_types::*;
use ::std::collections::{HashMap, HashSet};
use curie::{Curie, PrefixMapping};
use serde::{de::value, Deserialize, Serialize};
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
    pub fn is_empty(&self) -> bool {
        // Check if any rules are set
        if self.nodes.of_type.len() > 0
            || self.objects.on_predicate.len() > 0
            || self.objects.on_type_predicate.len() > 0
        {
            return false;
        }
        return true;
    }
    pub fn has_valid_curies_and_uris(&self) -> bool {
        match &self.prefixes {
            // If no prefixes are set, check each URI for validity
            None => return self.check_uris(&self.nodes, &self.objects),
            // If some prefixes are set, check and expand each URI for validity
            Some(prefixes) => {
                let mut prefix_map = PrefixMapping::default();
                for prefix in prefixes {
                    match prefix_map.add_prefix(prefix.0, prefix.1) {
                        Ok(_) => continue,
                        Err(_) => return false,
                    }
                }
                return self.check_curies(&self.nodes, &self.objects, prefix_map);
            }
        }
    }

    fn to_curie<'a>(&self, uri: &'a str) -> Curie<'a> {
        let separator_idx = uri
            .chars()
            .position(|c| c == ':')
            .expect("No separator found in URI found in string");
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
        return node_uris.of_type.iter().all(|uri| {
            let curie = self.to_curie(uri);
            match prefixes.expand_curie(&curie) {
                Ok(_) => return true,
                Err(_) => return false,
            };
        }) && object_uris.on_predicate.iter().all(|uri| {
            let curie = self.to_curie(uri);
            match prefixes.expand_curie(&curie) {
                Ok(_) => return true,
                Err(_) => return false,
            };
        }) && object_uris.on_type_predicate.iter().all(|(k, v)| {
            let expanded_key = self.to_curie(k);
            let key_valid = match prefixes.expand_curie(&expanded_key) {
                            Ok(_) => true,
                            Err(_) => false,
                        };
            let value_valid = v.iter().all(|uri| {
                let curie = self.to_curie(uri);
                match prefixes.expand_curie(&curie) {
                    Ok(_) => true,
                    Err(_) => false,
                }
            });
            return key_valid && value_valid;
        });
    }

    pub fn expand_curie(&self) -> Rules {
        let prefix_map = match &self.prefixes {
            None => PrefixMapping::default(),
            Some(prefixes) => {
                let mut prefix_map = PrefixMapping::default();
                for prefix in prefixes {
                    if let Err(e) = prefix_map.add_prefix(&prefix.0, &prefix.1) {
                        eprintln!("Failed to add prefix: {:?}", e);
                    }
                }
                prefix_map
            }
        };
        return Rules {
            invert: self.invert,
            prefixes: self.prefixes.clone(),
            nodes: NodeRules {
                of_type: { self.expand_hashset(&self.nodes.of_type, &prefix_map) },
            },
            objects: ObjectRules {
                on_predicate: { self.expand_hashset(&self.objects.on_predicate, &prefix_map) },
                on_type_predicate: {
                    let mut expanded_type_predicate = HashMap::new();
                    for (k, v) in self.objects.on_type_predicate.iter() {
                        let expanded_key = self.expand_string(k, &prefix_map);
                        let expanded_value = self.expand_hashset(v, &prefix_map);
                        expanded_type_predicate.insert(expanded_key, expanded_value);
                    }
                    expanded_type_predicate
                },
            },
        };
    }
    fn expand_hashset(&self, set: &HashSet<String>, prefix_map: &PrefixMapping) -> HashSet<String> {
        let mut expanded_set = HashSet::new();
        for uri in set.iter() {
            let expanded_uri = self.expand_string(&uri, prefix_map);
            expanded_set.insert(expanded_uri);
        }
        return expanded_set;
    }
    fn expand_string(&self, uri: &str, prefix_map: &PrefixMapping) -> String {
        let separator_idx = uri
            .chars()
            .position(|c| c == ':')
            .expect("No separator found in URI");
        let prefix = Some(&uri[..separator_idx]);
        let reference = &uri[separator_idx + 1..];
        let curie = if prefix != Some("http") {
            Curie::new(prefix, reference)
        } else {
            Curie::new(None, uri)
        };
        return prefix_map.expand_curie(&curie).unwrap().to_string();
    }
    fn check_uris(&self, nodes: &NodeRules, objects: &ObjectRules) -> bool {
        // Check if the URIs are valid and there are no cURIEs
        return nodes
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
            });
    }
    fn check_string_iri(&self, uri: &str) -> bool {
        match Iri::new(uri) {
            Ok(_) => return true,
            Err(_) => return false,
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

    return mask;
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

    return mask;
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

    return TripleMask::default();
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
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rio_api::parser::TriplesParser;
    use rio_turtle::{TurtleError, TurtleParser};
    use rstest::rstest;
    use serde_yml;

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
    fn empty_rules() {
        let rules: Rules = parse_rules(
            r#"
            nodes:
              of_type: 
            objects:
              on_predicate: 
              on_type_predicate:
            "#,
        );
        assert!(rules.is_empty());
    }
    #[rstest]
    fn valid_full_uri() {
        let rules: Rules = parse_rules(
            r#"
            nodes:
              of_type: ["http:Person"]
            objects:
              on_predicate: ["http:hasLastName"]
              on_type_predicate:
                "http:Person": ["http:hasAge"]
            "#,
        );
        rules.check_uri(&rules.nodes, &rules.objects, None);
    }
}
