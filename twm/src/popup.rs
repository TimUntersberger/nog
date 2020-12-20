use crate::{
    system::Rectangle, system::SystemResult, window::Window, window::WindowEvent, AppState,
    NOG_POPUP_NAME,
};
use parking_lot::Mutex;
use std::{fmt::Debug, sync::Arc, thread, thread::JoinHandle};

static POPUP: Mutex<Option<Popup>> = Mutex::new(None);

pub type PopupActionCallback = Arc<dyn Fn() -> () + Sync + Send>;

#[derive(Default, Clone)]
pub struct PopupAction {
    pub text: String,
    pub cb: Option<PopupActionCallback>,
}

impl Debug for PopupAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("PopupAction {{ text = {} }}", self.text))
    }
}

#[derive(Debug, Clone)]
pub struct Popup {
    window: Option<Window>,
    padding: i32,
    text: Vec<String>,
    pub actions: Vec<PopupAction>,
}

impl Popup {
    pub fn new() -> Self {
        Self {
            window: None,
            padding: 5,
            text: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn new_error(msg: Vec<String>) -> Self {
        Popup::new().with_padding(5).with_text(
            msg.into_iter()
                .chain(vec!["".into(), "(Press Alt+Q to close)".into()])
                .collect(),
        )
    }

    pub fn error(msg: Vec<String>, state_arc: Arc<Mutex<AppState>>) {
        thread::spawn(move || Popup::new_error(msg).create(state_arc).unwrap());
    }

    pub fn with_text<T: Into<String>>(mut self, text: Vec<T>) -> Self {
        self.text = text.into_iter().map(|x| x.into()).collect();
        self
    }

    pub fn with_padding(mut self, padding: i32) -> Self {
        self.padding = padding + 5;
        self
    }

    /// Creates the window for the popup with the configured parameters.
    ///
    /// This function closes a popup that is currently visible.
    pub fn create(&mut self, state_arc: Arc<Mutex<AppState>>) -> SystemResult<JoinHandle<()>> {
        if is_visible() {
            close()?;
        }

        let state = state_arc.lock();

        let text = self.text.join("\n");
        let padding = self.padding;

        let mut window = Window::new()
            .with_title(NOG_POPUP_NAME)
            .with_font(&state.config.bar.font)
            .with_size(10, 10)
            .with_font_size(state.config.bar.font_size)
            .with_is_popup(true)
            .with_background_color(state.config.bar.color);

        drop(state);

        let t = window.create(state_arc, true, move |event| {
            match event {
                WindowEvent::Draw { api, .. } => {
                    let rect = api.calculate_text_rect(&text);

                    let height = rect.height();
                    let width = rect.width();

                    let x = api.display.width() / 2 - width / 2 - padding;
                    let y = api.display.height() / 2 - height / 2 - padding;

                    api.window
                        .set_window_pos(
                            Rectangle {
                                left: x,
                                right: x + width + padding * 2,
                                top: y,
                                bottom: y + height + padding * 2,
                            },
                            None,
                            None,
                        )
                        .expect("Failed to move popup to its location");

                    api.set_text_color(0xffffff);
                    api.write_text(&text, padding, padding, false, false);
                }
                _ => {}
            }
            Ok(())
        });

        self.window = Some(window);
        *POPUP.lock() = Some(self.clone());

        Ok(t)
    }
}

pub fn cleanup() -> SystemResult {
    close()
}

/// Close the current popup, if there is one.
pub fn close() -> SystemResult {
    if let Some(window) = POPUP.lock().clone().and_then(|p| p.window) {
        window.close()?;
    }

    Ok(())
}

/// Is there a popup currently visible?
pub fn is_visible() -> bool {
    POPUP.lock().is_some()
}
