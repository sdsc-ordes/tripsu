use super::model::Entity;
use crate::{model::TripleMask, rdf_types::*};
use blake3;

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
            Entity::Literal(l) => Entity::Literal(self.pseudo_literal(&l)),
            Entity::NamedNode(n) => Entity::NamedNode(self.pseudo_named_node(&n)),
            Entity::BlankNode(b) => Entity::BlankNode(self.pseudo_blank_node(&b)),
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

pub struct DefaultHasher {
    hasher: blake3::Hasher,
}

impl DefaultHasher {
    pub fn new() -> Self {
        return DefaultHasher {
            hasher: blake3::Hasher::new(),
        };
    }
}

impl Pseudonymize for DefaultHasher {
    fn pseudo_named_node(&self, t: &NamedNode) -> NamedNode {
        return t.clone();
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
