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
    #[test]
    // Test the parsing of a triple.
    fn parse_ntriples() {
        let input = "\n
                <http://example.org/resource2> <http://example.org/relatedTo> <http://example.org/resource3>\n
                <http://example.org/resource2> <http://example.org/relatedTo> <http://example.org/resource3>\n
            ";
    }
}
