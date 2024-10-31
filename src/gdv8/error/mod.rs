#[derive(Clone, Copy)]
pub enum Error {
    None,
    ScopePointerAllocationFailed,
    ContextAllocationFailed,
    InvalidEnvironment
}