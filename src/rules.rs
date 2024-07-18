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
    use rstest::rstest;

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

    #[rstest]
    // Subject is in the rules & type index
    #[case(true, "Alice", "Alice", "Person", "Person", true, false)]
    // Subject is in the type index, not in the rules
    #[case(true, "Alice", "Alice", "Person", "Bank", false, false)]
    // Subject is not in the type index
    #[case(true, "Alice", "BankName", "Bank", "Bank", false, false)]
    // Object is in the rules & type index
    #[case(false, "Alice", "Alice", "Person", "Person", false, true)]
    fn type_rule(
        #[case] is_subject: bool,
        #[case] node_iri: &str,
        #[case] index_subject: &str,
        #[case] index_object: &str,
        #[case] rule_type: &str,
        #[case] expected_s: bool,
        #[case] expected_o: bool,
    ) {
        let rules = set_type_rule(rule_type);
        let mut type_index = HashMap::new();
        type_index.insert(index_subject.to_string(), index_object.to_string());
        let mut mask = TripleMask::default();
        mask = if is_subject {
            let node = Subject::NamedNode(NamedNode {
                iri: node_iri.to_string(),
            });
            match_type_rule_subject(&node, mask, &type_index, &rules)
        } else {
            let node = Term::NamedNode(NamedNode {
                iri: node_iri.to_string(),
            });
            match_type_rule_object(&node, mask, &type_index, &rules)
        };
        assert_eq!(mask.is_set(&TripleMask::SUBJECT), expected_s);
        assert_eq!(mask.is_set(&TripleMask::OBJECT), expected_o);
    }

    #[rstest]
    // Predicate is in the rules
    #[case("hasName", "hasName", true)]
    // Predicate is not in the rules
    #[case("hasName", "hasAge", false)]
    fn predicate_rule(
        #[case] node_iri: &str,
        #[case] rule_type: &str,
        #[case] expected_o: bool,
    ) {
        let predicate = NamedNode {
            iri: node_iri.to_string(),
        };
        let rules = set_predicate_rule(rule_type);
        let mut mask = TripleMask::default();

        mask = match_predicate_rule(&predicate, mask, &rules);

        assert!(!mask.is_set(&TripleMask::SUBJECT));
        assert_eq!(mask.is_set(&TripleMask::OBJECT), expected_o);
    }

    #[rstest]
    // Subject predicate in config
    #[case(
        "Alice", "hasName", "Person", "hasName", "Alice", "Person", true
    )]
    // Subject in config, predicate not
    #[case(
        "Alice", "hasName", "Person", "hasAge", "Alice", "Person", false
    )]
    // Subject predicate not in config
    #[case("Alice", "hasName", "Bob", "hasAge", "Alice", "Person", false)]
    // Subject not in type index
    #[case("Alice", "hasName", "Bob", "hasAge", "Bob", "Person", false)]
    fn subject_predicate_rule(
        #[case] subject_iri: &str,
        #[case] predicate_iri: &str,
        #[case] rule_subject: &str,
        #[case] rule_predicate: &str,
        #[case] index_subject: &str,
        #[case] index_object: &str,
        #[case] expected_o: bool,
    ) {
        let subject = Subject::NamedNode(NamedNode {
            iri: subject_iri.to_string(),
        });
        let predicate = NamedNode {
            iri: predicate_iri.to_string(),
        };

        let rules = set_subject_predicate_rule(rule_subject, rule_predicate);

        let mut mask = TripleMask::default();
        let mut type_map = HashMap::new();
        type_map.insert(index_subject.to_string(), index_object.to_string());

        mask = match_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);

        assert!(!mask.is_set(&TripleMask::SUBJECT));
        assert_eq!(mask.is_set(&TripleMask::OBJECT), expected_o);
    }
}
