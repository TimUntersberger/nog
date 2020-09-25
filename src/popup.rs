use crate::{system::Rectangle, window::Window, window::WindowEvent, CONFIG};
use parking_lot::Mutex;
use std::sync::Arc;

static POPUP: Mutex<Option<Popup>> = Mutex::new(None);

pub type PopupActionCallback = Arc<dyn Fn() -> () + Sync + Send>;
#[derive(Default, Clone)]
pub struct PopupAction {
    pub text: String,
    pub cb: Option<PopupActionCallback>,
}

#[derive(Clone)]
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

    pub fn with_text(mut self, text: &[&str]) -> Self {
        self.text = text.iter().map(|x| x.to_string()).collect();
        self
    }

    pub fn with_padding(mut self, padding: i32) -> Self {
        self.padding = padding + 5;
        self
    }

    /// Creates the window for the popup with the configured parameters.
    ///
    /// This function closes a popup that is currently visible.
    pub fn create(&mut self) {
        if is_visible() {
            close();
        }

        let text = self.text.join("\n");
        let padding = self.padding;

        let (font, font_size, color) = {
            let config = CONFIG.lock();

            (
                config.bar.font.clone(),
                config.bar.font_size,
                config.bar.color,
            )
        };

        let mut window = Window::new()
            .with_title("NogPopup")
            .with_font(&font)
            .with_size(10, 10)
            .with_font_size(font_size)
            .with_is_popup(true)
            .with_background_color(color as u32);

        window.create(move |event| match event {
            WindowEvent::Draw { api, .. } => {
                let rect = api.calculate_text_rect(&text);

                let height = rect.height();
                let width = rect.width();

                let x = api.display.width() / 2 - width / 2 - padding;
                let y = api.display.height() / 2 - height / 2 - padding;

                api.window.set_window_pos(
                    Rectangle {
                        left: x,
                        right: x + width + padding * 2,
                        top: y,
                        bottom: y + height + padding * 2,
                    },
                    None,
                    None,
                );

                api.set_text_color(0xffffff);
                api.write_text(&text, padding, padding, false, false);
            }
            _ => {}
        });

        self.window = Some(window);
        *POPUP.lock() = Some(self.clone());
    }
}

pub fn cleanup() {
    close();
}

/// Close the current popup, if there is one.
pub fn close() {
    if let Some(window) = POPUP.lock().clone().and_then(|p| p.window) {
        window.close();
    }
}

/// Is there a popup currently visible?
pub fn is_visible() -> bool {
    POPUP.lock().is_some()
}

#[test]
pub fn test() {
    Popup::new().with_text(&vec!["hello", "world"]).create();

    loop {
        std::thread::sleep_ms(1000);
    }
}
