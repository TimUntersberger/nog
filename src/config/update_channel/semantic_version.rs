use std::str::FromStr;
#[derive(Clone, Debug, Default)]
pub struct SemanticVersion {
    pub major: i32,
    pub minor: i32,
    pub patch: i32
}

impl From<String> for SemanticVersion {
    fn from(s: String) -> Self {
        let mut tokens = s.split(".");
        let major = i32::from_str(tokens.next().unwrap_or_default()).unwrap_or_default();
        let minor = i32::from_str(tokens.next().unwrap_or_default()).unwrap_or_default();
        let patch = i32::from_str(tokens.next().unwrap_or_default()).unwrap_or_default();

        Self {
            major,
            minor,
            patch
        }
    }
}