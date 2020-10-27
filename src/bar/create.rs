use super::{
    component::Component, component::ComponentText, item::Item, item_section::ItemSection, Bar,
};
use crate::{
    config::Config, display::Display, event::Event, system::Rectangle, window::Api,
    window::WindowEvent, AppState, NOG_BAR_NAME,
};
use log::{debug, error, info};
use parking_lot::Mutex;
use std::sync::Arc;

fn draw_component_text(
    api: &Api,
    rect: &Rectangle,
    config: &Config,
    component_text: &ComponentText,
) {
    if component_text.display_text.is_empty() {
        return;
    }

    let fg = Some(component_text.foreground_color)
        .filter(|x| *x > 0)
        .unwrap_or(if config.light_theme {
            0x00333333
        } else {
            0x00ffffff
        });

    let bg = Some(component_text.background_color)
        .filter(|x| *x > 0)
        .unwrap_or(config.bar.color);

    api.set_text_color(fg);
    api.set_background_color(bg);
    api.write_text(
        &component_text.display_text,
        rect.left,
        rect.top,
        true,
        false,
    )
}

fn draw_components(
    api: &Api,
    display: &Display,
    state: &AppState,
    mut offset: i32,
    components: &[Component],
) {
    for component in components {
        let component_texts = component.render(display, state);

        for (_i, component_text) in component_texts.iter().enumerate() {
            let width = api
                .calculate_text_rect(&component_text.display_text)
                .width();

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
    components: &[Component],
) -> ItemSection {
    let mut section = ItemSection::default();
    let mut component_offset = 0;

    for component in components {
        let mut item = Item::default();
        let mut component_text_offset = 0;
        let mut component_width = 0;

        for component_text in component.render(display, state) {
            let width = api
                .calculate_text_rect(&component_text.display_text)
                .width();
            let left = component_text_offset;
            let right = component_text_offset + width;

            item.cached_result.push(((left, right), component_text));

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

pub fn create(state_arc: Arc<Mutex<AppState>>) {
    info!("Creating appbar");

    let mut state = state_arc.lock();
    let config = state.config.clone();

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
            .with_title(NOG_BAR_NAME)
            .with_refresh_rate(100)
            .with_font(&config.bar.font)
            .with_font_size(config.bar.font_size)
            .with_background_color(config.bar.color)
            .with_pos(left, top)
            .with_size(width, config.bar.height);

        let sender = sender.clone();

        bar.window
            .create(state_arc.clone(), true, move |event| match event {
                WindowEvent::Native { msg, display, .. } => {
                    //TODO: make this cleaner
                    #[cfg(target_os = "windows")]
                    unsafe {
                        use winapi::um::shellapi::ABN_FULLSCREENAPP;
                        use winapi::um::winuser::WM_APP;

                        if msg.code == WM_APP + 1 {
                            if msg.params.0 == ABN_FULLSCREENAPP as usize {
                                sender
                                    .send(Event::ToggleAppbar(display.id))
                                    .expect("Failed to send ToggleAppbar event");
                            }
                        }
                    }
                }
                WindowEvent::Click {
                    x, display, state, ..
                } => {
                    display
                        .appbar
                        .as_ref()
                        .and_then(|b| b.item_at_pos(*x).cloned())
                        .map(|item| {
                            if item.component.is_clickable {
                                for (i, (width, text)) in item.cached_result.iter().enumerate() {
                                    if width.0 <= *x && *x <= width.1 {
                                        item.component.on_click(display, state, text.value.clone(), i);
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
                        let left = components_to_section(
                            api,
                            &display,
                            state,
                            &state.config.bar.components.left,
                        );

                        let mut center = components_to_section(
                            api,
                            &display,
                            state,
                            &state.config.bar.components.center,
                        );

                        center.left = working_area_width / 2 - center.right / 2;
                        center.right += center.left;

                        let mut right = components_to_section(
                            api,
                            &display,
                            state,
                            &state.config.bar.components.right,
                        );
                        right.left = working_area_width - right.right;
                        right.right += right.left;

                        draw_components(
                            api,
                            &display,
                            state,
                            left.left,
                            &state.config.bar.components.left,
                        );
                        draw_components(
                            api,
                            &display,
                            state,
                            center.left,
                            &state.config.bar.components.center,
                        );
                        draw_components(
                            api,
                            &display,
                            state,
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

                        sender
                            .send(Event::UpdateBarSections(display.id, left, center, right))
                            .expect("Failed to send UpdateBarSections event");
                    }
                }
                _ => {}
            });

        display.appbar = Some(bar);
    }
}

#[test] #[ignore] // test never exits
pub fn test() {
    crate::logging::setup();
    // let state = AppState::new();
    // create(state);

    loop {
        sleep!(1000);
    }
}
