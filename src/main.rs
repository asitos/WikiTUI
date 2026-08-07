use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
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
mod layout;
mod parser;
mod theme;
mod ui;

use crate::api::{NetworkCommand, NetworkEvent};
use crate::app::{App, InputMode};

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
            match event::read()? {
                Event::Key(key) if key.kind == event::KeyEventKind::Press => match app.input_mode {
                    InputMode::LocalSearch => match key.code {
                        KeyCode::Char(c) => {
                            let pane = app.active_pane_mut();
                            pane.local_search_query.push(c);
                            app.update_local_search();
                        }
                        KeyCode::Backspace => {
                            let pane = app.active_pane_mut();
                            pane.local_search_query.pop();
                            app.update_local_search();
                        }
                        KeyCode::Enter | KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Char(c) => {
                            app.type_search_char(c);
                        }
                        KeyCode::Backspace => {
                            app.backspace_search_char();
                        }
                        KeyCode::Enter => {
                            app.submit_search();
                        }
                        KeyCode::Esc => {
                            app.exit_search_mode();
                        }
                        _ => {}
                    },
                    InputMode::Normal => {
                        if app.waiting_for_split_cmd {
                            app.waiting_for_split_cmd = false;
                            match key.code {
                                KeyCode::Char('v') => {
                                    app.split_active_pane(layout::SplitDirection::Vertical);
                                }
                                KeyCode::Char('s') => {
                                    app.split_active_pane(layout::SplitDirection::Horizontal);
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Char('q') => {
                                    app.quit();
                                }
                                KeyCode::Char('/') => {
                                    app.enter_local_search_mode();
                                }
                                KeyCode::Char('n') => {
                                    app.next_local_match();
                                }
                                KeyCode::Char('N') => {
                                    app.prev_local_match();
                                }
                                KeyCode::Char('s')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    app.enter_search_mode();
                                }
                                KeyCode::Char('t')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    app.new_tab();
                                }
                                KeyCode::Char('w')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    app.waiting_for_split_cmd = true;
                                }
                                KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::ALT) => {
                                    app.prev_tab();
                                }
                                KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::ALT) => {
                                    app.next_tab();
                                }
                                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::ALT) => {
                                    app.close_active_pane();
                                }
                                KeyCode::Char('h')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    let size = terminal.size()?;
                                    app.navigate_panes('h', size.width, size.height);
                                }
                                KeyCode::Char('l')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    let size = terminal.size()?;
                                    app.navigate_panes('l', size.width, size.height);
                                }
                                KeyCode::Char('j')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    let size = terminal.size()?;
                                    app.navigate_panes('j', size.width, size.height);
                                }
                                KeyCode::Char('k')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    let size = terminal.size()?;
                                    app.navigate_panes('k', size.width, size.height);
                                }
                                KeyCode::Tab => {
                                    app.focus_next_link();
                                }
                                KeyCode::BackTab => {
                                    app.focus_prev_link();
                                }
                                KeyCode::Char('j') => {
                                    app.select_next_item();
                                }
                                KeyCode::Char('k') => {
                                    app.select_prev_item();
                                }
                                KeyCode::Char('t') => {
                                    app.activate_selected_in_new_tab();
                                }
                                KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
                                    app.activate_selected_in_new_tab();
                                }
                                KeyCode::Enter => {
                                    app.activate_selected();
                                }
                                _ => {}
                            }
                        }
                    }
                },
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
