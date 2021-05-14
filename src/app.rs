use crate::views::explorer::ViewExplorerHome;
use crate::views::watchlist::ViewWatchList;
use crate::views::{Draw, SharedViewState, View};
use crate::{tango_utils::TangoDevicesLookup, views::AttributeReadings};

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use std::{collections::HashMap, env};
use tui::{backend::Backend, Frame};
#[derive(Default)]
pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub update_tango_device_list: bool,
    pub enhanced_graphics: bool,
    pub views: HashMap<String, View<'a>>,
    pub shared_view_state: SharedViewState<'a>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool) -> App<'a> {
        let mut app = App {
            title,
            should_quit: false,
            update_tango_device_list: true,
            enhanced_graphics,
            views: HashMap::new(),
            shared_view_state: SharedViewState::default(),
        };

        app.shared_view_state.tango_host = match env::var("TANGO_HOST") {
            Ok(host) => Some(host),
            Err(_) => None,
        };

        match TangoDevicesLookup::build() {
            Ok(tdl) => {
                let explorer_view = View::Explorer(ViewExplorerHome::new(&tdl));
                app.shared_view_state.current_view = explorer_view.to_string();
                app.views.insert(explorer_view.to_string(), explorer_view);
                app.shared_view_state.tango_devices_lookup = tdl;
            }
            Err(_) => {
                panic!("Could not get Tango devices")
            }
        };
        let watchlist_view = View::WatchList(ViewWatchList::new());
        app.views.insert(watchlist_view.to_string(), watchlist_view);
        app
    }

    pub fn handle_event(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Tab => {
                self.shared_view_state.toggle_current_view();
            }
            _ => {}
        }

        let current_view = self
            .views
            .get_mut(&self.shared_view_state.current_view)
            .unwrap();
        match current_view {
            View::Explorer(eh) => eh.handle_event(key_event, &mut self.shared_view_state),
            View::WatchList(wl) => wl.handle_event(key_event, &mut self.shared_view_state),
        };
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let view = self
            .views
            .get(&self.shared_view_state.current_view)
            .unwrap();
        self.shared_view_state.current_view = view.to_string();
        match view {
            View::Explorer(eh) => {
                eh.draw(f, &mut self.shared_view_state, view.into());
            }
            View::WatchList(wl) => {
                wl.draw(f, &mut self.shared_view_state, view.into());
            }
        }
    }

    pub fn update_device_attr_map(&mut self, attr_map: AttributeReadings) {
        match self.shared_view_state.watch_list.try_lock() {
            Ok(mut wl) => {
                *wl = attr_map;
            }
            Err(_) => {}
        }
    }

    pub fn on_tick(&mut self) {
        // Unimplemented for now
    }
}
