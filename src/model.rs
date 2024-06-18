use crate::crypto::hash;
use bitflags::bitflags;
use rio_api::model::{Literal, NamedNode, Term, Triple};


bitflags! {
    pub struct TripleMask: u8 {
        const SUBJECT = 1 << 0;
        const OBJECT = 1 << 1;
    }
}

impl TripleMask {
    // Checks if a all bits in `mask` are set.
    fn is_set(&self, mask: TripleMask) -> bool {
        return self.bits() & mask.bits() == mask.bits();
    }
}

pub trait HashTriple {
    fn hash_parts(&self, mask: TripleMask) -> Triple;
}

pub trait HashTerm {
    fn hash_term(&self) -> Term;
}


impl HashTriple for Triple<'_> {

    fn hash_parts(&self, mask: TripleMask) -> Triple {
        let hash_subject = if mask.is_set(TripleMask::SUBJECT) {
            hash(&self.subject)
        } else {
            self.subject.clone()
        };

        let hash_object = if mask.is_set(TripleMask::OBJECT) {
            hash(&self.object)
        } else {
            self.object.clone()
        };
        return Triple {
            subject: NamedNode { iri: "http://example.com/foo" }.into(),
            predicate: self.predicate,
            object: NamedNode { iri: "http://example.com/foo" }.into(),
        }
        return Triple::new(hash_subject, hash_predicate, hash_object);
    }

}

impl HashTerm for Literal<'_> {
    fn hash_term(&self) -> Term {
        return Term::Literal(Literal::new_hashed(self.value()));
    }
}

impl HashTerm for NamedNode<'_> {
    fn hash_term(&self) -> Term {
        return Term::NamedNode(NamedNode::new_hashed(self.iri()));
    }
}

// TODO: implement for blanknodes
// NOTE: Support for RDF-star?

