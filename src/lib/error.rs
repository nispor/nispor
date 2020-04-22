#[derive(Debug, Clone)]
pub struct NisporError {
    pub msg: String,
}

impl std::fmt::Display for NisporError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for NisporError {
    /* TODO */
}
