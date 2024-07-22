use super::model::Entity;
use crate::{model::TripleMask, rdf_types::*};
use rand::Rng;

// generate a cryptographic key of predetermined length
pub(crate) fn generate_key(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    return (0..size).map(|_| rng.gen::<u8>()).collect();
}

pub trait Pseudonymize {
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
    fn pseudo_named_node(&self, t: &NamedNode) -> NamedNode;
    //return t.clone();

    fn pseudo_literal(&self, l: &Literal) -> Literal;
    //return l.clone();

    fn pseudo_blank_node(&self, u: &BlankNode) -> BlankNode;
    // return u.clone()
}

#[derive(Default)]
pub struct DefaultHasher {
    // Key used for hashing
    pub key: [u8; 32],
}
impl DefaultHasher {
    pub fn new(key: Vec<u8>) -> Self {
        // convert the key to a [u8, 32]
        let mut new_key = [0; 32];
        // make sure that 
        // 1) the key is not longer than 32 bytes
        // 2) the key is not shorter than 32 bytes
        // To do: make sure that the loop below works
        for i in 0..32 {
            new_key[i] = key[i];
        }
        return DefaultHasher { key: new_key };
    }
}

impl Pseudonymize for DefaultHasher {
    fn pseudo_named_node(&self, t: &NamedNode) -> NamedNode {
        // We check for the last backslash in the IRI and add 1 to include the backslash
        let prefix = &t.iri[0..t.iri.rfind('/').unwrap() + 1];
        let hash = blake3::keyed_hash(self.key, t.iri.as_bytes()).to_string();
        return NamedNode {
            iri: format!("{prefix}{hash}"),
        };
    }

    fn pseudo_literal(&self, l: &Literal) -> Literal {
        let value = match l {
            Literal::Typed { value, datatype: _ } => value,
            Literal::LanguageTaggedString { value, language: _ } => value,
            Literal::Simple { value } => value,
        };
        let hash = blake3::hash(value.as_bytes());
        return Literal::Simple {
            value: hash.to_string(),
        };
    }

    fn pseudo_blank_node(&self, u: &BlankNode) -> BlankNode {
        return u.clone();
    }
}
