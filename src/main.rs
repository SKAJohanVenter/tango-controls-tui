// #[allow(dead_code)]
// mod demo;
// #[allow(dead_code)]
// mod util;

mod app;
// mod ui;
mod stateful_tree;
mod tango_utils;
mod views;

use crate::stateful_tree::StatefulTree;
use app::App;
use argh::FromArgs;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
use std::sync::{Arc, Mutex};
use std::{
    error::Error,
    io::stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tango_utils::{GetTreeItems, TangoDevicesLookup};
use tui::{backend::CrosstermBackend, Terminal};
use views::{View, ViewExplorerHome};

enum Event<I> {
    Input(I),
    Tick,
    // UpdateTangoDeviceList(TangoDevicesLookup<'a>),
}

/// Crossterm demo
#[derive(Debug, FromArgs)]
struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "250")]
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    #[argh(option, default = "true")]
    enhanced_graphics: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();
    let tango_update_tx = tx.clone();

    let tick_rate = Duration::from_millis(cli.tick_rate);

    // Do the tick in the background
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut app = App::new("Tango Controls TUI", cli.enhanced_graphics);

    // Get the tango devices
    // thread::spawn(move || match TangoDevicesLookup::build() {
    //     Ok(tdl) => {
    //         tango_update_tx
    //             .send(Event::UpdateTangoDeviceList(tdl))
    //             .unwrap();
    //     }
    //     Err(_) => {}
    // });

    terminal.clear()?;

    loop {
        terminal.draw(|f| app.draw(f))?;
        // println!("{:#?}", app.tango_devices_lookup);
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(std::io::stdout(), DisableMouseCapture)?;
                    terminal.clear()?;
                    app.should_quit = true;
                    break;
                }
                _ => {
                    app.handle_event(&event);
                }
            },
            Event::Tick => {
                app.on_tick();
            } // Event::UpdateTangoDeviceList(tdl) => {
              // let a = tdl.get_tree_items();
              // app.tango_devices_lookup = tdl;
              // app.tango_devices_lookup = tdl;
              // app.views[0] = View::ExplorerHome(ViewExplorerHome {
              //     id: 0,
              //     stateful_tree: StatefulTree::with_items(a),
              // });
              // let a = app.views.get_mut(0).unwrap();
              // match a {
              //     View::ExplorerHome(eh) => {
              //         eh.stateful_tree =
              //             StatefulTree::with_items(app.tango_devices_lookup.get_tree_items())
              //     }
              //     _ => {}
              // }

              // let mut view = app.views.get_mut(0).unwrap();
              // match view {
              //     View::ExplorerHome(eh) => {
              //         eh.stateful_tree =
              //             StatefulTree::with_items(app.tango_devices_lookup.get_tree_items());
              //     }
              //     _ => {}
              // }
              // }
        }

        if app.should_quit {
            terminal.clear()?;
        }

        if app.should_quit || app.tango_host.is_empty() {
            disable_raw_mode()?;
            execute!(std::io::stdout(), DisableMouseCapture)?;
            break;
        }
    }
    Ok(())
}
