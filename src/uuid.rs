use uuid as _uuid;
pub type Uuid = String;
pub fn uuid() -> Uuid { _uuid::Uuid::new_v4().to_string() }