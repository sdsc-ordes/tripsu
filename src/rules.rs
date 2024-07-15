use crate::rdf_types::*;
use ::std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::model::TripleMask;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Rules {
    // Replace values of nodes with a certain type.
    pub replace_uri_of_nodes_with_type: HashSet<String>,

    // Replace values of `subject` & `predicate`.
    pub replace_values_of_subject_predicate: HashMap<String, HashSet<String>>,

    // Replace values in matched `predicates`.
    pub replace_value_of_predicate: HashSet<String>,
}

pub fn match_type_rule_named_node(
    is_subject: bool,
    n: &NamedNode,
    mask: TripleMask,
    rules: &Rules,
    type_map: &HashMap<String, String>,
) -> TripleMask {
    let iri_type = if let Some(v) = type_map.get(&n.iri) {
        v
    } else {
        // Not in the type map.
        return mask;
    };

    if !rules.replace_uri_of_nodes_with_type.contains(iri_type) {
        // Not in the rules.
        return mask;
    }

    return if is_subject {
        mask | TripleMask::SUBJECT
    } else {
        mask | TripleMask::OBJECT
    };
}

pub fn match_type_rule_subject(
    subject: &Subject,
    mask: TripleMask,
    type_map: &HashMap<String, String>,
    rules: &Rules,
) -> TripleMask {
    match subject {
        Subject::NamedNode(n) => {
            return mask | match_type_rule_named_node(true, n, mask, rules, type_map);
        }
        Subject::BlankNode(_) => return mask,
    }
}

pub fn match_type_rule_object(
    object: &Term,
    mask: TripleMask,
    type_map: &HashMap<String, String>,
    rules: &Rules,
) -> TripleMask {
    match object {
        Term::NamedNode(n) => {
            return mask | match_type_rule_named_node(false, n, mask, rules, type_map);
        }
        _ => return mask,
    }
}

pub fn match_predicate_rule(predicate: &NamedNode, mask: TripleMask, rules: &Rules) -> TripleMask {
    let NamedNode { iri: i } = predicate;

    if rules.replace_value_of_predicate.contains(i) {
        return mask | TripleMask::OBJECT;
    } else {
        return mask;
    }
}

pub fn match_subject_predicate_rule(
    subject: &Subject,
    predicate: &NamedNode,
    mask: TripleMask,
    type_map: &HashMap<String, String>,
    rules: &Rules,
) -> TripleMask {
    match subject {
        Subject::NamedNode(n) => {
            let subject_type = if let Some(v) = type_map.get(&n.iri) {
                v
            } else {
                // Not in the type map.
                return mask;
            };

            let preds = rules.replace_values_of_subject_predicate.get(subject_type);
            if preds.is_none() || !preds.unwrap().contains(&predicate.iri) {
                // Not in the rules.
                return mask;
            }

            return mask | TripleMask::OBJECT;
        }
        Subject::BlankNode(_) => return mask,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_type_rule(t: &str) -> Rules {
        let mut rules = Rules::default();

        rules.replace_uri_of_nodes_with_type.insert(t.to_string());
        return rules;
    }

    fn set_predicate_rule(p: &str) -> Rules {
        let mut rules = Rules {
            replace_uri_of_nodes_with_type: HashSet::new(),
            replace_values_of_subject_predicate: HashMap::new(),
            replace_value_of_predicate: HashSet::new(),
        };
        rules.replace_value_of_predicate.insert(p.to_string());
        return rules;
    }

    fn set_subject_predicate_rule(s: &str, p: &str) -> Rules {
        let mut rules = Rules {
            replace_uri_of_nodes_with_type: HashSet::new(),
            replace_values_of_subject_predicate: HashMap::new(),
            replace_value_of_predicate: HashSet::new(),
        };

        let mut set = HashSet::new();
        set.insert(p.to_string());

        rules
            .replace_values_of_subject_predicate
            .insert(s.to_string(), set);

        return rules;
    }

    #[test]
    // Test the type rule for a subject that is in the rules & type map
    fn is_subject_set() {
        let subject = Subject::NamedNode(NamedNode {
            iri: "http://example.org/Alice".to_string(),
        });
        let rules = set_type_rule("Person");
        let mut type_map = HashMap::new();
        type_map.insert("http://example.org/Alice".to_string(), "Person".to_string());
        let mut mask = TripleMask::default();
        mask = match_type_rule_subject(&subject, mask, &type_map, &rules);
        assert!(mask.is_set(&TripleMask::SUBJECT));
        assert!(!mask.is_set(&TripleMask::OBJECT));
    }
    #[test]
    // Test the type rule for a subject that is not in the rules but in the type map
    fn subject_not_in_rules() {
        let subject = Subject::NamedNode(NamedNode {
            iri: "http://example.org/Alice".to_string(),
        });
        let rules = set_type_rule("Bank");
        let mut type_map = HashMap::new();
        type_map.insert("http://example.org/Alice".to_string(), "Person".to_string());
        let mut mask = TripleMask::default();

        mask = match_type_rule_subject(&subject, mask, &type_map, &rules);

        assert!(!mask.is_set(&TripleMask::SUBJECT));
        assert!(!mask.is_set(&TripleMask::OBJECT));
    }

    #[test]
    // Test the type rule for a subject neither in the rules nor in the type map
    fn subject_not_in_types() {
        let subject = Subject::NamedNode(NamedNode {
            iri: "http://example.org/Alice".to_string(),
        });
        let rules = set_type_rule("Bank");
        let mut type_map = HashMap::new();
        type_map.insert("http://example.org/Bank".to_string(), "Bank".to_string());
        let mut mask = TripleMask::default();

        mask = match_type_rule_subject(&subject, mask, &type_map, &rules);

        assert!(!mask.is_set(&TripleMask::SUBJECT));
        assert!(!mask.is_set(&TripleMask::OBJECT));
    }
    #[test]
    // Test the type rule for an object that is in the rules & type map
    fn is_object_set() {
        let object = Term::NamedNode(NamedNode {
            iri: "http://example.org/Alice".to_string(),
        });
        let rules = set_type_rule("Person");
        let mut type_map = HashMap::new();
        type_map.insert("http://example.org/Alice".to_string(), "Person".to_string());
        let mut mask = TripleMask::default();

        mask = match_type_rule_object(&object, mask, &type_map, &rules);

        assert!(mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }

    #[test]
    fn predicate_in_config() {
        let predicate = NamedNode {
            iri: "http://example.org/hasName".to_string(),
        };
        let rules = set_predicate_rule("http://example.org/hasName");
        let mut mask = TripleMask::default();

        mask = match_predicate_rule(&predicate, mask, &rules);

        assert!(mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }

    #[test]
    fn predicate_not_in_config() {
        let predicate = NamedNode {
            iri: "http://example.org/hasName".to_string(),
        };
        let rules = set_predicate_rule("http://example.org/hasAge");
        let mut mask = TripleMask::default();

        mask = match_predicate_rule(&predicate, mask, &rules);

        assert!(!mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }

    #[test]
    fn subject_predicate_in_config() {
        let alice_iri = "http://example.org/Alice";
        let person_iri = "http://example.org/Person";
        let pred_hn = "http://example.org/hasName";

        let subject = Subject::NamedNode(NamedNode {
            iri: alice_iri.to_string(),
        });
        let predicate = NamedNode {
            iri: pred_hn.to_string(),
        };

        let rules =
            set_subject_predicate_rule("http://example.org/Person", "http://example.org/hasName");

        let mut mask = TripleMask::default();
        let mut type_map = HashMap::new();
        type_map.insert(alice_iri.to_string(), person_iri.to_string());

        mask = match_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);

        assert!(mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }

    #[test]
    fn subject_in_config_predicate_not_() {
        let subject = Subject::NamedNode(NamedNode {
            iri: "http://example.org/Alice".to_string(),
        });
        let predicate = NamedNode {
            iri: "http://example.org/hasName".to_string(),
        };
        let rules =
            set_subject_predicate_rule("http://example.org/Alice", "http://example.org/hasAge");
        let mut mask = TripleMask::default();
        let mut type_map = HashMap::new();
        type_map.insert(
            "http://example.org/Alice".to_string(),
            "http://example.org/Person".to_string(),
        );

        mask = match_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);

        assert!(!mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }

    #[test]
    fn subject_predicate_not_in_config() {
        let subject = Subject::NamedNode(NamedNode {
            iri: "http://example.org/Alice".to_string(),
        });
        let predicate = NamedNode {
            iri: "http://example.org/hasName".to_string(),
        };
        let rules =
            set_subject_predicate_rule("http://example.org/Bob", "http://example.org/hasAge");
        let mut mask = TripleMask::default();
        let mut type_map = HashMap::new();
        type_map.insert(
            "http://example.org/Alice".to_string(),
            "http://example.org/Person".to_string(),
        );

        mask = match_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);

        assert!(!mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }

    #[test]
    // Rule subject predicate where subject is not in type list
    fn subject_predicate_not_in_types() {
        let subject = Subject::NamedNode(NamedNode {
            iri: "http://example.org/Alice".to_string(),
        });
        let predicate = NamedNode {
            iri: "http://example.org/hasName".to_string(),
        };
        let rules =
            set_subject_predicate_rule("http://example.org/Bob", "http://example.org/hasAge");

        let mut mask = TripleMask::default();
        let mut type_map = HashMap::new();

        type_map.insert(
            "http://example.org/Bob".to_string(),
            "http://example.org/Person".to_string(),
        );

        mask = match_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);

        assert!(!mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }
}
