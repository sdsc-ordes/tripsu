use std::hash::Hash;

use crate::rdf_types::*;
use bitflags;

pub enum Entity {
    Literal(Literal),
    NamedNode(NamedNode),
    BlankNode(BlankNode),
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
