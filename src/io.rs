use crate::{index::TypeIndex, rules::Rules};
use rio_turtle::NTriplesParser;
use std::{
    fs::File,
    io::{self, stdin, stdout, BufRead, BufReader, BufWriter, Read},
    path::{Path, PathBuf},
};

use io_enum::{BufRead, Read, Write};

#[derive(Read, BufRead)]
pub enum Reader {
    Stdio(BufReader<io::Stdin>),
    File(BufReader<File>),
}

#[derive(Write)]
pub enum Writer {
    Stdio(BufWriter<io::Stdout>),
    File(BufWriter<File>),
}

// Get a reader based on input path, either from stdin or a file.
pub fn get_reader(path: &Path) -> Reader {
    return match path.to_str().unwrap() {
        "-" => Reader::Stdio(BufReader::new(stdin())),
        path => Reader::File(BufReader::new(File::open(path).unwrap())),
    };
}

// Get a writer based on input path, either to stdout or a file.
pub fn get_writer(path: &Path) -> Writer {
    return match path.to_str().unwrap() {
        "-" => Writer::Stdio(BufWriter::new(stdout())),
        path => Writer::File(BufWriter::new(File::create(path).unwrap())),
    };
}

// Parse RDF triples.
// This function takes ownership of a generic type which implements `BufRead`.
pub fn parse_ntriples(reader: impl BufRead) -> NTriplesParser<impl BufRead> {
    return NTriplesParser::new(reader);
}

// Parse yaml configuration file.
pub fn parse_rules(path: &Path) -> Rules {
    let rules: Rules = match File::open(path) {
        Ok(file) => serde_yml::from_reader(file).expect("Error parsing rules file."),
        Err(e) => panic!("Cannot open rules file '{:?}': '{}'.", path, e),
    };
    if rules.is_empty() {
        panic!("Rules file is empty.");
    } else if !rules.has_valid_curies() {
        panic!("Rules file has invalid URIs.");
    } else {
        return rules.expand_curie();
    }
}

// Parse yaml type index
pub fn parse_index(path: &Path) -> TypeIndex {
    return match File::open(path) {
        Ok(file) => serde_json::from_reader(file).expect("Error parsing index file."),
        Err(e) => panic!("Cannot open index file '{:?}': '{}'.", path, e),
    };
}

// Read all file content as bytes.
pub fn read_bytes(path: &PathBuf) -> Vec<u8> {
    let mut file = File::open(path).expect("Error opening key file.");
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .expect("Error reading key file.");

    return data;
}

#[cfg(test)]
mod tests {
    use super::{parse_ntriples, parse_rules};
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
    fn rules_parsing() {
        let config_path = Path::new("tests/data/rules.yaml");
        parse_rules(&config_path);
    }
}
