use crate::model::Triple;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    iter::Iterator,
    path::Path,
};

pub fn get_buffer(path: &Path) -> BufReader<File> {
    return match File::open(&path) {
        Ok(file) => BufReader::new(file),
        Err(e) => panic!("Cannot open file '{:?}': '{}'.", path, e),
    };
}

// Parse RDF triples.
pub fn parse_ntriples(reader: Box<dyn BufRead>) -> impl Iterator<Item = Triple> {
    return reader.lines().map(|l| Triple::parse_ntriples(&l.unwrap()));
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
