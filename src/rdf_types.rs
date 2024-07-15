use super::model::Entity;
use std::{fmt, fmt::Write};

// Rewrite all the rio types to be able to instanciate triples
// Rename rio types as XXXView to distinguish them from our types
// Use rio types for parsing and serializing
// Define mappers between the two types
//
pub type NamedNodeView<'a> = rio_api::model::NamedNode<'a>;
pub type LiteralView<'a> = rio_api::model::Literal<'a>;
pub type TermView<'a> = rio_api::model::Term<'a>;
pub type TripleView<'a> = rio_api::model::Triple<'a>;
pub type BlankNodeView<'a> = rio_api::model::BlankNode<'a>;
pub type SubjectView<'a> = rio_api::model::Subject<'a>;

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct Triple {
    pub subject: Subject,
    pub predicate: NamedNode,
    pub object: Term,
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Subject {
    NamedNode(NamedNode),
    BlankNode(BlankNode),
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Hash)]
pub struct NamedNode {
    /// The [IRI](https://www.w3.org/TR/rdf11-concepts/#dfn-iri) itself.
    pub iri: String,
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Term {
    NamedNode(NamedNode),
    BlankNode(BlankNode),
    Literal(Literal),
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct BlankNode {
    /// The [blank node identifier](https://www.w3.org/TR/rdf11-concepts/#dfn-blank-node-identifier).
    pub id: String,
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Literal {
    /// A [simple literal](https://www.w3.org/TR/rdf11-concepts/#dfn-simple-literal) without datatype or language form.
    Simple {
        /// The [lexical form](https://www.w3.org/TR/rdf11-concepts/#dfn-lexical-form).
        value: String,
    },
    /// A [language-tagged string](https://www.w3.org/TR/rdf11-concepts/#dfn-language-tagged-string)
    LanguageTaggedString {
        /// The [lexical form](https://www.w3.org/TR/rdf11-concepts/#dfn-lexical-form).
        value: String,
        /// The [language tag](https://www.w3.org/TR/rdf11-concepts/#dfn-language-tag).
        language: String,
    },
    /// A literal with an explicit datatype
    Typed {
        /// The [lexical form](https://www.w3.org/TR/rdf11-concepts/#dfn-lexical-form).
        value: String,
        /// The [datatype IRI](https://www.w3.org/TR/rdf11-concepts/#dfn-datatype-iri).
        datatype: NamedNode,
    },
}

impl fmt::Display for NamedNode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.iri)
    }
}

impl fmt::Display for Literal {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Simple { value } => fmt_quoted_str(value, f),
            Literal::LanguageTaggedString { value, language } => {
                fmt_quoted_str(value, f)?;
                write!(f, "@{}", language)
            }
            Literal::Typed { value, datatype } => {
                fmt_quoted_str(value, f)?;
                write!(f, "^^{}", datatype)
            }
        }
    }
}

impl fmt::Display for Term {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::NamedNode(node) => node.fmt(f),
            Term::BlankNode(node) => node.fmt(f),
            Term::Literal(literal) => literal.fmt(f),
        }
    }
}

impl fmt::Display for Triple {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.subject, self.predicate, self.object)
    }
}

impl fmt::Display for Subject {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subject::NamedNode(node) => node.fmt(f),
            Subject::BlankNode(node) => node.fmt(f),
        }
    }
}

impl fmt::Display for BlankNode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_:{}", self.id)
    }
}

impl<'a> From<TripleView<'a>> for Triple {
    fn from(t: TripleView<'a>) -> Self {
        Triple {
            subject: t.subject.into(),
            predicate: t.predicate.into(),
            object: t.object.into(),
        }
    }
}

impl<'a> From<SubjectView<'a>> for Subject {
    #[inline]
    fn from(resource: SubjectView) -> Self {
        match resource {
            SubjectView::NamedNode(node) => Subject::NamedNode(node.into()),
            SubjectView::BlankNode(node) => Subject::BlankNode(node.into()),
            _ => panic!("Unexpected subject type"),
        }
    }
}

impl<'a> From<TermView<'a>> for Term {
    #[inline]
    fn from(term: TermView<'a>) -> Self {
        match term {
            TermView::NamedNode(node) => Term::NamedNode(node.into()),
            TermView::BlankNode(node) => Term::BlankNode(node.into()),
            TermView::Literal(literal) => Term::Literal(literal.into()),
            _ => panic!("Unexpected term type"),
        }
    }
}

impl<'a> From<NamedNodeView<'a>> for NamedNode {
    #[inline]
    fn from(node: NamedNodeView<'a>) -> Self {
        NamedNode {
            iri: node.iri.to_string(),
        }
    }
}

impl<'a> From<BlankNodeView<'a>> for BlankNode {
    #[inline]
    fn from(node: BlankNodeView<'a>) -> Self {
        BlankNode {
            id: node.id.to_string(),
        }
    }
}

impl<'a> From<LiteralView<'a>> for Literal {
    fn from(l: LiteralView<'a>) -> Self {
        match l {
            LiteralView::Simple { value } => Literal::Simple {
                value: value.to_string(),
            },
            LiteralView::LanguageTaggedString { value, language } => {
                Literal::LanguageTaggedString {
                    value: value.to_string(),
                    language: language.to_string(),
                }
            }
            LiteralView::Typed { value, datatype } => Literal::Typed {
                value: value.to_string(),
                datatype: NamedNode {
                    iri: datatype.iri.to_string(),
                },
            },
        }
    }
}

impl From<Subject> for Entity {
    fn from(subject: Subject) -> Entity {
        match subject {
            Subject::NamedNode(node) => Entity::NamedNode(node),
            Subject::BlankNode(node) => Entity::BlankNode(node),
        }
    }
}

impl From<Term> for Entity {
    fn from(term: Term) -> Entity {
        match term {
            Term::NamedNode(node) => Entity::NamedNode(node),
            Term::BlankNode(node) => Entity::BlankNode(node),
            Term::Literal(literal) => Entity::Literal(literal),
        }
    }
}

impl From<Entity> for Subject {
    fn from(entity: Entity) -> Subject {
        match entity {
            Entity::NamedNode(node) => Subject::NamedNode(node),
            Entity::BlankNode(node) => Subject::BlankNode(node),
            _ => panic!("Unexpected entity type"),
        }
    }
}

impl From<Entity> for Term {
    fn from(entity: Entity) -> Term {
        match entity {
            Entity::NamedNode(node) => Term::NamedNode(node),
            Entity::BlankNode(node) => Term::BlankNode(node),
            Entity::Literal(literal) => Term::Literal(literal),
        }
    }
}

#[inline]
fn fmt_quoted_str(string: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_char('"')?;
    for c in string.chars() {
        match c {
            '\n' => f.write_str("\\n"),
            '\r' => f.write_str("\\r"),
            '"' => f.write_str("\\\""),
            '\\' => f.write_str("\\\\"),
            c => f.write_char(c),
        }?;
    }
    f.write_char('"')
}
