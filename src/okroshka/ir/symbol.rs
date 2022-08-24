#[derive(Debug)]
pub enum IRSymbol {
    Global(String),
    ThreadLocal(String)
}

impl IRSymbol {
    pub fn name(&self) -> &str {
        match self {
            IRSymbol::Global(s) => &s,
            IRSymbol::ThreadLocal(s) => &s
        }
    }
}
