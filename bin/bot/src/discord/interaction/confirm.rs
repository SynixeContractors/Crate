#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Confirmation {
    Yes,
    No,
    Timeout,
}
