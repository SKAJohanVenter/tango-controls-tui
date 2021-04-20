mod app;

// mod ui;
mod stateful_tree;
mod tango_utils;
mod views;

use crate::views::{AttributeName, AttributeValue, DeviceName};
use app::App;
use clap::{AppSettings, Clap};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnableLineWrap, EnterAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, stdout, Write};
use std::{
    collections::BTreeMap,
    env,
    sync::{Arc, Mutex},
};
use std::{
    error::Error,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};

enum Event<I> {
    Input(I),
    Tick,
    UpdateTangoDeviceList(BTreeMap<DeviceName, BTreeMap<AttributeName, AttributeValue>>),
}

#[derive(Clap)]
#[clap(version = "0.1", author = "Johan Venter <a.johan.venter@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Cli {
    #[clap(short, long, default_value = "250")]
    tick_rate: u64,

    #[clap(short, long)]
    enhanced_graphics: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::parse();

    let tango_host = match env::var("TANGO_HOST") {
        Ok(host) => host,
        Err(_) => String::from(""),
    };
    if tango_host.is_empty() {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(b"\nTANGO_HOST not set\n\n")?;
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableLineWrap
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup     handling
    let (tx, rx) = mpsc::channel();
    let tx_watch_list = tx.clone();

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

    // Update the watched attributes in a separate thread
    let watch_list = Arc::clone(&app.shared_view_state.watch_list);
    let watch_sleep = Duration::from_millis(cli.tick_rate);
    thread::spawn(move || {
        loop {
            thread::sleep(watch_sleep);
            let mut device_attr_map = { watch_list.lock().unwrap().clone() };

            for (device_name, attr_map) in device_attr_map.iter_mut() {
                let attrs: Vec<String> = attr_map.keys().cloned().collect();
                for attr_name in attrs {
                    let new_value = match tango_utils::read_attribute(device_name, &attr_name) {
                        Ok(value) => match value.data.into_string() {
                            Ok(val) => Some(val),
                            // Looks like err is a valid value
                            Err(err) => Some(format!("{}", err)),
                        },
                        Err(err) => Some(format!("Error: {}", err)),
                    };
                    attr_map.insert(attr_name, new_value);
                }
            }
            match tx_watch_list.send(Event::UpdateTangoDeviceList(device_attr_map)) {
                Ok(_) => {}
                Err(err) => return err,
            }
        }
    });

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
            Event::UpdateTangoDeviceList(updated_device_value_map) => {
                app.update_device_attr_map(updated_device_value_map);
            }
        }

        if app.should_quit {
            disable_raw_mode()?;
            execute!(std::io::stdout(), DisableMouseCapture)?;
            break;
        }
    }
    Ok(())
}
