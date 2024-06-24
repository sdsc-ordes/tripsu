use rio_turtle::NTriplesParser;
use crate::rules::Config;
use serde_yaml;

use std::{
    boxed::Box,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
    path::Path,
};

/// Get a reader based on input path, either from stdin or a file.
pub fn get_reader(path: &Path) -> Box<dyn BufRead> {
    return match path.to_str().unwrap() {
        "-" => Box::new(BufReader::new(stdin())),
        _ => Box::new(BufReader::new(File::open(&path).unwrap())),
    };
}

/// Get a writer based on input path, either to stdout or a file.
pub fn get_writer(path: &Path) -> Box<dyn Write> {
    return match path.to_str().unwrap() {
        "-" => Box::new(BufWriter::new(stdout())),
        path => Box::new(BufWriter::new(File::open(path).unwrap())),
    };
}

// Parse RDF triples.
// This function takes ownership of a generic type which implements `BufRead`.
pub fn parse_ntriples(reader: impl BufRead) -> NTriplesParser<impl BufRead> {
    return NTriplesParser::new(reader);
}

// Parse yaml configuration file.
pub fn parse_config(path: &Path) -> Config {
    return match File::open(&path) {
        Ok(file) => {
            serde_yaml::from_reader(file).unwrap()
        }
        Err(e) => panic!("Cannot open file '{:?}': '{}'.", path, e),
    };
}

#[cfg(test)]
mod tests {
use super::{parse_config, parse_ntriples};
    use rio_api::parser::TriplesParser;
    use std::{
        io::{BufRead, BufReader},
        path::Path,
    };

    #[test]
    // Test the parsing of a triple.
    fn triple_parsing() {
        let input: &[u8] = "<http://example.org/resource2> <http://example.org/relatedTo> <http://example.org/resource3> .\n".as_bytes();
        let buffer_input: Box<dyn BufRead> = Box::new(BufReader::new(input));
        let mut triples = parse_ntriples(buffer_input);
        triples
            .parse_all(&mut |t| -> Result<(), Box<dyn std::error::Error>> {
                assert_eq!(t.subject.to_string(), "<http://example.org/resource2>");
                assert_eq!(t.predicate.to_string(), "<http://example.org/relatedTo>");
                assert_eq!(t.object.to_string(), "<http://example.org/resource3>");
                Ok(())
            })
            .expect("Error parsing triple");
    }
    // Test the parsing of a config file.
    #[test]
    fn config_parsing() {
        let config_path = Path::new("tests/data/config.yaml");
        let config = parse_config(config_path);
        assert_eq!(config.replace_values_of_nodes_with_type[0], "http://xmlns.com/foaf/0.1/Person");
        assert_eq!(config.replace_values_of_nodes_with_type[1], "http://xmlns.com/foaf/OnlineAccount");
        assert_eq!(config.replace_values_of_subject_predicate[0].1, "http://schema.org/name");
        assert_eq!(config.replace_values_of_subject_predicate[1].0, "http://xmlns.com/foaf/0.1/Person");
        assert_eq!(config.replace_value_of_predicate[0], "http://schema.org/accessCode")
    }
}