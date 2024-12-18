#[derive(Debug)]
pub enum OverlayError {
    RegisterClassA,
    CreateWindowExA,
    SetLayeredWindowAttributes,
}

impl std::fmt::Display for OverlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for OverlayError {}
