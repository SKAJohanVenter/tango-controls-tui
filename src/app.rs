use crate::tango_utils::TangoDevicesLookup;
use crate::views::explorer::ViewExplorerHome;
use crate::views::watchlist::ViewWatchList;
use crate::views::{AttributeName, AttributeValue, DeviceName};
use crate::views::{Draw, SharedViewState, TabChoice, View};
use std::collections::BTreeMap;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use std::env;
use tui::{backend::Backend, Frame};
#[derive(Default)]
pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub update_tango_device_list: bool,
    pub enhanced_graphics: bool,
    pub current_view_ix: usize,
    pub views: Vec<View<'a>>,
    pub tango_devices_lookup: TangoDevicesLookup<'a>,
    pub shared_view_state: SharedViewState,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool) -> App<'a> {
        let mut app = App {
            title,
            should_quit: false,
            update_tango_device_list: true,
            enhanced_graphics,
            current_view_ix: 0,
            views: vec![],
            tango_devices_lookup: TangoDevicesLookup::default(),
            shared_view_state: SharedViewState::default(),
        };

        app.shared_view_state.tango_host = match env::var("TANGO_HOST") {
            Ok(host) => Some(host),
            Err(_) => None,
        };

        if let Ok(tdl) = TangoDevicesLookup::build() {
            app.views
                .push(View::Explorer(ViewExplorerHome::new(0, &tdl)));
            app.tango_devices_lookup = tdl;
        }
        app.views.push(View::WatchList(ViewWatchList::new(1)));
        app
    }

    pub fn handle_event(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Tab => {
                if self.current_view_ix == 0 {
                    self.current_view_ix = 1;
                } else {
                    self.current_view_ix = 0;
                }
            }
            _ => {}
        }

        let current_view = self.views.get_mut(self.current_view_ix).unwrap();
        match current_view {
            View::Explorer(eh) => eh.handle_event(
                key_event,
                &self.tango_devices_lookup,
                &mut self.shared_view_state,
            ),
            View::WatchList(wl) => wl.handle_event(
                key_event,
                &self.tango_devices_lookup,
                &mut self.shared_view_state,
            ),
        };
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let view = self.views.get(self.current_view_ix).unwrap();
        match view {
            View::Explorer(eh) => {
                self.shared_view_state.current_tab = TabChoice::Explorer;
                eh.draw(f, &mut self.shared_view_state);
            }
            View::WatchList(wl) => {
                self.shared_view_state.current_tab = TabChoice::WatchList;
                wl.draw(f, &mut self.shared_view_state);
            }
        }
    }

    pub fn update_device_attr_map(
        &mut self,
        attr_map: BTreeMap<DeviceName, BTreeMap<AttributeName, AttributeValue>>,
    ) {
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
