use super::model::Entity;
use crate::{model::TripleMask, rdf_types::*};
use rand::Rng;

/// Generate a cryptographic key of predetermined length
pub(crate) fn generate_key(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    return (0..size).map(|_| rng.gen::<u8>()).collect();
}

/// Provides a generic interface for pseudonymization of RDF data
/// Implementers only define raw bytes pseudonymization, while
/// higher-level methods are provided.
pub trait Pseudonymize {
    /// Pseudonymize a byte array
    fn pseudo(&self, input: &[u8]) -> String;

    /// Pseudonymize parts of a triple set by its mask
    fn pseudo_triple(&self, triple: &Triple, mask: TripleMask) -> Triple {
        let pseudo_subject = if mask.is_set(&TripleMask::SUBJECT) {
            &self.pseudo_entity(&triple.subject.clone().into())
        } else {
            &triple.subject.clone().into()
        };

        let pseudo_object = if mask.is_set(&TripleMask::OBJECT) {
            &self.pseudo_entity(&triple.object.clone().into())
        } else {
            &triple.object.clone().into()
        };

        return Triple {
            subject: Subject::from(pseudo_subject.clone()),
            predicate: triple.predicate.clone(),
            object: Term::from(pseudo_object.clone()),
        };
    }

    /// Pseudonymize an entity (component of a triple) based on its type.
    fn pseudo_entity(&self, e: &Entity) -> Entity {
        match e {
            Entity::Literal(l) => Entity::Literal(self.pseudo_literal(l)),
            Entity::NamedNode(n) => Entity::NamedNode(self.pseudo_named_node(n)),
            Entity::BlankNode(b) => Entity::BlankNode(self.pseudo_blank_node(b)),
        }
    }

    /// Pseudonymize a named node, preserving its prefix.
    fn pseudo_named_node(&self, t: &NamedNode) -> NamedNode {
        // We check for the last fragment or path separator in the IRI
        let prefix_end = t.iri.rfind(['#', '/']).unwrap();
        let prefix = &t.iri[0..=prefix_end];
        let crypted = self.pseudo(t.iri.as_bytes()).to_string();
        return NamedNode {
            iri: format!("{prefix}{crypted}"),
        };
    }

    /// Pseudonymize a literal resulting in a simple literal (hash string).
    fn pseudo_literal(&self, l: &Literal) -> Literal {
        let value = match l {
            Literal::Typed { value, datatype: _ } => value,
            Literal::LanguageTaggedString { value, language: _ } => value,
            Literal::Simple { value } => value,
        };
        let crypted = self.pseudo(value.as_bytes());
        return Literal::Simple { value: crypted };
    }

    /// Leave blank nodes unchanged.
    fn pseudo_blank_node(&self, u: &BlankNode) -> BlankNode {
        return u.clone();
    }
}

/// Available pseudonymization algorithms.
pub enum Algorithm {
    Blake3,
}

impl Default for Algorithm {
    fn default() -> Self {
        return Algorithm::Blake3;
    }
}

/// Factory method for creating a pseudonymizer
/// based on the selected algorithm and secret key.
pub fn new_pseudonymizer(algo: Option<Algorithm>, secret: Option<Vec<u8>>) -> impl Pseudonymize {
    let pseudonymizer = match algo.unwrap_or_default() {
        Algorithm::Blake3 => Blake3Hasher::new(secret),
    };

    return pseudonymizer;
}

/// BLAKE3-based pseudonymizer.
pub struct Blake3Hasher {
    pub key: [u8; 32],
}

impl Blake3Hasher {
    pub fn new(secret: Option<Vec<u8>>) -> Self {
        secret.as_ref().inspect(|s| {
            if s.len() < 32 {
                panic!("Secret must be at least 32 bytes long");
            }
        });

        // blake3 key must be exactly 32 bytes long
        let mut key = [0u8; 32];
        let key_vec = match secret {
            Some(s) => blake3::hash(&s).as_bytes()[..32].to_vec(),
            None => generate_key(32),
        };
        key.copy_from_slice(&key_vec[..32]);

        return Self { key };
    }
}

impl Pseudonymize for Blake3Hasher {
    fn pseudo(&self, data: &[u8]) -> String {
        return blake3::keyed_hash(&self.key, data).to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid_hex(input: &str) -> bool {
        input.chars().all(|c| c.is_digit(16))
    }

    // Test the generation of a cryptographic key
    #[test]
    fn test_generate_key() {
        let key = generate_key(42);
        assert_eq!(key.len(), 42);
    }

    #[test]
    fn test_pseudo_named_node() {
        let hasher = Blake3Hasher::new(None);
        let named_node = NamedNode {
            iri: "http://example.com/abc".to_string(),
        };
        let pseudo = hasher.pseudo_named_node(&named_node).iri;
        // test that output is prefix + hash
        assert_eq!(pseudo.starts_with("http://example.com/"), true);
        assert!(is_valid_hex(
            pseudo.strip_prefix("http://example.com/").unwrap()
        ));
    }

    #[test]
    fn test_pseudo_literal() {
        let hasher = Blake3Hasher::new(None);
        let literal = Literal::Simple {
            value: "example".to_string(),
        };
        let pseudo_literal = hasher.pseudo_literal(&literal);
        // test that output is quoted hash
        assert!(is_valid_hex(&pseudo_literal.to_string().trim_matches('"')));
    }
}
