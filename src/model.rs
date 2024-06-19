use rio_api::model::{Subject, Term, Triple};

pub trait Pseudonymize {
    fn pseudo(&self) -> Self;
}

// Represent an individual component of a triple.
#[repr(u8)]
pub enum TriplePart {
    SUBJECT = 0b100,
    PREDICATE = 0b010,
    OBJECT = 0b001,
}

// Used to select any combination of fields in a triple
pub struct TripleMask(u8);

impl TripleMask {
    pub fn new() -> Self {
        return TripleMask(0);
    }

    pub fn union(&mut self, other: TripleMask) -> TripleMask {
        return TripleMask(self.0 | other.0);
    }

    pub fn is_set(&self, part: TriplePart) -> bool {
        return (self.0 & part as u8) != 0;
    }

    pub fn bits(&self) -> u8 {
        return self.0;
    }

    pub fn set(&mut self, part: TriplePart) {
        self.0 |= part as u8;
    }
}

// Pseudonymize parts of a triple set by its mask
pub fn pseudonymize_triple<'a>(triple: &Triple<'a>, mask: TripleMask) -> Triple<'a> {
    let pseudo_subject = if mask.is_set(TriplePart::SUBJECT) {
        &triple.subject.pseudo()
    } else {
        &triple.subject.clone()
    };

    let pseudo_object = if mask.is_set(TriplePart::OBJECT) {
        triple.object.pseudo()
    } else {
        triple.object.clone()
    };

    return Triple {
        subject: *pseudo_subject,
        predicate: triple.predicate,
        object: pseudo_object,
    };
}

// Pseudonymization of objects (Nodes or literals)
impl Pseudonymize for Term<'_> {
    fn pseudo(&self) -> Self {
        match self {
            Term::Literal(val) => Term::Literal(*val),
            Term::NamedNode(val) => Term::NamedNode(*val),
            Term::BlankNode(val) => Term::BlankNode(*val),
            Term::Triple(_) => panic!("RDF-star not supported (triple as object)"),
        }
    }
}

// Pseudonymization of subjects (always a URI / blank node)
impl Pseudonymize for Subject<'_> {
    fn pseudo(&self) -> Self {
        match self {
            Subject::NamedNode(val) => Subject::NamedNode(*val),
            Subject::BlankNode(val) => Subject::BlankNode(*val),
            Subject::Triple(_) => panic!("RDF-star not supported (triple as subject)"),
        }
    }
}

// TODO: implement for blanknodes
// NOTE: Support for RDF-star?
