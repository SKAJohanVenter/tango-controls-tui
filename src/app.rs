use crate::views::command::ViewCommand;
use crate::views::confirm_command::ViewConfirmCommand;
use crate::views::explorer::ViewExplorerHome;
use crate::views::watchlist::ViewWatchList;
use crate::views::{Draw, SharedViewState, View, ViewType};
use crate::Event;
use crate::{tango_utils::TangoDevicesLookup, views::AttributeReadings};

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use std::error::Error;
use std::sync::mpsc;
use std::{collections::HashMap, env};
use tui::{backend::Backend, Frame};

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub update_tango_device_list: bool,
    pub enhanced_graphics: bool,
    pub views: HashMap<View, ViewType<'a>>,
    pub shared_view_state: SharedViewState<'a>,
}

impl<'a> App<'a> {
    pub fn new(
        title: &'a str,
        enhanced_graphics: bool,
        tx_command: mpsc::Sender<Event>,
    ) -> Result<App<'a>, Box<dyn Error>> {
        let mut app = App {
            title,
            should_quit: false,
            update_tango_device_list: true,
            enhanced_graphics,
            views: HashMap::new(),
            shared_view_state: SharedViewState::new(tx_command),
        };

        app.shared_view_state.tango_host = match env::var("TANGO_HOST") {
            Ok(host) => Some(host),
            Err(_) => None,
        };

        match TangoDevicesLookup::build() {
            Ok(tdl) => {
                let explorer_view_type = ViewType::Explorer(ViewExplorerHome::new(&tdl));
                app.shared_view_state.current_view = View::Explorer;
                app.views.insert(View::Explorer, explorer_view_type);
                app.shared_view_state.tango_devices_lookup = tdl;
            }
            Err(err) => {
                return Err(err);
            }
        };
        let watchlist_view = ViewType::WatchList(ViewWatchList::new());
        app.views.insert(View::WatchList, watchlist_view);

        let command_view = ViewType::Command(ViewCommand::new());
        app.views.insert(View::Command, command_view);

        let confirm_view = ViewType::ConfirmCommand(ViewConfirmCommand::new());
        app.views.insert(View::ConfirmCommand, confirm_view);

        Ok(app)
    }

    pub fn handle_event(&mut self, key_event: &KeyEvent) {
        if let KeyCode::Tab = key_event.code {
            self.shared_view_state.toggle_current_view();
        }

        let current_view = self
            .views
            .get_mut(&self.shared_view_state.current_view)
            .unwrap();
        match current_view {
            ViewType::Explorer(eh) => eh.handle_event(key_event, &mut self.shared_view_state),
            ViewType::WatchList(wl) => wl.handle_event(key_event, &mut self.shared_view_state),
            ViewType::Command(co) => co.handle_event(key_event, &mut self.shared_view_state),
            ViewType::ConfirmCommand(po) => po.handle_event(key_event, &mut self.shared_view_state),
        };
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let view = self
            .views
            .get(&self.shared_view_state.current_view)
            .unwrap();
        self.shared_view_state.current_view = view.into();
        match view {
            ViewType::Explorer(eh) => {
                eh.draw(f, &mut self.shared_view_state, view.into());
            }
            ViewType::WatchList(wl) => {
                wl.draw(f, &mut self.shared_view_state, view.into());
            }

            ViewType::Command(co) => {
                co.draw(f, &mut self.shared_view_state, view.into());
            }
            ViewType::ConfirmCommand(po) => {
                po.draw(f, &mut self.shared_view_state, view.into());
            }
        }
    }

    pub fn update_device_attr_map(&mut self, attr_map: AttributeReadings) {
        if let Ok(mut wl) = self.shared_view_state.watch_list.try_lock() {
            *wl = attr_map;
        }
    }

    pub fn on_tick(&mut self) {
        // Unimplemented for now
    }
}
