use serde::{Deserialize, Serialize};
use::std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    // Replace values of nodes with a certain type.
    pub replace_uri_of_nodes_with_type: Vec<String>,

    // Replace values of `subject` & `predicate`.
    pub replace_values_of_subject_predicate: HashMap<String, Vec<String>>,

    // Replace values in matched `predicates`.
    pub replace_value_of_predicate: Vec<String>,
}
