//! Model for declarations

use super::{Loc, RpEnumBody, RpInterfaceBody, RpName, RpReg, RpServiceBody, RpTupleBody,
            RpTypeBody};
use std::fmt;
use std::rc::Rc;
use std::slice;

/// Iterator over declarations.
pub struct Decls<'a> {
    iter: slice::Iter<'a, Rc<Loc<RpDecl>>>,
}

impl<'a> Iterator for Decls<'a> {
    type Item = &'a Rc<Loc<RpDecl>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RpDecl {
    Type(Rc<Loc<RpTypeBody>>),
    Tuple(Rc<Loc<RpTupleBody>>),
    Interface(Rc<Loc<RpInterfaceBody>>),
    Enum(Rc<Loc<RpEnumBody>>),
    Service(Rc<Loc<RpServiceBody>>),
}

impl RpDecl {
    pub fn decls(&self) -> Decls {
        use self::RpDecl::*;

        let iter = match *self {
            Type(ref body) => body.decls.iter(),
            Interface(ref body) => body.decls.iter(),
            Enum(ref body) => body.decls.iter(),
            Tuple(ref body) => body.decls.iter(),
            Service(ref body) => body.decls.iter(),
        };

        Decls { iter: iter }
    }

    pub fn local_name(&self) -> &str {
        use self::RpDecl::*;

        match *self {
            Type(ref body) => body.local_name.as_str(),
            Interface(ref body) => body.local_name.as_str(),
            Enum(ref body) => body.local_name.as_str(),
            Tuple(ref body) => body.local_name.as_str(),
            Service(ref body) => body.local_name.as_str(),
        }
    }

    pub fn name(&self) -> &RpName {
        use self::RpDecl::*;

        match *self {
            Type(ref body) => &body.name,
            Interface(ref body) => &body.name,
            Enum(ref body) => &body.name,
            Tuple(ref body) => &body.name,
            Service(ref body) => &body.name,
        }
    }

    pub fn comment(&self) -> &[String] {
        use self::RpDecl::*;

        match *self {
            Type(ref body) => &body.comment,
            Interface(ref body) => &body.comment,
            Enum(ref body) => &body.comment,
            Tuple(ref body) => &body.comment,
            Service(ref body) => &body.comment,
        }
    }

    /// Convert a declaration into its registered types.
    pub fn into_reg(&self) -> Vec<RpReg> {
        use self::RpDecl::*;

        let mut out = Vec::new();

        match *self {
            Type(ref ty) => {
                out.push(RpReg::Type(ty.clone()));
            }
            Interface(ref interface) => {
                for sub_type in interface.sub_types.values() {
                    out.push(RpReg::SubType(interface.clone(), sub_type.clone()));
                    out.extend(sub_type.decls.iter().flat_map(|d| d.into_reg()));
                }

                out.push(RpReg::Interface(interface.clone()));
            }
            Enum(ref en) => {
                for variant in &en.variants {
                    out.push(RpReg::EnumVariant(en.clone(), variant.clone()));
                }

                out.push(RpReg::Enum(en.clone()));
            }
            Tuple(ref tuple) => {
                out.push(RpReg::Tuple(tuple.clone()));
            }
            Service(ref service) => {
                out.push(RpReg::Service(service.clone()));
            }
        }

        out.extend(self.decls().flat_map(|d| d.into_reg()));
        out
    }

    /// Get stringy kind of the declaration.
    pub fn kind(&self) -> &str {
        use self::RpDecl::*;

        match *self {
            Type(_) => "type",
            Interface(_) => "interface",
            Enum(_) => "enum",
            Tuple(_) => "tuple",
            Service(_) => "service",
        }
    }
}

impl fmt::Display for RpDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::RpDecl::*;

        match *self {
            Type(ref body) => write!(f, "type {}", body.name),
            Interface(ref body) => write!(f, "interface {}", body.name),
            Enum(ref body) => write!(f, "enum {}", body.name),
            Tuple(ref body) => write!(f, "tuple {}", body.name),
            Service(ref body) => write!(f, "service {}", body.name),
        }
    }
}
