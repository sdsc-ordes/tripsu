use rio_turtle::NTriplesParser;
use std::{
    boxed::Box,
    fs::File,
    io::{stdout, BufRead, BufReader, Write, BufWriter},
    path::Path,
};

pub fn get_buffer(path: &Path) -> BufReader<File> {
    return match File::open(&path) {
        Ok(file) => BufReader::new(file),
        Err(e) => panic!("Cannot open file '{path:?}': '{e}'."),
    };
}

pub fn get_writer(path: &Path) -> Box<dyn Write> {
    return match path.to_str().unwrap() {
        "" => Box::new(BufWriter::new(stdout())),
        path => Box::new(BufWriter::new(File::open(path).unwrap())),
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
