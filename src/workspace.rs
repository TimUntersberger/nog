pub struct Workspace {
    pub id: i32,
    pub visible: bool
}

impl Workspace {
    pub fn new(id: i32) -> Self {

        Self {
            id,
            visible: false
        }
    }
}