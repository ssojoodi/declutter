use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, IsTerminal};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use declutter::{
    app::{App, AppState, Command},
    config::Config,
    events::{handle_key_event, Message},
    scanner::Scanner,
    ui::render_ui,
};

fn main() -> Result<()> {
    if !io::stdout().is_terminal() {
        std::process::exit(1);
    }

    // Load configuration
    let config = Config::load()?;

    // Get starting directory
    let start_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(config.clone(), start_dir.clone());

    let result = run_app(&mut terminal, &mut app, Scanner::new(config));

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Some(folder) = result? {
        let shell = std::env::var_os("SHELL")
            .filter(|shell| !shell.is_empty())
            .unwrap_or_else(|| "/bin/sh".into());
        eprintln!(concat!(
            "+-- NESTED SHELL --------------------+\n",
            "| Type exit to return to your shell. |\n",
            "+------------------------------------+",
        ));
        let mut command = std::process::Command::new(&shell);
        if std::path::Path::new(&shell)
            .file_name()
            .is_some_and(|name| name == "bash")
        {
            // Give only this Bash process an rcfile, while still loading the user's prompt.
            command
                .args(["-c", r#"exec "$0" --rcfile /dev/fd/3 -i 3<<<"$1""#])
                .arg(&shell)
                .arg(include_str!("tempshell.bash"));
        } else {
            command.arg("-i");
        }
        return Err(command.current_dir(&folder).exec())
            .with_context(|| format!("Could not open {:?} in {}", shell, folder.display()));
    }
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    scanner: Scanner,
) -> Result<Option<PathBuf>> {
    terminal.draw(|f| render_ui(f, app))?;
    let root = app.current_dir.clone();
    let latest = Arc::new(Mutex::new(root.clone()));
    let worker_folder = Arc::clone(&latest);
    let worker = thread::Builder::new()
        .name("folder-index".into())
        .spawn(move || {
            scanner.cached_scan_with_progress(&root, |path| {
                let mut folder = worker_folder.lock().unwrap();
                folder.clear();
                folder.push(path);
            })
        })?;

    while !worker.is_finished() {
        if event::poll(Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                if matches!(handle_key_event(key), Some(Message::Quit)) {
                    return Ok(None);
                }
            }
        }
        if let AppState::Loading { folder, tick } = &mut app.state {
            folder.clone_from(&latest.lock().unwrap());
            *tick = tick.wrapping_add(1);
        }
        terminal.draw(|f| render_ui(f, app))?;
    }
    let result = worker
        .join()
        .map_err(|_| anyhow::anyhow!("Indexing thread stopped unexpectedly"))?;
    app.update(match result {
        Ok(tree) => Message::ScanComplete(tree),
        Err(error) => Message::ScanError(error.to_string()),
    });
    loop {
        terminal.draw(|f| render_ui(f, app))?;

        match event::read()? {
            Event::Key(key) => {
                if let Some(msg) = handle_key_event(key) {
                    match app.update(msg) {
                        Command::None => {}
                        Command::Quit => return Ok(None),
                        Command::OpenShell(folder) => return Ok(Some(folder)),
                    }
                }
            }
            _ => {}
        }
    }
}
