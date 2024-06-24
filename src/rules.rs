use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    // Replace values of nodes with a certain type.
    pub replace_values_of_nodes_with_type: Vec<String>,

    // Replace values of `subject` & `predicate`.
    pub replace_values_of_subject_predicate: Vec<(String, String)>,

    // Replace values in matched `predicates`.
    pub replace_value_of_predicate: Vec<String>,
}
