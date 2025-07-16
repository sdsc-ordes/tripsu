use crate::{index::TypeIndex, rules::Rules};
use rio_api::{formatter::TriplesFormatter, parser::TriplesParser};
use rio_turtle::{NTriplesFormatter, NTriplesParser, TurtleParser};
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

/// Get a reader based on input path, either from stdin or a file.
pub fn get_reader(path: &Path) -> Reader {
    return match path.to_str().unwrap() {
        "-" => Reader::Stdio(BufReader::new(stdin())),
        path => Reader::File(BufReader::new(File::open(path).unwrap())),
    };
}

/// Get a writer based on input path, either to stdout or a file.
pub fn get_writer(path: &Path) -> Writer {
    return match path.to_str().unwrap() {
        "-" => Writer::Stdio(BufWriter::new(stdout())),
        path => Writer::File(BufWriter::new(File::create(path).unwrap())),
    };
}

/// Parse RDF triples.
/// This function takes ownership of a generic type which implements `BufRead`.
pub fn parse_ntriples<R: BufRead + 'static>(
    reader: R,
) -> Result<NTriplesParser<Box<dyn BufRead>>, anyhow::Error> {
    let reader: Box<dyn BufRead> = Box::new(reader);
    Ok(NTriplesParser::new(reader))
}

/// Parse turtle file and convert to Ntriples stream
pub fn parse_turtle<R: BufRead>(
    reader: R,
) -> Result<NTriplesParser<Box<dyn BufRead>>, anyhow::Error> {
    let mut formatter = NTriplesFormatter::new(Vec::default());
    let mut turtle_parser = TurtleParser::new(reader, None);

    turtle_parser
        .parse_all(&mut |triple| formatter.format(&triple))
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let formatted_data = formatter
        .finish()
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let reader: Box<dyn BufRead> = Box::new(BufReader::new(std::io::Cursor::new(formatted_data)));
    Ok(NTriplesParser::new(reader))
}

/// Detect input format and parse accordingly.
pub fn parse_input<R: BufRead + 'static>(
    input: &Path,
    buf_input: R,
) -> Result<NTriplesParser<Box<dyn BufRead>>, anyhow::Error> {
    match input.extension() {
        Some(ext) if ext == std::ffi::OsStr::new("nt") => parse_ntriples(buf_input),
        Some(ext) if ext == std::ffi::OsStr::new("ttl") => parse_turtle(buf_input),
        Some(_) => Err(anyhow::Error::msg(
            "Extension not supported as input. Please use '.nt' or '.ttl'",
        )),
        None => Err(anyhow::Error::msg(
            "No input format detected. Please use '.nt' or '.ttl'",
        )),
    }
}

/// Parse yaml configuration file.
pub fn parse_rules(path: &Path) -> Rules {
    let rules: Rules = match File::open(path) {
        Ok(file) => serde_yml::from_reader(file).expect("Error parsing rules file."),
        Err(e) => panic!("Cannot open rules file '{:?}': '{}'.", path, e),
    };
    match rules.expand_rules_curie() {
        Ok(expanded_rules) => expanded_rules,
        Err(e) => panic!("Error expanding rules curie. {}", e),
    }
}

/// Parse yaml type index
pub fn parse_index(path: &Path) -> TypeIndex {
    match File::open(path) {
        Ok(file) => serde_json::from_reader(file).expect("Error parsing index file."),
        Err(e) => panic!("Cannot open index file '{:?}': '{}'.", path, e),
    }
}

/// Read all file content as bytes.
pub fn read_bytes(path: &PathBuf) -> Vec<u8> {
    let mut file = File::open(path).expect("Error opening key file.");
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .expect("Error reading key file.");

    data
}

#[cfg(test)]
mod tests {
    use crate::io::parse_input;

    use super::parse_rules;
    use rio_api::parser::TriplesParser;
    use std::{
        io::{BufRead, BufReader},
        path::Path,
    };

    use rstest::rstest;

    // Test the parsing of an input in a given format.
    #[rstest]
    // Ntriples input
    #[case("test.nt", "ntriple", true)]
    // Turtle input
    #[case("test.ttl", "turtle", true)]
    // Input with no extention
    #[case("test", "turtle", false)]
    fn triple_parsing(
        #[case] input_filename: &str,
        #[case] mock_input_type: &str,
        #[case] correct_input: bool,
    ) {
        let input_filename_string = input_filename.to_string();
        let input_path = Path::new(&input_filename_string);
        let mock_input: &[u8] = match mock_input_type {
            "ntriple" => "<http://example.org/resource2> <http://example.org/relatedTo> <http://example.org/resource3> .\n".as_bytes(),
            "turtle" => "@prefix : <http://example.org/> .\n\n :resource2 :relatedTo :resource3 .\n".as_bytes(),
            _ => "".as_bytes()
        };
        let buffer_input: Box<dyn BufRead> = Box::new(BufReader::new(mock_input));
        if correct_input {
            let mut triples = parse_input(input_path, buffer_input).expect("Error parsing input");
            triples
                .parse_all(&mut |t| -> Result<(), Box<dyn std::error::Error>> {
                    assert_eq!(t.subject.to_string(), "<http://example.org/resource2>");
                    assert_eq!(t.predicate.to_string(), "<http://example.org/relatedTo>");
                    assert_eq!(t.object.to_string(), "<http://example.org/resource3>");
                    Ok(())
                })
                .expect("Error parsing triple");
        } else {
            assert!(parse_input(input_path, buffer_input).is_err())
        }
    }
    // Test the parsing of a config file.
    #[test]
    fn rules_parsing() {
        let config_path = Path::new("tests/data/rules.yaml");
        parse_rules(config_path);
    }
}
