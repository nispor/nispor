#[derive(Debug, Clone)]
pub struct ZatelError {
    msg: String,
}

impl std::fmt::Display for ZatelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for ZatelError {
    /* TODO */
}
