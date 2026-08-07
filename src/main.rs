use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, mpsc};

mod api;
mod app;
mod keybinds;
mod layout;
mod parser;
mod theme;
mod ui;

use crate::api::{NetworkCommand, NetworkEvent};
use crate::app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // communication channel setup (between tokio and ui)
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<NetworkCommand>();
    let (ev_tx, mut ev_rx) = mpsc::unbounded_channel::<NetworkEvent>();

    let cmd_rx = Arc::new(Mutex::new(cmd_rx));
    let ev_tx = Arc::new(Mutex::new(ev_tx));

    // tokio background worker thread
    let worker = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();
        rt.block_on(async move {
            api::run_worker(cmd_rx, ev_tx).await;
        });
    });

    // panic hook
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    let mut app = App::new(cmd_tx.clone());
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    // main event loop
    while app.running {
        // drain background network events
        while let Ok(ev) = ev_rx.try_recv() {
            app.handle_network_event(ev);
        }

        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // keybinds
        if event::poll(timeout)? {
            match event::read() {
                Ok(Event::Key(key)) if key.kind == event::KeyEventKind::Press => {
                    let size = terminal.size()?;
                    keybinds::handle_key_event(&mut app, key, size.width, size.height);
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    drop(app);
    drop(cmd_tx);
    let _ = worker.join();

    Ok(())
}
