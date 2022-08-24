use std::fmt;

pub type IRIdentifier = u64;

#[derive(Debug, Clone)]
pub struct IRError(pub String);

impl fmt::Display for IRError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
