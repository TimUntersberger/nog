#[derive(Debug, Clone)]
pub struct WorkspaceSetting {
    pub id: i32,
    pub monitor: i32,
}

impl Default for WorkspaceSetting {
    fn default() -> Self {
        Self {
            id: -1,
            monitor: -1,
        }
    }
}