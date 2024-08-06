use crate::rdf_types::*;
use ::std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::model::TripleMask;

/// Rules for pseudonymizing subjects
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SubjectRules {
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
    pub subjects: SubjectRules,

    #[serde(default)]
    pub objects: ObjectRules,
}

/// Check all parts of the triple against rules.
pub fn match_rules(
    triple: &Triple,
    rules: &Rules,
    type_map: &HashMap<String, String>,
) -> TripleMask {
    let mut mask =
        match_subject_rules(triple, rules, type_map) | match_object_rules(triple, rules, type_map);

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
        Subject::NamedNode(n) => match_type(&n.iri, rules, type_map),
        _ => false,
    };
    let pseudo_object = match &triple.object {
        Term::NamedNode(n) => match_type(&n.iri, rules, type_map),
        _ => false,
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
        }
        _ => false,
    };

    let mask = if pseudo_object {
        TripleMask::OBJECT
    } else {
        TripleMask::default()
    };

    return mask;
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
        Some(v) => v,
    };
    let preds = rules.objects.on_type_predicate.get(subject_type);
    if preds.is_none() || !preds.unwrap().contains(predicate) {
        return false;
    }

    return true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_yml;

    // Instance used in tests
    const SUBJECT_IRI: &str = "Alice";
    const PREDICATE_IRI: &str = "hasName";

    // Helper macro to create a HashMap from pairs
    #[macro_export]
    macro_rules! index {
    () => {
            ::std::collections::HashMap::new()
        };

        ($($key:expr => $value:expr),+ $(,)?) => {
            ::std::collections::HashMap::from([
                $((String::from($key), String::from($value))),*
            ])
        };
    }

    fn parse_rules(yml: &str) -> Rules {
        serde_yml::from_str(yml).unwrap()
    }

    #[rstest]
    // Subject is in the rules & type index
    #[case(index! { SUBJECT_IRI => "Person" }, "Person", true)]
    // Subject is in the type index, not in the rules
    #[case(index! { SUBJECT_IRI => "Person" }, "Bank", false)]
    // Subject is not in the type index
    #[case(index! { "BankName" => "Bank" }, "Bank", false)]
    fn type_rule(
        #[case] index: HashMap<String, String>,
        #[case] rule_type: &str,
        #[case] match_expected: bool,
    ) {
        let rules = parse_rules(&format!(
            "
            subjects:
              of_type:
              - {rule_type}
        "
        ));

        assert_eq!(match_type(SUBJECT_IRI, &rules, &index), match_expected);
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
    #[case("Person", "hasName", index! { SUBJECT_IRI => "Person" }, true)]
    // Subject in config, predicate not
    #[case("Person", "hasAge", index! { SUBJECT_IRI => "Person" }, false)]
    // Subject predicate not in config
    #[case("Bob", "hasAge", index! { SUBJECT_IRI => "Person" }, false)]
    // Subject not in type index
    #[case("Bob", "hasAge", index! { "Bob" => "Person" }, false)]
    fn type_predicate_rule(
        #[case] rule_type: &str,
        #[case] rule_predicate: &str,
        #[case] index: HashMap<String, String>,
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
            match_type_predicate(SUBJECT_IRI, PREDICATE_IRI, &index, &rules),
            match_expected
        );
    }
}
