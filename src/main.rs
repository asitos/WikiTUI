use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

use wikid::api::{self, NetworkCommand, NetworkEvent};
use wikid::app::App;
use wikid::keybinds;
use wikid::ui;

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        PopKeyboardEnhancementFlags
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    );

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        original_hook(panic_info);
    }));

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (cmd_tx, cmd_rx) = mpsc::channel::<NetworkCommand>();
    let (ev_tx, ev_rx) = mpsc::channel::<NetworkEvent>();

    let worker = std::thread::spawn(move || {
        api::run_worker(cmd_rx, ev_tx);
    });

    let mut app = App::new(cmd_tx.clone());
    let run_res = run_app(&mut terminal, &mut app, &ev_rx);

    restore_terminal();
    let _ = terminal.show_cursor();

    drop(app);
    drop(cmd_tx);
    let _ = worker.join();

    run_res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    ev_rx: &mpsc::Receiver<NetworkEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    while app.running {
        app.check_config_sync();

        while let Ok(ev) = ev_rx.try_recv() {
            app.handle_network_event(ev);
        }

        terminal.draw(|f| ui::draw(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == event::KeyEventKind::Press {
                    let size = terminal.size()?;
                    keybinds::handle_key_event(app, key, size.width, size.height);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}
