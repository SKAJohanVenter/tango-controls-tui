// #[allow(dead_code)]
// mod demo;
// #[allow(dead_code)]
// mod util;

mod app;
// mod ui;
mod stateful_tree;
mod tango_utils;
mod views;

use app::App;
use argh::FromArgs;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
use std::{
    error::Error,
    io::stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};

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
                match tx.send(Event::Tick) {
                    Ok(_) => {}
                    Err(_) => {}
                }
                last_tick = Instant::now();
            }
        }
    });

    let mut app = App::new("Tango Controls TUI", cli.enhanced_graphics);

    terminal.clear()?;

    loop {
        terminal.draw(|f| app.draw(f))?;
        // println!("{:#?}", app.tango_devices_lookup);
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Esc => {
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
            }
        }

        if app.should_quit {
            terminal.clear()?;
        }

        if app.should_quit || app.shared_view_state.tango_host.is_none() {
            disable_raw_mode()?;
            execute!(std::io::stdout(), DisableMouseCapture)?;
            break;
        }
    }
    Ok(())
}
