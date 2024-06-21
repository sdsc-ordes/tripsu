use rio_api::{model::Triple, parser::TriplesParser};
use rio_turtle::TurtleError;
use std::{
    collections::HashMap,
    io::{BufRead, Write},
    path::Path,
};

use crate::{
    io,
    log::Logger,
    model::{pseudonymize_triple, TripleMask},
};

// mask and encode input triple
// NOTE: This will need the type-map to perform masking
fn process_triple(triple: &Triple) -> Result<(), TurtleError> {
    let mask = TripleMask::SUBJECT;
    println!("{}", pseudonymize_triple(&triple, mask).to_string());
    Ok(())
}

fn mask_triple(triple: &Triple) -> TripleMask {
    return TripleMask::SUBJECT;
}

// mask and encode input triple
// NOTE: This will need the type-map to perform masking
fn process_triple(triple: &Triple, out: &mut impl Write) -> Result<(), TurtleError> {
    let mask = mask_triple(triple);
    let pseudo_triple = pseudonymize_triple(&triple, mask);
    let _ = out.write(&format!("{} .\n", &pseudo_triple.to_string()).into_bytes());

    Ok(())
}

// Create a index mapping node -> type from an input ntriples buffer
fn load_type_map(input: impl BufRead) -> HashMap<String, String> {
    let mut node_to_type: HashMap<String, String> = HashMap::new();
    let mut triples = io::parse_ntriples(input);

    while !triples.is_end() {
        let _: Result<(), TurtleError> = triples.parse_step(&mut |t| {
            node_to_type.insert(t.subject.to_string(), t.object.to_string());
            Ok(())
        });
    }

    return node_to_type;
}

pub fn pseudonymize_graph(log: &Logger, input: &Path, output: &Path, index: &Path) {
    let buf_input = io::get_reader(input);
    let buf_index = io::get_reader(index);
    let mut buf_output = io::get_writer(output);
    let config = io::parse_config(config);
    
    let node_to_type: HashMap<String, String> = load_type_map(buf_index);
    let mut triples = io::parse_ntriples(buf_input);
    while !triples.is_end() {
        triples
            .parse_step(&mut |t| process_triple(&t, &mut buf_output))
            .unwrap();
    }
}
#[cfg(test)]
mod tests {
    use super::pseudonymize_graph;
    use crate::log;
    use std::path::Path;

    #[test]
    // Test the parsing of a triple.
    fn encrypt_nt_file() {
        let input_path = Path::new("tests/data/test.nt");
        let config_path = Path::new("tests/data/config.yaml");
        let output_path = Path::new("tests/data/output.nt");
        let type_map_path = Path::new("tests/data/type_map.nt");
        let logger = log::create_logger(true);
        pseudonymize_graph(&logger, &input_path, &output_path, &type_map_path);
    }
}
