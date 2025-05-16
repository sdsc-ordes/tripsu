use crate::{index::TypeIndex, model::TripleMask, uris::*};
use ::std::collections::{HashMap, HashSet};
use anyhow::{Error, Result};
use rio_api::model::*;
use serde::{Deserialize, Serialize};

/// Rules for pseudonymizing nodes
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NodeRules {
    // Replace values of nodes with a certain type.
    #[serde(default)]
    of_type: HashSet<String>,
}

impl NodeRules {

    /// Validate all URIs and CURIEs in node rules and ensure they can
    /// be expanded with the provided prefix map.
    pub fn check_uris(&self, prefixes: &PrefixMap) -> Result<(), anyhow::Error> {
        let uris = UriSet::try_from(self.of_type.clone())?;
        uris.expand(prefixes)?;
        Ok(())
    }

    /// Checks if the provided cURIs for nodes can be expanded given the prefixes provided
    pub fn expand_curies(&self, prefixes: &PrefixMap) -> Result<NodeRules, PrefixError> {
        let expanded = UriSet::try_from(self.of_type.clone())
            .unwrap()
            .expand(prefixes)?;

        Ok(NodeRules {
            of_type: expanded.into(),
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

    /// Validate all URIs and CURIEs in node rules and ensure they can
    /// be expanded with the provided prefix map.
    pub fn check_uris(&self, prefixes: &PrefixMap) -> Result<(), anyhow::Error> {
        let uris = UriSet::try_from(self.on_predicate.clone())?;
        uris.expand(prefixes)?;

        for (k, v) in self.on_type_predicate.iter() {
            Uri::try_from(k.clone())?.expand(prefixes);
            UriSet::try_from(v.clone())?.expand(prefixes)?;
        }

        Ok(())
    }


    /// Checks if the provided cURIs for objects can be expanded given the prefixes provided
    pub fn expand_curies(&self, prefixes: &PrefixMap) -> Result<ObjectRules, anyhow::Error> {

        let expanded_on_predicate = UriSet::try_from(self.on_predicate.clone())
            .unwrap()
            .expand(prefixes)?;

        let mut expanded_on_type_predicate = HashMap::<String, HashSet<String>>::new();
        for (k, v) in self.on_type_predicate.iter() {
            let type_key = Uri::try_from(k.clone())?.expand(prefixes)?;
            let pred_values = UriSet::try_from(v.clone())?.expand(prefixes)?;
            expanded_on_type_predicate.insert(
                type_key.to_string(), pred_values.into()
            );
        }

        Ok(ObjectRules{
            on_predicate: expanded_on_predicate.into(),
            on_type_predicate: expanded_on_type_predicate,
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
    prefixes: Option<HashMap<Option<String>, String>>,

    #[serde(default)]
    pub nodes: NodeRules,

    #[serde(default)]
    pub objects: ObjectRules,
}

/// Check if rules are setup correctly
impl Rules {
    pub fn check_uris(&self) -> Result<(), anyhow::Error> {
        // If prefixes are set, build prefix map, try expanding
        // and check both compact URIs and full URIs
        if self.prefixes.is_some() {
            // If prefixes are set, check if they are valid
            let prefix_map = PrefixMap::from_hashmap(&self.prefixes.clone().unwrap())?;
            self.nodes.check_uris(&prefix_map).map_err(Error::from)?;
            self.objects.check_uris(&prefix_map).map_err(Error::from)?;

        // If no prefix are set, check each URI for validity
        } else {
            UriSet::try_from(self.nodes.of_type.clone())?;
            UriSet::try_from(self.objects.on_predicate.clone())?;
            for (k, v) in self.objects.on_type_predicate.iter() {
                Uri::try_from(k.clone())?;
                UriSet::try_from(v.clone())?;
            }
        };
        Ok(())
    }

    pub fn expand_rules_curie(&self) -> Result<Rules, anyhow::Error> {
        self.check_uris()?;
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
                let prefix_map = PrefixMap::from_hashmap(p)?;
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
        Subject::Triple(_) => panic!("RDF-star data not supported"),
    };
    let pseudo_object = match &triple.object {
        Term::NamedNode(n) => match_type(&n.to_string(), rules, type_map),
        Term::BlankNode(_) => false,
        Term::Literal(_) => false,
        Term::Triple(_) => panic!("RDF-star data not supported"),
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
        Subject::Triple(_) => panic!("RDF-star data not supported"),
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
    #[case("\"\"", "<http://example.org/>", ":Person", "<http:hasName>", true)]
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
        assert_eq!(rules.check_uris().is_ok(), match_expected);
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
        assert!(
            expanded.unwrap().objects.on_type_predicate[expanded_rule_type]
                .contains(expanded_rule_predicate)
        );
    }
}
