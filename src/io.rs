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

#[cfg(test)]
mod tests {
    use super::parse_ntriples;
    use std::io::{BufRead, BufReader};
    use rio_api::parser::TriplesParser;

    #[test]
    // Test the parsing of a triple.
    fn triple_parsing() {
        let input: &[u8] = "<http://example.org/resource2> <http://example.org/relatedTo> <http://example.org/resource3> .\n".as_bytes();
        let buffer_input: Box<dyn BufRead> = Box::new(BufReader::new(input));
        let mut triples = parse_ntriples(buffer_input);
        triples.parse_all(&mut |t| -> Result<(), Box<dyn std::error::Error>> {
            assert_eq!(t.subject.to_string(), "<http://example.org/resource2>" );
            assert_eq!(t.predicate.to_string(), "<http://example.org/relatedTo>" );
            assert_eq!(t.object.to_string(), "<http://example.org/resource3>" );
            Ok(())
        }).expect("Error parsing triple");
        }
    }

