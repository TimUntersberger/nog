#[derive(Clone, Debug)]
pub struct Repository {
    origin: String,
    name: String,
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            origin: "TimUntersberger".into(),
            name: "wwm".into(),
        }
    }
}

impl From<String> for Repository {
    fn from(s: String) -> Self {
        let mut tokens = s.split('/');
        let origin = tokens.next().unwrap_or_default().into();
        let name = tokens.next().unwrap_or_default().into();

        Self { origin, name }
    }
}
