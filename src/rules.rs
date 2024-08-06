use crate::rdf_types::*;
use ::std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::model::TripleMask;

/// Rules for pseudonymizing subjects
#[derive(Serialize, Deserialize, Debug, Default)]
struct SubjectRules {
    // Replace values of nodes with a certain type.
    of_type: HashSet<String>,
}

/// Rules for pseudonymizing objects
#[derive(Serialize, Deserialize, Debug, Default)]
struct ObjectRules {
    // Replace values in matched `predicates`.
    on_predicate: HashSet<String>,
    // Replace values of predicates for specific types
    on_type_predicate: HashMap<String, HashSet<String>>,
}

/// Rules for pseudonymizing triples
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Rules {
    // Invert all matchings
    pub invert: bool,

    pub subjects: SubjectRules,

    pub objects: ObjectRules,

}

/// Check all parts of the triple against rules.
pub fn match_rules(
    triple: &Triple,
    rules: &Rules,
    type_map: &HashMap<String, String>,
) -> TripleMask {

    let mut mask = 
        match_subject_rules(triple, rules, type_map)
        | match_object_rules(triple, rules, type_map);

    if rules.invert {
        mask = mask.invert();
    }

    return mask;
}

/// Checks subject and object against subject-rules.
pub fn match_subject_rules(
    triple: &Triple,
    rules: &Rules,
    type_map: &HashMap<String, String>,
) -> TripleMask {
    let pseudo_subject = match &triple.subject {
        Subject::NamedNode(n) => {
            match_type(&n.iri, rules, type_map)
        },
        _ => false,
    };
    let pseudo_object = match &triple.object {
        Term::NamedNode(n) => {
            match_type(&n.iri, rules, type_map)
        },
        _ => false,
    };

    let mut mask = TripleMask::default();
    if pseudo_subject {
        mask |=  TripleMask::SUBJECT;
    };
    if pseudo_object {
        mask |= TripleMask::OBJECT;
    };

    return mask
}

/// Checks triple against object rules
pub fn match_object_rules(
    triple: &Triple,
    rules: &Rules,
    type_map: &HashMap<String, String>,
) -> TripleMask {
    let pseudo_object = match &triple.object {
        Term::NamedNode(n) => {
            if match_predicate(&n.iri, rules) {
                true
            } else {
                match_type_predicate(&n.iri, &triple.predicate.iri, type_map, rules)
            }
        },
        _ => false,
    };

    let mask = if pseudo_object {
        TripleMask::OBJECT
    } else {
        TripleMask::default()
    };

    return mask
}

/// Check if the type of input instance URI is in the rules.
fn match_type(subject: &str, rules: &Rules, type_map: &HashMap<String, String>) -> bool {
    match type_map.get(subject) {
        Some(v) => rules.subjects.of_type.contains(v),
        None => false,
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
    type_map: &HashMap<String, String>,
    rules: &Rules,
) -> bool {

    let subject_type = match type_map.get(subject) {
        None => return false,
        Some(v) => v
    };
    let preds = rules.objects.on_type_predicate.get(subject_type);
    if preds.is_none() || !preds.unwrap().contains(predicate) {
        return false
    }

    return true
}



#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
use crate::model::TripleMask;
    use crate::rdf_types::Triple;

    fn set_type_rule(t: &str) -> Rules {
        let mut rules = Rules::default();

        rules.subjects.of_type.insert(t.to_string());
        return rules;
    }

    fn set_predicate_rule(p: &str) -> Rules {
        let mut rules = Rules {
            invert: false,
            subjects: SubjectRules {
                of_type: HashSet::new(),
            },
            objects: ObjectRules {
                on_predicate: HashSet::new(),
                on_type_predicate: HashMap::new(),
            },
        };
        rules.objects.on_predicate.insert(p.to_string());
        return rules;
    }

    fn set_type_predicate_rule(s: &str, p: &str) -> Rules {
        let mut rules = Rules {
            invert: false,
            subjects: SubjectRules {
                of_type: HashSet::new(),
            },
            objects: ObjectRules {
                on_predicate: HashSet::new(),
                on_type_predicate: HashMap::new(),
            },
        };

        let mut set = HashSet::new();
        set.insert(p.to_string());

        rules
            .objects
            .on_type_predicate
            .insert(s.to_string(), set);

        return rules;
    }

    #[rstest]
    // Subject is in the rules & type index
    #[case("Alice", HashMap::from([("Alice", "Person")]), "Person", true)]
    // Subject is in the type index, not in the rules
    #[case("Alice", HashMap::from([("Alice", "Person")]), "Bank", false)]
    // Subject is not in the type index
    #[case("Alice", HashMap::from([("BankName", "Bank")]), "Bank", false)]
    fn type_rule(
        #[case] subject: &str,
        #[case] index: HashMap<&str, &str>,
        #[case] rule_type: &str,
        #[case] match_expected: bool,
    ) {
        // convert index key/values into Strings
        let type_index: HashMap<String, String> = index
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let rules = set_type_rule(rule_type);
        assert_eq!(match_type(subject, &rules, &type_index), match_expected);
    }

    #[rstest]
    // Predicate is in the rules
    #[case("hasName", "hasName", true)]
    // Predicate is not in the rules
    #[case("hasName", "hasAge", false)]
    fn predicate_rule(#[case] node_iri: &str, #[case] rule_type: &str, #[case] expected_o: bool) {
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
    #[case("Alice", "hasName", "Person", "hasName", "Alice", "Person", true)]
    // Subject in config, predicate not
    #[case("Alice", "hasName", "Person", "hasAge", "Alice", "Person", false)]
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

        let rules = set_type_predicate_rule(rule_subject, rule_predicate);

        let mut mask = TripleMask::default();
        let mut type_map = HashMap::new();
        type_map.insert(index_subject.to_string(), index_object.to_string());

        mask = match_subject_predicate_rule(&subject, &predicate, mask, &type_map, &rules);

        assert!(!mask.is_set(&TripleMask::SUBJECT));
        assert_eq!(mask.is_set(&TripleMask::OBJECT), expected_o);
    }
}
