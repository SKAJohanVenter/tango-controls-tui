mod app;

// mod ui;
mod stateful_tree;
mod tango_utils;
mod views;

use app::App;
use clap;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnableLineWrap, EnterAlternateScreen},
};
use log::{error, info};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::Root,
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use std::io::{self, stdout, Write};
use std::{env, sync::Arc};
use std::{
    error::Error,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};
use views::AttributeReadings;

enum Event<I> {
    Input(I),
    Tick,
    UpdateTangoDeviceReadings(AttributeReadings),
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse args
    let matches = parse_commandline_args();
    let tick_rate = matches.value_of("tick_rate").unwrap().parse::<u64>()?;
    let enhanced_graphics = match matches.value_of("enhanced_graphics") {
        Some(_) => true,
        None => false,
    };

    //Set up logging
    let log_config = build_log_config(&matches)?;
    let _handle = log4rs::init_config(log_config)?;

    info!("Starting up");

    let tango_host = match env::var("TANGO_HOST") {
        Ok(host) => host,
        Err(_) => String::from(""),
    };
    if tango_host.is_empty() {
        error!("TANGO_HOST not set");
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

    let tick_rate_duration = Duration::from_millis(tick_rate);

    // Do the tick in the background
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate_duration
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate_duration {
                match tx.send(Event::Tick) {
                    Ok(_) => {}
                    Err(_) => {}
                }
                last_tick = Instant::now();
            }
        }
    });

    let mut app = match App::new("Tango Controls TUI", enhanced_graphics) {
        Ok(the_app) => the_app,
        Err(err) => {
            disable_raw_mode()?;
            execute!(std::io::stdout(), DisableMouseCapture)?;
            return Err(err);
        }
    };

    // Update the watched attributes in a separate thread
    let watch_list = Arc::clone(&app.shared_view_state.watch_list);
    let watch_sleep = Duration::from_millis(tick_rate);
    thread::spawn(move || loop {
        thread::sleep(watch_sleep);
        let mut device_attr_map = { watch_list.lock().unwrap().clone() };

        for (device_name, attr_map) in device_attr_map.iter_mut() {
            for (attr_name, attr_reading) in attr_map.iter_mut() {
                attr_reading.update(device_name, attr_name);
            }
        }
        match tx_watch_list.send(Event::UpdateTangoDeviceReadings(device_attr_map)) {
            Ok(_) => {}
            Err(err) => return err,
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
            Event::UpdateTangoDeviceReadings(updated_device_value_map) => {
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

fn validate_tick_rate(v: &str) -> Result<u64, String> {
    match v.parse::<u64>() {
        Ok(parsed_val) => Ok(parsed_val),
        Err(_) => Err("Tick rate should be a number".to_string()),
    }
}

fn parse_commandline_args() -> clap::ArgMatches {
    clap::App::new("tango-controls-tui")
        .version("0.0.1")
        .author("Johan Venter <a.johan.venter@gmail.com>")
        .about("A terminal application to explore Tango devices")
        .arg(
            clap::Arg::new("tick_rate")
                .short('t')
                .long("tick-rate")
                .about("The refresh rate")
                .validator(validate_tick_rate)
                .default_value("250"),
        )
        .arg(
            clap::Arg::new("enhanced_graphics")
                .short('e')
                .long("enhanced-graphics")
                .about("Whether to use unicode symbols for better rendering"),
        )
        .arg(
            clap::Arg::new("logfile_path")
                .short('l')
                .long("logfile")
                .about("The path to the log file. If not specified logs will be sent to stderr")
                .takes_value(true),
        )
        .get_matches()
}

fn build_log_config(matches: &clap::ArgMatches) -> Result<log4rs::Config, Box<dyn Error>> {
    // Build the logger
    // Write to a file if specified, otherwise write to stderr
    let level = log::LevelFilter::Info;
    let (destination, appender) = if let Some(logfile_path) = matches.value_of("logfile_path") {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} {l} {t} - {m}{n}")))
            .build(logfile_path)?;
        (
            "logfile",
            log4rs::config::Appender::builder().build("logfile", Box::new(logfile)),
        )
    } else {
        let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

        (
            "stderr",
            log4rs::config::Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
    };
    let config = log4rs::Config::builder()
        .appender(appender)
        .build(Root::builder().appender(destination).build(level))?;
    Ok(config)
}
