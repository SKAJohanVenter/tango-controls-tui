use crate::stateful_tree::StatefulTree;
// use crate::tango_utils::GetTreeItems;
use crate::tango_utils::TangoDevicesLookup;
use crate::views::{
    Draw, View, ViewExplorerAttributes, ViewExplorerCommands, ViewExplorerHome, ViewWatchList,
};

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
    pub tango_host: String,
    pub tango_devices_lookup: TangoDevicesLookup<'a>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool) -> App<'a> {
        let tango_host = match env::var("TANGO_HOST") {
            Ok(host) => host,
            Err(_) => "".to_string(),
        };
        let mut app = App {
            title,
            should_quit: false,
            update_tango_device_list: true,
            enhanced_graphics,
            current_view_ix: 0,
            views: vec![
                // View::ExplorerHome(ViewExplorerHome::new(0)),
                View::ExplorerCommands(ViewExplorerCommands::new(1)),
                View::ExplorerAttributes(ViewExplorerAttributes::new(2)),
                View::WatchList(ViewWatchList::new(3)),
            ],
            tango_host,
            tango_devices_lookup: TangoDevicesLookup::default(),
            // tango_devices_lookup: TangoDevicesLookup::default(),
        };

        if let Ok(tdl) = TangoDevicesLookup::build() {
            app.views
                .insert(0, View::ExplorerHome(ViewExplorerHome::new(0, &tdl)));
            app.tango_devices_lookup = tdl;
        }
        app
    }

    pub fn handle_event(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Tab => {
                if self.current_view_ix == 0 {
                    self.current_view_ix = 3;
                } else {
                    self.current_view_ix = 0;
                }
            }
            _ => {}
        }
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let view = self.views.get(self.current_view_ix).unwrap();
        match view {
            View::ExplorerHome(eh) => {
                eh.draw(f, self);
            }
            View::ExplorerCommands(ec) => {
                ec.draw(f, self);
            }
            View::ExplorerAttributes(ea) => {
                ea.draw(f, self);
            }
            View::WatchList(wl) => {
                wl.draw(f, self);
            }
        }
    }

    pub fn on_tick(&mut self) {}

    pub fn get_current_view(&self) -> &View {
        self.views.get(self.current_view_ix).unwrap()
    }

    // pub fn get_current_view(&mut self) -> Box<dyn Draw> {
    //     let view = self.views.get(self.current_view_ix).unwrap();
    //     match view {
    //         View::ExplorerHome(eh) => {
    //             eh
    //             // eh.draw(f, self)
    //         }
    //         View::ExplorerCommands(ec) => {
    //             ec
    //             // ec.draw(f, self)
    //         }
    //         View::ExplorerAttributes(ea) => {
    //             ea
    //             // ea.draw(f, self)
    //         }
    //         View::WatchList(wl) => {
    //             wl
    //             // wl.draw(f, self)
    //         }
    //     }
    // }
}
