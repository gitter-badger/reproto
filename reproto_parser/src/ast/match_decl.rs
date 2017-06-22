use super::*;

#[derive(Debug)]
pub struct MatchDecl<'input> {
    pub members: Vec<AstLoc<'input, MatchMember<'input>>>,
}
