use std::hash::Hash;

use crate::rdf_types::*;
use bitflags;

pub enum Entity {
    Literal(Literal),
    Subject(Subject),
    NamedNode(NamedNode),
    Triple(Triple),
    BlankNode(BlankNode),
}

pub trait Pseudonymize {
    fn pseudo(&self, e: Entity) -> Entity {
        match e {
            Literal => self.pseudo_literal(e),
            Subject => self.pseudo_uri(e),
            NamedNode => self.pseudo_uri(e),
            Triple => self.pseudo_triple(e),
            BlankNode => e,
        }
    }
    // private methods? Blanket implementations
    fn pseudo_triple(&self, t: Entity) -> Entity {
        return t;
    }
    fn pseudo_literal(&self, l: Entity) -> Entity {
        return l;
    }
    fn pseudo_uri(&self, u: Entity) -> Entity {
        return u;
    }
}

// Used to select any combination of fields in a triple
bitflags::bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct TripleMask: u8 {
        const SUBJECT = 1 << 2 ;
        const PREDICATE = 1 << 1;
        const OBJECT = 1 << 0;
    }
}

impl TripleMask {
    // Checks if bit from another mask are all set in this mask
    pub fn is_set(&self, other: &TripleMask) -> bool {
        return (*other - *self).bits() != 0;
    }
}
