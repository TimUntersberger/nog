use crate::{
    config::{rule::Rule, Config},
    system::{NativeWindow, SystemError, SystemResult, WindowId},
    tile_grid::store::Store,
    AppState,
};
use log::debug;
use std::collections::HashMap;

static NUMBER_OF_WORKSPACES: usize = 10;
static GLOBAL_INDEX: i32 = 0; // index of the container storing globally pinned windows

#[derive(Clone, Debug)]
pub struct Pinned {
    // index 0 is global, 1-10 are workspaces
    containers: Vec<PinnedContainer>,
}

#[derive(Clone, Debug)]
struct PinnedContainer {
    is_visible: bool,
    windows: HashMap<i32, NativeWindow>,
}

impl PinnedContainer {
    pub fn new() -> Self {
        PinnedContainer {
            is_visible: true,
            windows: HashMap::new(),
        }
    }

    pub fn cleanup(&mut self) -> SystemResult {
        for window in self.windows.values_mut() {
            if window.is_hidden() {
                window.show();
            }

            window.remove_topmost()?;
            window.cleanup()?;
        }

        self.windows.clear();
        self.is_visible = false;

        Ok(())
    }

    pub fn insert(&mut self, window_id: i32, window: NativeWindow) {
        self.windows.insert(window_id, window);
        self.is_visible = true;
    }

    pub fn contains(&self, window_id: &i32) -> bool {
        self.windows.contains_key(window_id.into())
    }

    pub fn remove(&mut self, window_id: &i32) -> Option<NativeWindow> {
        let window = self.windows.remove(window_id.into());
        if self.windows.len() == 0 {
            self.is_visible = false;
        }
        window
    }
}

impl Pinned {
    pub fn new() -> Self {
        Pinned {
            containers: vec![PinnedContainer::new(); NUMBER_OF_WORKSPACES + 1],
        }
    }

    pub fn load(
        pinned_windows: Vec<(bool, Vec<i32>)>,
        state: &AppState,
    ) -> Result<Pinned, SystemError> {
        let mut pinned = Pinned::new();
        let pinned_windows = pinned_windows.into_iter().enumerate();
        for (index, (is_visible, pinned_ids)) in pinned_windows {
            let index = index as i32;
            for pinned_id in pinned_ids {
                let window_id: WindowId = pinned_id.into();
                let window: NativeWindow = NativeWindow::from(window_id);
                if pinned.can_pin(&window) {
                    pinned.pin_window(
                        window,
                        Some(index),
                        &state.config,
                        &state.additonal_rules,
                    )?;
                }
            }

            if !is_visible {
                pinned.toggle_view_pinned(Some(index))?;
            } else {
                pinned.store(Some(index));
            }
        }

        Ok(pinned)
    }

    pub fn cleanup(&mut self) -> SystemResult {
        for container in self.containers.iter_mut() {
            container.cleanup()?;
        }

        Ok(())
    }

    pub fn can_pin(&self, window: &NativeWindow) -> bool {
        !self.is_pinned(&window.id.into()) && window.is_window()
    }

    pub fn is_pinned(&self, window_id: &i32) -> bool {
        self.containers.iter().any(|c| c.contains(window_id.into()))
    }

    pub fn contains(&self, window_id: &i32, ws_id: Option<i32>) -> bool {
        let container_index = ws_id.unwrap_or(GLOBAL_INDEX) as usize;
        self.containers[container_index].contains(window_id)
    }

    pub fn is_empty(&self, ws_id: Option<i32>) -> bool {
        let container_index = ws_id.unwrap_or(GLOBAL_INDEX) as usize;
        self.containers[container_index].windows.is_empty()
    }

    pub fn pin_window(
        &mut self,
        mut window: NativeWindow,
        ws_id: Option<i32>,
        config: &Config,
        additional_rules: &Vec<Rule>,
    ) -> SystemResult {
        let container_index = ws_id.unwrap_or(GLOBAL_INDEX) as usize;
        let rules = config.rules.iter().chain(additional_rules.iter()).collect();

        window.set_matching_rule(rules);
        window.init(false, config.use_border)?;

        if !window.is_visible() {
            window.show();
        }

        window.to_foreground(true)?;

        self.containers[container_index].insert(window.id.into(), window);

        Ok(())
    }

    pub fn toggle_pin(
        &mut self,
        win_id: i32,
        ws_id: Option<i32>,
        config: &Config,
        additional_rules: &Vec<Rule>,
    ) -> SystemResult {
        let container_index = ws_id.unwrap_or(GLOBAL_INDEX) as usize;
        let win_id: WindowId = win_id.into();
        let window: NativeWindow = NativeWindow::from(win_id);

        if self.containers[container_index].contains(&window.id.into()) {
            let mut window = self.containers[container_index]
                .remove(&window.id.into())
                .unwrap();
            window.remove_topmost()?;
            window.cleanup()?;
        } else if self.can_pin(&window) {
            self.pin_window(window, ws_id, config, additional_rules)?;
        }

        self.store(ws_id);

        Ok(())
    }

    pub fn toggle_view_pinned(&mut self, ws_id: Option<i32>) -> SystemResult {
        let container_index = ws_id.unwrap_or(GLOBAL_INDEX) as usize;
        if self.containers[container_index].is_visible {
            self.containers[container_index]
                .windows
                .values_mut()
                .for_each(|w| w.hide());
            self.containers[container_index].is_visible = false;
        } else {
            for window in self.containers[container_index].windows.values_mut() {
                window.show();
                window.focus()?;
            }
            self.containers[container_index].is_visible = true;
        }

        self.store(ws_id);

        Ok(())
    }

    pub fn store(&self, ws_id: Option<i32>) {
        let container_index = ws_id.unwrap_or(GLOBAL_INDEX) as usize;
        let window_ids = self.containers[container_index]
            .windows
            .keys()
            .collect::<Vec<_>>();
        let pinned_visible = self.containers[container_index].is_visible;

        Store::save_pinned(container_index, window_ids, pinned_visible);
    }

    pub fn each_pinned_window(
        &mut self,
        callback: impl Fn(&mut NativeWindow) -> SystemResult + Copy,
    ) -> SystemResult {
        for container in self.containers.iter_mut() {
            for window in container.windows.values_mut() {
                callback(window)?;
            }
        }

        Ok(())
    }

    pub fn get(&self, window_id: &i32, ws_id: Option<i32>) -> Option<&NativeWindow> {
        let container_index = ws_id.unwrap_or(GLOBAL_INDEX) as usize;
        self.containers[container_index].windows.get(window_id)
    }

    pub fn get_mut(&mut self, window_id: &i32) -> Option<&mut NativeWindow> {
        for container in self.containers.iter_mut() {
            if container.contains(window_id) {
                return container.windows.get_mut(window_id);
            }
        }

        None
    }

    pub fn get_active_workspaces(&self) -> Vec<i32> {
        self.containers
            .iter()
            .enumerate()
            .filter(|(i, c)| *i != GLOBAL_INDEX as usize && !c.windows.is_empty())
            .map(|(i, _)| i as i32)
            .collect()
    }

    pub fn show(&mut self, workspace_ids: Vec<i32>) -> SystemResult {
        let (show_containers, hide_containers): (
            Vec<(usize, &mut PinnedContainer)>,
            Vec<(usize, &mut PinnedContainer)>,
        ) = self
            .containers
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i != GLOBAL_INDEX as usize)
            .partition(|(i, container)| {
                workspace_ids.contains(&(*i as i32)) && container.is_visible
            });

        for (i, container) in show_containers {
            debug!("Showing {}", i);
            for window in container.windows.values_mut() {
                if window.is_hidden() {
                    window.show();
                }
            }
        }

        for (i, container) in hide_containers {
            debug!("Hiding {}", i);
            for window in container.windows.values_mut() {
                if window.is_visible() {
                    window.hide();
                }
            }
        }

        Ok(())
    }

    pub fn remove(&mut self, window_id: &i32) -> Option<NativeWindow> {
        for (container_id, container) in self.containers.iter_mut().enumerate() {
            if container.contains(window_id) {
                let window = container.remove(window_id);

                self.store(Some(container_id as i32));

                return window;
            }
        }

        None
    }
}
