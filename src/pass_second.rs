use rio_api::parser::TriplesParser;
use rio_turtle::TurtleError;
use std::{
    collections::HashMap,
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

use crate::{
    crypto::{Algorithm, Hasher, Pseudonymize},
    io,
    log::Logger,
    model::TripleMask,
    rdf_types::*,
    rules::{
        match_predicate_rule, match_subject_predicate_rule, match_type_rule_object,
        match_type_rule_subject, Rules,
    },
};

fn match_rules(triple: Triple, rules: &Rules, type_map: &HashMap<String, String>) -> TripleMask {
    // Check each field of the triple against the rules
    let mut mask = TripleMask::default();

    mask = match_type_rule_subject(&triple.subject, mask, type_map, rules);
    mask = match_type_rule_object(&triple.object, mask, type_map, rules);
    mask = match_predicate_rule(&triple.predicate, mask, rules);
    mask = match_subject_predicate_rule(&triple.subject, &triple.predicate, mask, type_map, rules);

    return mask;
}

// mask and encode input triple
// NOTE: This will need the type-map to perform masking
fn process_triple(
    triple: Triple,
    rules_config: &Rules,
    node_to_type: &HashMap<String, String>,
    out: &mut impl Write,
    hasher: &dyn Pseudonymize,
) {
    let mask = match_rules(triple.clone(), rules_config, node_to_type);

    let r = || -> std::io::Result<()> {
        out.write_all(hasher.pseudo_triple(&triple, mask).to_string().as_bytes())?;
        out.write_all(b" .\n")
    }();

    if let Err(e) = r {
        panic!("Error writting to out buffer: {e}");
    }
}

// Create a index mapping node -> type from an input ntriples buffer
fn load_type_map(input: impl BufRead) -> HashMap<String, String> {
    let mut node_to_type: HashMap<String, String> = HashMap::new();
    let mut triples = io::parse_ntriples(input);

    while !triples.is_end() {
        let _: Result<(), TurtleError> = triples.parse_step(&mut |t| {
            node_to_type.insert(
                t.subject.to_string().replace(['<', '>'], ""),
                t.object.to_string().replace(['<', '>'], ""),
            );
            Ok(())
        });
    }

    return node_to_type;
}

pub fn pseudonymize_graph(_: &Logger, input: &Path, config: &Path, output: &Path, index: &Path, secret: &Option<PathBuf>) {
    let buf_input = io::get_reader(input);
    let buf_index = io::get_reader(index);
    let mut buf_output = io::get_writer(output);

    let rules_config = io::parse_config(config);
    let node_to_type: HashMap<String, String> = load_type_map(buf_index);

    let hasher = if let Some(secret) = secret {
        let key = io::get_key(secret);
        Hasher::new(Algorithm::Blake3, Some(key))
    } else {
        Hasher::default()
    };
    let pseudonymizer = hasher.get_pseudonymizer();

    let mut triples = io::parse_ntriples(buf_input);

    // Run the loop single-threaded.
    while !triples.is_end() {
        triples
            .parse_step(&mut |t: TripleView| {
                process_triple(t.into(), &rules_config, &node_to_type, &mut buf_output, pseudonymizer);
                Result::<(), TurtleError>::Ok(())
            })
            .inspect_err(|e| {
                panic!("Parsing error occured: {e}");
            })
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::pseudonymize_graph;
    use crate::log;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    // Test the parsing of a triple.
    fn encrypt_nt_file() {
        let logger = log::create_logger(true);

        let dir = tempdir().unwrap();
        let input_path = Path::new("tests/data/test.nt");
        let config_path = Path::new("tests/data/config.yaml");
        let output_path = dir.path().join("output.nt");
        let type_map_path = Path::new("tests/data/type_map.nt");
        let key = None;

        pseudonymize_graph(
            &logger,
            &input_path,
            &config_path,
            &output_path,
            &type_map_path,
            &key,
        );
    }
}
