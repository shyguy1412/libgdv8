#[derive(Clone, Debug)]
pub enum Error {
    Exception(String),
    None,
    ScopePointerAllocationFailed,
    ContextAllocationFailed,
    UnitializedRuntime,
    InvalidContext,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::Exception(v) => &v,
                Error::None => "None",
                Error::ScopePointerAllocationFailed => "ScopePointerAllocationFailed",
                Error::ContextAllocationFailed => "ContextAllocationFailed",
                Error::UnitializedRuntime => "UnitializedRuntime",
                Error::InvalidContext => "InvalidContext",
            }
        )
    }
}
