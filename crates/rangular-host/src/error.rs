#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostError {
    ReadOnly(&'static str),
    UnknownMember(String),
    CallFailed(String),
}
