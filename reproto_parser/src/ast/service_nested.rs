use super::*;

#[derive(Debug)]
pub enum ServiceNested<'a> {
    Endpoint {
        url: AstLoc<String>,
        comment: Vec<&'a str>,
        options: Vec<AstLoc<OptionDecl>>,
        children: Vec<ServiceNested<'a>>,
    },
    Star {
        comment: Vec<&'a str>,
        options: Vec<AstLoc<OptionDecl>>,
        children: Vec<ServiceNested<'a>>,
    },
    Returns {
        comment: Vec<&'a str>,
        ty: AstLoc<RpType>,
        options: Vec<AstLoc<OptionDecl>>,
    },
}

impl<'a> ServiceNested<'a> {
    pub fn is_returns(&self) -> bool {
        match *self {
            ServiceNested::Returns { .. } => true,
            _ => false,
        }
    }
}
