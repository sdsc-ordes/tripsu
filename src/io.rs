use crate::rules::Config;

use serde_yml;

use rio_turtle::NTriplesParser;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

pub fn get_buffer(path: &Path) -> BufReader<File> {
    return match File::open(&path) {
        Ok(file) => BufReader::new(file),
        Err(e) => panic!("Cannot open file '{path:?}': '{e}'."),
    };
}

// Parse RDF triples.
// This function takes ownership of a generic type which implements `BufRead`.
pub fn parse_ntriples(reader: impl BufRead) -> NTriplesParser<impl BufRead> {
    return NTriplesParser::new(reader);
}

// Parse yaml configuration file.
pub fn parse_config(path: &Path) -> () {
    return match File::open(&path) {
        Ok(file) => {
            let res: Config = serde_yml::from_reader(file).unwrap();
            println!("{:?}", res);
        }
        Err(e) => panic!("Cannot open file '{:?}': '{}'.", path, e),
    };
}

#[cfg(test)]
mod tests {
    use super::{parse_config, parse_ntriples};
    use std::{
        io::{BufRead, BufReader},
        path::Path,
    };

    #[test]
    // Test the parsing of a triple.
    fn simple_parsing() {
        let input: &[u8] = "<http://example.org/resource2> <http://example.org/relatedTo> <http://example.org/resource3> .\n".as_bytes();
        let buffer_input: Box<dyn BufRead> = Box::new(BufReader::new(input));
        parse_ntriples(buffer_input).for_each(|t| {
            assert_eq!(t.subject, "A"); // to replace with http://example.org/resource2
            assert_eq!(t.predicate, "B"); // to replace with http://example.org/relatedTo
            assert_eq!(t.object, "C"); // to replace with http://example.org/resource3
        });

        #[test]
        // Test the parsing of a config file.
        fn config_parsing() {
            let config_path = Path::new("tests/data/config.yaml");
            parse_config(config_path);
        }
    }
}
