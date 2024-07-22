use super::model::Entity;
use crate::{model::TripleMask, rdf_types::*};
use rand::Rng;

// generate a cryptographic key of predetermined length
pub(crate) fn generate_key(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    return (0..size).map(|_| rng.gen::<u8>()).collect();
}

pub trait Pseudonymize {
    // implementers need to define raw bytes pseudonymization
    fn pseudo(&self, input: &[u8]) -> String;

    // Pseudonymize parts of a triple set by its mask
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

    fn pseudo_entity(&self, e: &Entity) -> Entity {
        match e {
            Entity::Literal(l) => Entity::Literal(self.pseudo_literal(l)),
            Entity::NamedNode(n) => Entity::NamedNode(self.pseudo_named_node(n)),
            Entity::BlankNode(b) => Entity::BlankNode(self.pseudo_blank_node(b)),
        }
    }
    // private methods? Blanket implementations
    fn pseudo_named_node(&self, t: &NamedNode) -> NamedNode {
        // We check for the last fragment or path separator in the IRI
        let prefix_end = t.iri.rfind(|c: char| (c == '/') || (c == '#')).unwrap();
        let prefix = &t.iri[0..=prefix_end];
        let crypted = self.pseudo(t.iri.as_bytes()).to_string();
        return NamedNode {
            iri: format!("{prefix}{crypted}"),
        };
    }

    fn pseudo_literal(&self, l: &Literal) -> Literal {
        let value = match l {
            Literal::Typed { value, datatype: _ } => value,
            Literal::LanguageTaggedString { value, language: _ } => value,
            Literal::Simple { value } => value,
        };
        let crypted = self.pseudo(value.as_bytes());
        return Literal::Simple { value: crypted };
    }

    fn pseudo_blank_node(&self, u: &BlankNode) -> BlankNode {
        return u.clone();
    }
}

// Helper enums to instantiate various hashers

pub enum Algorithm {
    Blake3,
}

impl Default for Algorithm {
    fn default() -> Self {
        return Algorithm::Blake3;
    }
}

pub enum Hasher {
    Blake3(Blake3Hasher),
}

impl Hasher {
    pub fn new(algo: Algorithm, key: Option<Vec<u8>>) -> Self {
        let salt = key.unwrap_or(generate_key(32));
        match algo {
            Algorithm::Blake3 => Hasher::Blake3(Blake3Hasher::new(salt)),
        }
    }

    pub fn get_pseudonymizer(&self) -> &dyn Pseudonymize {
        match self {
            Hasher::Blake3(h) => h,
        }
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Hasher::new(Algorithm::Blake3, None)
    }
}

// BLAKE3 hasher

pub struct Blake3Hasher {
    pub key: [u8; 32],
}

impl Blake3Hasher {
    pub fn new(key: Vec<u8>) -> Self {
        let mut new_key = [0u8; 32];
        if key.len() == 32 {
            new_key.copy_from_slice(&key[..32]);
        } else {
            panic!("Key must be 32 bytes long");
        }

        return Self { key: new_key };
    }
}

impl Pseudonymize for Blake3Hasher {
    fn pseudo(&self, data: &[u8]) -> String {
        return blake3::keyed_hash(&self.key, &data).to_string();
    }
}
