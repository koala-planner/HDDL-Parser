use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub struct TokenPosition {
    pub line: u32,
}
