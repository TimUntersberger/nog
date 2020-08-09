#[derive(Debug, Clone)]
pub struct WorkspaceSetting {
    pub id: i32,
    pub monitor: i32,
    pub text: String
}

impl Default for WorkspaceSetting {
    fn default() -> Self {
        Self {
            id: -1,
            monitor: -1,
            text: "".into()
        }
    }
}