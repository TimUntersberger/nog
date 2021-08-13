use crate::{config::Config, system::SystemResult, util, window::Window, NOG_POPUP_NAME};
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

    pub fn error(msg: Vec<String>, config: &Config) {
        Popup::new_error(msg).create(config).unwrap();
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
    pub fn create(&mut self, config: &Config) -> SystemResult<JoinHandle<()>> {
        if is_visible() {
            close()?;
        }

        let base_color = util::hex_to_rgb(config.bar.color);
        let background_color = (base_color.0 as u8, base_color.1 as u8, base_color.2 as u8);
        let text = self.text.clone();
        let text_color = if config.light_theme {
            (0, 0, 0)
        } else {
            (255, 255, 255)
        };
        let font_size = config.bar.font_size as usize;
        let height = self.text.len() * font_size;

        let t = EventLoopExtWindows::new_any_thread(move || {
            IcedPopup::run(Settings {
                window: window::Settings {
                    position: window::Position::Centered,
                    size: (200, height as u32),
                    decorations: false,
                    resizable: false,
                    always_on_top: true,
                    ..Default::default()
                },
                flags: PopupSettings {
                    content: text,
                    text_color,
                    background_color,
                },
                default_text_size: font_size as u16,
                ..Default::default()
            })
            .unwrap();
        });

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

    *POPUP.lock() = None;

    Ok(())
}

/// Is there a popup currently visible?
pub fn is_visible() -> bool {
    POPUP.lock().is_some()
}

use iced::{window, Align, Application, Clipboard, Column, Command, Element, Settings, Text};

#[derive(Clone, Debug)]
enum PopupMessage {}

struct IcedPopup {
    content: Vec<String>,
    text_color: iced::Color,
    background_color: iced::Color,
}

#[derive(Debug, Default, Clone)]
struct PopupSettings {
    content: Vec<String>,
    text_color: (u8, u8, u8),
    background_color: (u8, u8, u8),
}

impl Application for IcedPopup {
    type Executor = iced::executor::Default;
    type Message = PopupMessage;
    type Flags = PopupSettings;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                content: flags.content,
                text_color: iced::Color::from_rgb8(
                    flags.text_color.0,
                    flags.text_color.1,
                    flags.text_color.2,
                ),
                background_color: iced::Color::from_rgb8(
                    flags.background_color.0,
                    flags.background_color.1,
                    flags.background_color.2,
                ),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from(NOG_POPUP_NAME)
    }

    fn update(&mut self, message: Self::Message, _: &mut Clipboard) -> Command<Self::Message> {
        Command::none()
    }

    fn background_color(&self) -> iced::Color {
        self.background_color
    }

    fn should_exit(&self) -> bool {
        false
    }

    fn view(&mut self) -> Element<Self::Message> {
        let mut col = Column::new().padding(10).align_items(Align::Center);

        for line in &self.content {
            col = col.push(Text::new(line.clone()).color(self.text_color));
        }

        col.into()
    }
}
