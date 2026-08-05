use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io,
    time::{Duration, Instant},
};

mod app;
mod layout;
mod theme;
mod ui;

use crate::app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // panic hook
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    let mut app = App::new();
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    // event loop
    while app.running {
        terminal.draw(|f| ui::draw(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // keybinds
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == event::KeyEventKind::Press => {
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
                            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.new_tab();
                            }
                            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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
                            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                let size = terminal.size()?;
                                app.navigate_panes('h', size.width, size.height);
                            }
                            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                let size = terminal.size()?;
                                app.navigate_panes('l', size.width, size.height);
                            }
                            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                let size = terminal.size()?;
                                app.navigate_panes('j', size.width, size.height);
                            }
                            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                let size = terminal.size()?;
                                app.navigate_panes('k', size.width, size.height);
                            }
                            _ => {}
                        }
                    }
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

    Ok(())
}
