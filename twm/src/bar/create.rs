use super::{
    component::Component, component::ComponentText, item::Item, item_section::ItemSection, Bar,
};
use crate::{
    config::Config, display::Display, event::Event, system::DisplayId, system::Rectangle,
    window::Api, window::WindowEvent, AppState, NOG_BAR_NAME, util,
};
use log::{debug, error, info};
use mlua::Result as RuntimeResult;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

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
    config: &Config,
    display: &Display,
    mut offset: i32,
    components: &[Component],
) -> RuntimeResult<()> {
    for component in components {
        let display_id = display.id;
        let component_texts = component.render(display_id)?;

        for (_i, component_text) in component_texts.iter().enumerate() {
            let width = api
                .calculate_text_rect(&component_text.display_text)
                .width();

            let rect = Rectangle {
                left: offset,
                right: offset + width,
                bottom: util::points_to_pixels(config.bar.height, &display),
                top: 0,
            };

            offset = rect.right;

            draw_component_text(api, &rect, config, &component_text);
        }
    }

    Ok(())
}

fn components_to_section(
    api: &Api,
    display_id: DisplayId,
    components: &[Component],
) -> RuntimeResult<ItemSection> {
    let mut section = ItemSection::default();
    let mut component_offset = 0;

    for component in components {
        let mut item = Item::default();
        let mut component_text_offset = 0;
        let mut component_width = 0;

        for component_text in component.render(display_id)? {
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

    Ok(section)
}

fn clear_section(api: &Api, config: &Config, left: i32, right: i32, display: &Display) {
    let height = util::points_to_pixels(config.bar.height, display);
    api.fill_rect(left, 0, right - left, height, config.bar.color)
}

pub fn create_or_update(state_arc: Arc<Mutex<AppState>>) {
    info!("Creating appbar");

    let sender = state_arc
        .try_lock_for(Duration::from_millis(100))
        .unwrap()
        .event_channel
        .sender
        .clone();
    let displays = state_arc
        .try_lock_for(Duration::from_millis(100))
        .unwrap()
        .displays
        .clone();

    for display in displays {
        let config = state_arc
            .try_lock_for(Duration::from_millis(100))
            .unwrap()
            .config
            .clone();

        if let Some(existing_bar) = display.appbar {
            // Change height of existing bars in case the display has changed
            existing_bar.change_height(config.bar.height);
            continue;
        }

        debug!("Creating appbar for display {:?}", display.id);
        let mut bar = Bar::default();

        bar.display_id = display.id;

        let height = util::points_to_pixels(config.bar.height, &display);
        let left = display.working_area_left();
        let top = display.working_area_top(&config) - height;
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
            .with_size(width, height);

        let sender = sender.clone();
        let state_arc2 = state_arc.clone();

        bar.window.create(state_arc.clone(), true, move |event| {
            match event {
                WindowEvent::Native {
                    msg, display_id, ..
                } => {
                    //TODO: make this cleaner
                    #[cfg(target_os = "windows")]
                    {
                        use winapi::um::shellapi::ABN_FULLSCREENAPP;
                        use winapi::um::winuser::WM_APP;

                        if msg.code == WM_APP + 1 {
                            if msg.params.0 == ABN_FULLSCREENAPP as usize {
                                sender
                                    .send(Event::ToggleAppbar(*display_id))
                                    .expect("Failed to send ToggleAppbar event");
                            }
                        }
                    }
                }
                WindowEvent::Click {
                    x,
                    display_id,
                    state_arc,
                    ..
                } => {
                    let clickable_items = state_arc
                        .lock()
                        .get_display_by_id(*display_id)
                        .unwrap()
                        .appbar
                        .as_ref()
                        .and_then(|b| b.item_at_pos(*x).cloned())
                        .filter(|item| item.component.is_clickable);

                    for item in clickable_items {
                        for (i, (width, text)) in item.cached_result.iter().enumerate() {
                            if width.0 <= *x && *x <= width.1 {
                                item.component
                                    .on_click(*display_id, text.value.clone(), i)?;
                            }
                        }
                    }
                }
                WindowEvent::MouseMove {
                    x,
                    api,
                    display_id,
                    state_arc,
                    ..
                } => {
                    state_arc
                        .lock()
                        .get_display_by_id(*display_id)
                        .unwrap()
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
                    display_id,
                    state_arc,
                    ..
                } => {
                    if let Some(state) = state_arc.try_lock_for(Duration::from_millis(20)) {
                        let config = state.config.clone();
                        let bar = state.get_display_by_id(*display_id).unwrap().appbar.clone();
                        drop(state);

                        if let Some(bar) = bar {
                            api.with_font(&config.bar.font, config.bar.font_size, || {
                                let working_area_width = display.working_area_width(&config);
                                let left = components_to_section(
                                    api,
                                    *display_id,
                                    &config.bar.components.left,
                                )?;

                                let mut center = components_to_section(
                                    api,
                                    *display_id,
                                    &config.bar.components.center,
                                )?;

                                center.left = working_area_width / 2 - center.right / 2;
                                center.right += center.left;

                                let mut right = components_to_section(
                                    api,
                                    *display_id,
                                    &config.bar.components.right,
                                )?;
                                right.left = working_area_width - right.right;
                                right.right += right.left;

                                draw_components(
                                    api,
                                    &config,
                                    &display,
                                    left.left,
                                    &config.bar.components.left,
                                )?;
                                draw_components(
                                    api,
                                    &config,
                                    &display,
                                    center.left,
                                    &config.bar.components.center,
                                )?;
                                draw_components(
                                    api,
                                    &config,
                                    &display,
                                    right.left,
                                    &config.bar.components.right,
                                )?;

                                if bar.left.width() > left.width() {
                                    clear_section(api, &config, left.right, bar.left.right, &display);
                                }

                                if bar.center.width() > center.width() {
                                    clear_section(api, &config, bar.center.left, bar.center.right, &display);
                                }

                                if bar.right.width() > right.width() {
                                    clear_section(api, &config, bar.right.left, right.left, &display);
                                }

                                sender
                                    .send(Event::UpdateBarSections(display.id, left, center, right))
                                    .expect("Failed to send UpdateBarSections event");

                                Ok(())
                            })?;
                        }
                    }
                }
                _ => {}
            }

            Ok(())
        });

        state_arc
            .try_lock_for(Duration::from_millis(100))
            .unwrap()
            .get_display_by_id_mut(bar.display_id)
            .unwrap()
            .appbar = Some(bar.clone());
    }
}

#[test]
#[ignore] // test never exits
pub fn test() {
    crate::logging::setup();
    // let state = AppState::new();
    // create(state);

    loop {
        sleep!(1000);
    }
}
