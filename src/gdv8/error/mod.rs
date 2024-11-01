#[derive(Clone, Copy, Debug)]
pub enum Error {
    None,
    ScopePointerAllocationFailed,
    ContextAllocationFailed,
    InvalidEnvironment
}