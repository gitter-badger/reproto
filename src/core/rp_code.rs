use super::*;
use super::errors::*;
use super::merge::Merge;

#[derive(Debug, Clone, Serialize)]
pub struct RpCode {
    pub context: String,
    pub lines: Vec<String>,
}

impl Merge for Vec<RpLoc<RpCode>> {
    fn merge(&mut self, source: Vec<RpLoc<RpCode>>) -> Result<()> {
        self.extend(source);
        Ok(())
    }
}
