use regex::Regex;

#[derive(Debug, Clone)]
pub struct Rule {
    pub pattern: Regex,
    pub has_custom_titlebar: bool,
    pub manage: bool,
    pub chromium: bool,
    pub firefox: bool,
    pub workspace_id: i32,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            pattern: Regex::new("").unwrap(),
            has_custom_titlebar: false,
            manage: true,
            chromium: false,
            firefox: false,
            workspace_id: -1,
        }
    }
}
