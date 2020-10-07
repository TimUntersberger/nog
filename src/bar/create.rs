use super::{
    component::Component, component::ComponentText, item::Item, item_section::ItemSection, Bar,
};
use crate::{
    config::Config, display::Display, event::Event, event::EventChannel, keybindings::KbManager,
    system::Rectangle, window::Api, window::WindowEvent, AppState,
};
use log::{debug, error, info};
use parking_lot::Mutex;
use std::{sync::Arc, thread, time::Duration};

fn spawn_refresh_thread(chan: &EventChannel) {
    let sender = chan.sender.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(200));

        // TODO: Somehow get notified when nog leaves work mode
        // Maybe just close the channel when we leave work mode and then check if the send failed
        // without panicking.

        sender
            .send(Event::RedrawAppBar)
            .expect("Failed to send redraw-app-bar event");
    });
}

fn draw_component_text(
    api: &Api,
    rect: &Rectangle,
    config: &Config,
    component_text: &ComponentText,
) {
    let text = component_text.get_text();

    if text.is_empty() {
        return;
    }

    let fg = component_text.get_fg().unwrap_or(if config.light_theme {
        0x00333333
    } else {
        0x00ffffff
    });

    let bg = component_text.get_bg().unwrap_or(config.bar.color as u32);

    api.set_text_color(fg);
    api.set_background_color(bg);
    api.write_text(&text, rect.left, rect.top, true, false)
}

fn draw_components(
    api: &Api,
    display: &Display,
    state: &AppState,
    kb_manager: &KbManager,
    mut offset: i32,
    components: &[Component],
) {
    for component in components {
        let component_texts = component.render(display, state, kb_manager);

        for (_i, component_text) in component_texts.iter().enumerate() {
            let width = api.calculate_text_rect(&component_text.get_text()).width();

            let rect = Rectangle {
                left: offset,
                right: offset + width,
                bottom: state.config.bar.height,
                top: 0,
            };

            offset = rect.right;

            draw_component_text(api, &rect, &state.config, &component_text);
        }
    }
}

fn components_to_section(
    api: &Api,
    display: &Display,
    state: &AppState,
    kb_manager: &KbManager,
    components: &[Component],
) -> ItemSection {
    let mut section = ItemSection::default();
    let mut component_offset = 0;

    for component in components {
        let mut item = Item::default();
        let mut component_text_offset = 0;
        let mut component_width = 0;

        for component_text in component.render(display, state, kb_manager) {
            let width = api.calculate_text_rect(&component_text.get_text()).width();
            let left = component_text_offset;
            let right = component_text_offset + width;

            item.widths.push((left, right));

            component_width += width;
            component_text_offset += width;
        }

        item.left = component_offset;
        item.right = item.left + component_width;
        item.component = component.clone();

        section.items.push(item);

        component_offset += component_width;
    }

    section.right = component_offset;

    section
}

fn clear_section(api: &Api, config: &Config, left: i32, right: i32) {
    api.fill_rect(left, 0, right - left, config.bar.height, config.bar.color)
}

pub fn create(state_arc: Arc<Mutex<AppState>>, kb_manager: Arc<Mutex<KbManager>>) {
    info!("Creating appbar");

    let name = "nog_bar";

    let state_arc = state_arc.clone();
    let mut state = state_arc.lock();
    let config = state.config.clone();

    spawn_refresh_thread(&state.event_channel);
    let sender = state.event_channel.sender.clone();

    for display in state.displays.iter_mut() {
        if display.appbar.is_some() {
            error!(
                "Appbar for monitor {:?} already exists. Aborting",
                display.id
            );
            continue;
        }

        debug!("Creating appbar for display {:?}", display.id);
        let mut bar = Bar::default();

        bar.display_id = display.id;

        let left = display.working_area_left();
        let top = display.working_area_top(&config) - config.bar.height;
        let width = display.working_area_width(&config);

        bar.window = bar
            .window
            .with_is_popup(true)
            .with_border(false)
            .with_title(name)
            .with_font(&config.bar.font)
            .with_background_color(config.bar.color as u32)
            .with_pos(left, top)
            .with_size(width, config.bar.height);

        let kb_manager = kb_manager.clone();
        let sender = sender.clone();

        bar.window
            .create(state_arc.clone(), true, move |event| match event {
                WindowEvent::Click {
                    x, display, state, ..
                } => {
                    display
                        .appbar
                        .as_ref()
                        .and_then(|b| b.item_at_pos(*x).cloned())
                        .map(|item| {
                            if item.component.is_clickable {
                                for (i, width) in item.widths.iter().enumerate() {
                                    if width.0 <= *x && *x <= width.1 {
                                        item.component.on_click(display, state, i);
                                    }
                                }
                            }
                        });
                }
                WindowEvent::MouseMove {
                    x, api, display, ..
                } => {
                    display
                        .appbar
                        .as_ref()
                        .and_then(|b| b.item_at_pos(*x))
                        .map(|item| {
                            if item.component.is_clickable {
                                api.set_clickable_cursor();
                            } else {
                                api.set_default_cursor();
                            }
                        })
                        .or_else(|| {
                            api.set_default_cursor();
                            None
                        });
                }
                WindowEvent::Draw {
                    api,
                    display,
                    state,
                    ..
                } => {
                    if let Some(bar) = display.appbar.as_ref() {
                        let working_area_width = display.working_area_width(&state.config);
                        let kb_manager = kb_manager.lock();
                        let left = components_to_section(
                            api,
                            &display,
                            state,
                            &kb_manager,
                            &state.config.bar.components.left,
                        );

                        let mut center = components_to_section(
                            api,
                            &display,
                            state,
                            &kb_manager,
                            &state.config.bar.components.center,
                        );

                        center.left = working_area_width / 2 - center.right / 2;
                        center.right += center.left;

                        let mut right = components_to_section(
                            api,
                            &display,
                            state,
                            &kb_manager,
                            &state.config.bar.components.right,
                        );
                        right.left = working_area_width - right.right;
                        right.right += right.left;

                        draw_components(
                            api,
                            &display,
                            state,
                            &kb_manager,
                            left.left,
                            &state.config.bar.components.left,
                        );
                        draw_components(
                            api,
                            &display,
                            state,
                            &kb_manager,
                            center.left,
                            &state.config.bar.components.center,
                        );
                        draw_components(
                            api,
                            &display,
                            state,
                            &kb_manager,
                            right.left,
                            &state.config.bar.components.right,
                        );

                        if bar.left.width() > left.width() {
                            clear_section(api, &state.config, left.right, bar.left.right);
                        }

                        if bar.center.width() > center.width() {
                            let delta = (bar.center.right - center.right) / 2;
                            clear_section(
                                api,
                                &state.config,
                                bar.center.left,
                                bar.center.left + delta,
                            );
                            clear_section(
                                api,
                                &state.config,
                                bar.center.right - delta,
                                bar.center.right,
                            );
                        }

                        if bar.right.width() > right.width() {
                            clear_section(api, &state.config, bar.right.left, right.left);
                        }

                        sender.send(Event::UpdateBarSections(display.id, left, center, right));
                    }
                }
                _ => {}
            });

        display.appbar = Some(bar);
    }
}

#[test]
pub fn test() {
    crate::logging::setup();
    // let state = AppState::new();
    // create(state);

    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}
