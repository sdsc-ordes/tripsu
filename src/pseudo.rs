use rio_api::parser::TriplesParser;
use rio_turtle::TurtleError;
use std::{
    collections::HashMap,
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

use crate::{
    crypto::{new_pseudonymizer, Pseudonymize},
    io,
    log::Logger,
    rdf_types::*,
    rules::{match_rules, Rules},
};

// mask and encode input triple
// NOTE: This will need the type-map to perform masking
fn process_triple(
    triple: Triple,
    rules_config: &Rules,
    node_to_type: &HashMap<String, String>,
    out: &mut impl Write,
    hasher: &dyn Pseudonymize,
) {
    let mask = match_rules(&triple, rules_config, node_to_type);

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

pub fn pseudonymize_graph(
    _: &Logger,
    input: &Path,
    rules_path: &Path,
    output: &Path,
    index_path: &Path,
    secret_path: &Option<PathBuf>,
) {
    let buf_input = io::get_reader(input);
    let buf_index = io::get_reader(index_path);
    let mut buf_output = io::get_writer(output);

    let rules = io::parse_rules(rules_path);
    let node_to_type: HashMap<String, String> = load_type_map(buf_index);

    let secret = secret_path.as_ref().map(io::read_bytes);
    let pseudonymizer = new_pseudonymizer(None, secret);

    let mut triples = io::parse_ntriples(buf_input);

    // Run the loop single-threaded.
    while !triples.is_end() {
        triples
            .parse_step(&mut |t: TripleView| {
                process_triple(
                    t.into(),
                    &rules,
                    &node_to_type,
                    &mut buf_output,
                    &pseudonymizer,
                );
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
        let rules_path = Path::new("tests/data/rules.yaml");
        let output_path = dir.path().join("output.nt");
        let type_map_path = Path::new("tests/data/type_map.nt");
        let key = None;
        pseudonymize_graph(
            &logger,
            &input_path,
            &rules_path,
            &output_path,
            &type_map_path,
            &key,
        );
    }
}
