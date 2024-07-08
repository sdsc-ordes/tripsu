use crate::rdf_types::*;
use ::std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::model::TripleMask;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    // Replace values of nodes with a certain type.
    pub replace_uri_of_nodes_with_type: HashSet<String>,

    // Replace values of `subject` & `predicate`.
    pub replace_values_of_subject_predicate: HashMap<String, HashSet<String>>,

    // Replace values in matched `predicates`.
    pub replace_value_of_predicate: HashSet<String>,
}

pub fn eval_type_rule_named_node(
    is_subject: bool,
    n: NamedNode,
    mut mask: TripleMask,
    rules: &Config,
    type_map: &HashMap<String, String>,
) -> TripleMask {
    // First check if the iri is in any of the types in the type map
    let iri_type = if type_map.contains_key(&n.iri) {
        type_map.get(&n.iri).unwrap()
    } else {
        return mask;
    };
    // If the iri is in the type map, check if it is in the rules
    if rules.replace_uri_of_nodes_with_type.contains(iri_type) {
        if is_subject {
            mask |= TripleMask::SUBJECT;
            return mask;
        } else {
            mask |= TripleMask::OBJECT;
            return mask;
        };
    } else {
        return mask;
    }
}

pub fn eval_type_rule_subject(
    subject: &Subject,
    mut mask: TripleMask,
    type_map: &HashMap<String, String>,
    rules: &Config,
) -> TripleMask {
    match subject {
        Subject::NamedNode(n) => {
            mask = eval_type_rule_named_node(true, n.clone(), mask, rules, type_map);
            return mask;
        }
        Subject::BlankNode(_) => return mask,
    }
}

pub fn eval_type_rule_object(
    object: &Term,
    mut mask: TripleMask,
    type_map: &HashMap<String, String>,
    rules: &Config,
) -> TripleMask {
    match object {
        Term::NamedNode(n) => {
            mask = eval_type_rule_named_node(false, n.clone(), mask, rules, type_map);
            return mask;
        }
        _ => return mask,
    }
}

pub fn eval_predicate_rule(
    predicate: &NamedNode,
    mut mask: TripleMask,
    rules: &Config,
) -> TripleMask {
    match predicate {
        NamedNode { iri: n } => {
            // check if rule contains iri is in replace_value_of_predicate
            if rules.replace_value_of_predicate.contains(n) {
                mask |= TripleMask::OBJECT;
                return mask;
            } else {
                return mask;
            }
        }
    }
}

pub fn eval_subject_predicate_rule(
    subject: &Subject,
    predicate: &NamedNode,
    mut mask: TripleMask,
    type_map: &HashMap<String, String>,
    rules: &Config,
) -> TripleMask {
    match subject {
        Subject::NamedNode(n) => {
            // check if rule contains iri is in replace_value_of_subject_predicate
            let subject_type = if type_map.contains_key(&n.iri) {
                type_map.get(&n.iri).unwrap()
            } else {
                return mask;
            };
            if rules.replace_values_of_subject_predicate.contains_key(subject_type) {
                let is_in_config = rules.replace_values_of_subject_predicate[subject_type]
                    .contains(&predicate.iri);
                if is_in_config {
                    mask |= TripleMask::OBJECT;
                    return mask;
                } else {
                    return mask;
                }
            } else {
                return mask;
            }
        }
        Subject::BlankNode(_) => return mask,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_type_rule(t: &str) -> Config {
        let mut rules = Config {
            replace_uri_of_nodes_with_type: HashSet::new(),
            replace_values_of_subject_predicate: HashMap::new(),
            replace_value_of_predicate: HashSet::new(),
        };
        rules.replace_uri_of_nodes_with_type.insert(t.to_string());
        return rules;
    }

    fn set_predicate_rule(p: &str) -> Config {
        let mut rules = Config {
            replace_uri_of_nodes_with_type: HashSet::new(),
            replace_values_of_subject_predicate: HashMap::new(),
            replace_value_of_predicate: HashSet::new(),
        };
        rules.replace_value_of_predicate.insert(p.to_string());
        return rules;
    }

    fn set_subject_predicate_rule(s: &str, p: &str) -> Config {
        let mut rules = Config {
            replace_uri_of_nodes_with_type: HashSet::new(),
            replace_values_of_subject_predicate: HashMap::new(),
            replace_value_of_predicate: HashSet::new(),
        };
        let mut set = HashSet::new();
        set.insert(p.to_string());
        rules.replace_values_of_subject_predicate.insert(s.to_string(), set);
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
        let mut mask = TripleMask::new();
        mask = eval_type_rule_subject(&subject, mask, &type_map, &rules);
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
        let mut mask = TripleMask::new();
        mask = eval_type_rule_subject(&subject, mask, &type_map, &rules);
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
        let mut mask = TripleMask::new();
        mask = eval_type_rule_subject(&subject, mask, &type_map, &rules);
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
        let mut mask = TripleMask::new();
        mask = eval_type_rule_object(&object, mask, &type_map, &rules);
        assert!(mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }
    #[test]
    fn predicate_in_config() {
        let predicate = NamedNode {
            iri: "http://example.org/hasName".to_string(),
        };
        let rules = set_predicate_rule("http://example.org/hasName");
        let mut mask = TripleMask::new();
        mask = eval_predicate_rule(&predicate, mask, &rules);
        assert!(mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }
    #[test]
    fn predicate_not_in_config() {
        let predicate = NamedNode {
            iri: "http://example.org/hasName".to_string(),
        };
        let rules = set_predicate_rule("http://example.org/hasAge");
        let mut mask = TripleMask::new();
        mask = eval_predicate_rule(&predicate, mask, &rules);
        assert!(!mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }
    #[test]
    fn subject_predicate_in_config() {
        let subject = Subject::NamedNode(NamedNode {
            iri: "http://example.org/Alice".to_string(),
        });
        let predicate = NamedNode {
            iri: "http://example.org/hasName".to_string(),
        };
        let rules = set_subject_predicate_rule("http://example.org/Person", "http://example.org/hasName");
        let mut mask = TripleMask::new();
        let mut type_map = HashMap::new();
        type_map.insert("http://example.org/Alice".to_string(), "http://example.org/Person".to_string());
        mask = eval_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);
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
        let rules = set_subject_predicate_rule("http://example.org/Alice", "http://example.org/hasAge");
        let mut mask = TripleMask::new();
        let mut type_map = HashMap::new();
        type_map.insert("http://example.org/Alice".to_string(), "http://example.org/Person".to_string());
        mask = eval_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);
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
        let rules = set_subject_predicate_rule("http://example.org/Bob", "http://example.org/hasAge");
        let mut mask = TripleMask::new();
        let mut type_map = HashMap::new();
        type_map.insert("http://example.org/Alice".to_string(), "http://example.org/Person".to_string());
        mask = eval_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);
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
        let rules = set_subject_predicate_rule("http://example.org/Bob", "http://example.org/hasAge");
        let mut mask = TripleMask::new();
        let mut type_map = HashMap::new();
        type_map.insert("http://example.org/Bob".to_string(), "http://example.org/Person".to_string());
        mask = eval_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);
        assert!(!mask.is_set(&TripleMask::OBJECT));
        assert!(!mask.is_set(&TripleMask::SUBJECT));
    }
}
