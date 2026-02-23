mod cli;
mod detection;
mod embedded;
mod hook;
mod patcher;
mod settings;

use clap::Parser;

#[derive(Parser)]
#[command(name = "uprooted", about = "Uprooted installer for Root Communications")]
struct Cli {
    /// Uninstall Uprooted (remove env vars, restore HTML, delete files)
    #[arg(long)]
    uninstall: bool,

    /// Repair installation (re-deploy files, re-patch HTML)
    #[arg(long)]
    repair: bool,

    /// Run diagnostics (check files, env vars, patches)
    #[arg(long)]
    diagnose: bool,

    /// Plain ANSI output instead of TUI (for scripts / CI)
    #[arg(long)]
    plain: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InstallerMode {
    Install,
    Uninstall,
    Repair,
}

fn main() {
    let args = Cli::parse();

    if args.diagnose {
        cli::run_diagnose();
        return;
    }

    let mode = if args.uninstall {
        InstallerMode::Uninstall
    } else if args.repair {
        InstallerMode::Repair
    } else if args.plain {
        InstallerMode::Install
    } else {
        tui::run_mode_selector()
    };

    match (mode, args.plain) {
        (InstallerMode::Install, true) => cli::run_install_plain(),
        (InstallerMode::Install, false) => tui::run_install(),
        (InstallerMode::Uninstall, true) => cli::run_uninstall_plain(),
        (InstallerMode::Uninstall, false) => tui::run_uninstall(),
        (InstallerMode::Repair, true) => cli::run_repair_plain(),
        (InstallerMode::Repair, false) => tui::run_repair(),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// TUI module — btop-inspired console UI with ratatui
// ══════════════════════════════════════════════════════════════════════════════

mod tui {
    use crate::{detection, hook, patcher};
    use crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend,
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
        Frame, Terminal,
    };
    use std::io;
    use std::time::{Duration, Instant};

    #[derive(Clone, PartialEq)]
    enum StepStatus {
        Pending,
        Running,
        Done,
        Failed(String),
    }

    struct Step {
        label: String,
        status: StepStatus,
    }

    impl Step {
        fn new(label: &str) -> Self {
            Self {
                label: label.to_string(),
                status: StepStatus::Pending,
            }
        }
    }

    struct AppState {
        steps: Vec<Step>,
        title: &'static str,
        finished: bool,
        success: bool,
        message: String,
        spinner_tick: usize,
    }

    const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    fn render(frame: &mut Frame, state: &AppState) {
        let area = frame.area();

        // Center a box in the terminal
        let box_width = 60u16.min(area.width);
        let box_height = (state.steps.len() as u16 + 8).min(area.height);
        let x = (area.width.saturating_sub(box_width)) / 2;
        let y = (area.height.saturating_sub(box_height)) / 2;
        let centered = Rect::new(x, y, box_width, box_height);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                format!(" {} ", state.title),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(centered);
        frame.render_widget(block, centered);

        // Build content lines
        let mut lines: Vec<Line> = Vec::new();

        // Version line
        lines.push(Line::from(Span::styled(
            format!("  Uprooted v{}", env!("CARGO_PKG_VERSION")),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Steps
        for step in &state.steps {
            let (icon, style) = match &step.status {
                StepStatus::Pending => (
                    "  ○".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                StepStatus::Running => (
                    format!("  {}", SPINNER[state.spinner_tick % SPINNER.len()]),
                    Style::default().fg(Color::Yellow),
                ),
                StepStatus::Done => (
                    "  ✓".to_string(),
                    Style::default().fg(Color::Green),
                ),
                StepStatus::Failed(_) => (
                    "  ✗".to_string(),
                    Style::default().fg(Color::Red),
                ),
            };
            lines.push(Line::from(vec![
                Span::styled(icon, style),
                Span::raw(" "),
                Span::styled(step.label.clone(), style),
            ]));

            // Show error detail on failure
            if let StepStatus::Failed(msg) = &step.status {
                lines.push(Line::from(Span::styled(
                    format!("      {}", msg),
                    Style::default().fg(Color::Red),
                )));
            }
        }

        // Footer
        lines.push(Line::from(""));
        if state.finished {
            if state.success {
                lines.push(Line::from(Span::styled(
                    format!("  {}", state.message),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("  {}", state.message),
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )));
            }
            lines.push(Line::from(Span::styled(
                "  Press any key to exit...",
                Style::default().fg(Color::DarkGray),
            )));
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    fn run_tui(mut state: AppState, execute_steps: impl FnOnce(&mut AppState)) {
        // Setup terminal
        let _ = enable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, EnterAlternateScreen);
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(_) => {
                let _ = disable_raw_mode();
                eprintln!("Failed to initialize terminal UI");
                return;
            }
        };

        // Initial render
        let _ = terminal.draw(|f| render(f, &state));

        // Execute steps (blocking)
        execute_steps(&mut state);

        // Final render
        let _ = terminal.draw(|f| render(f, &state));

        // Wait for keypress or auto-close on success after 3s
        let deadline = if state.success {
            Some(Instant::now() + Duration::from_secs(3))
        } else {
            None
        };

        loop {
            let timeout = match deadline {
                Some(d) => d.saturating_duration_since(Instant::now()),
                None => Duration::from_secs(60),
            };

            if timeout.is_zero() {
                break;
            }

            if event::poll(timeout.min(Duration::from_millis(100))).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        break;
                    }
                }
            }

            // Update spinner for any running steps
            state.spinner_tick += 1;
            let _ = terminal.draw(|f| render(f, &state));
        }

        // Restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    }

    pub fn run_install() {
        let state = AppState {
            steps: vec![
                Step::new("Check for running Root process"),
                Step::new("Detect Root installation"),
                Step::new("Deploy hook files"),
                Step::new("Set environment variables"),
                Step::new("Patch HTML files"),
                Step::new("Verify installation"),
            ],
            title: "Install",
            finished: false,
            success: false,
            message: String::new(),
            spinner_tick: 0,
        };

        run_tui(state, |state| {
            // Step 0: Check for running Root process
            state.steps[0].status = StepStatus::Running;
            if hook::check_root_running() {
                let killed = hook::kill_root_processes();
                state.steps[0].label = format!("Closed Root ({} process{})", killed, if killed == 1 { "" } else { "es" });
            } else {
                state.steps[0].label = "Root is not running".to_string();
            }
            state.steps[0].status = StepStatus::Done;

            // Step 1: Detect
            state.steps[1].status = StepStatus::Running;
            let detection = detection::detect();
            if detection.root_found {
                state.steps[1].status = StepStatus::Done;
            } else {
                state.steps[1].status =
                    StepStatus::Failed(format!("Root not found at {}", detection.root_path));
                state.finished = true;
                state.message = "Installation failed.".to_string();
                return;
            }

            // Step 2: Deploy files
            state.steps[2].status = StepStatus::Running;
            match hook::deploy_files() {
                Ok(()) => state.steps[2].status = StepStatus::Done,
                Err(e) => {
                    state.steps[2].status = StepStatus::Failed(e);
                    state.finished = true;
                    state.message = "Installation failed.".to_string();
                    return;
                }
            }

            // Step 3: Set env vars
            state.steps[3].status = StepStatus::Running;
            match hook::set_env_vars() {
                Ok(()) => state.steps[3].status = StepStatus::Done,
                Err(e) => {
                    state.steps[3].status = StepStatus::Failed(e);
                    state.finished = true;
                    state.message = "Installation failed.".to_string();
                    return;
                }
            }

            // Step 4: Patch HTML
            state.steps[4].status = StepStatus::Running;
            let result = patcher::install();
            if result.success {
                state.steps[4].status = StepStatus::Done;
            } else {
                state.steps[4].status = StepStatus::Failed(result.message);
                state.finished = true;
                state.message = "Installation failed.".to_string();
                return;
            }

            // Step 5: Verify
            state.steps[5].status = StepStatus::Running;
            let final_check = detection::detect();
            if final_check.hook_status.files_ok && final_check.is_installed {
                state.steps[5].status = StepStatus::Done;
            } else {
                state.steps[5].status = StepStatus::Failed("Verification found issues".to_string());
                state.finished = true;
                state.message = "Installed with warnings.".to_string();
                state.success = true;
                return;
            }

            state.finished = true;
            state.success = true;
            state.message = "Patch active — restart Root to load Uprooted.".to_string();
        });
    }

    pub fn run_uninstall() {
        let state = AppState {
            steps: vec![
                Step::new("Check for running Root process"),
                Step::new("Remove environment variables"),
                Step::new("Restore HTML files"),
                Step::new("Remove settings files"),
                Step::new("Remove deployed files"),
            ],
            title: "Uninstall",
            finished: false,
            success: false,
            message: String::new(),
            spinner_tick: 0,
        };

        run_tui(state, |state| {
            // Step 0: Check for running Root process
            state.steps[0].status = StepStatus::Running;
            if hook::check_root_running() {
                let killed = hook::kill_root_processes();
                state.steps[0].label = format!("Closed Root ({} process{})", killed, if killed == 1 { "" } else { "es" });
            } else {
                state.steps[0].label = "Root is not running".to_string();
            }
            state.steps[0].status = StepStatus::Done;

            // Step 1: Remove env vars
            state.steps[1].status = StepStatus::Running;
            match hook::remove_env_vars() {
                Ok(()) => state.steps[1].status = StepStatus::Done,
                Err(e) => {
                    state.steps[1].status = StepStatus::Failed(e);
                    state.finished = true;
                    state.message = "Uninstall failed.".to_string();
                    return;
                }
            }

            // Step 2: Restore HTML
            state.steps[2].status = StepStatus::Running;
            let result = patcher::uninstall();
            if result.success {
                state.steps[2].status = StepStatus::Done;
            } else {
                state.steps[2].status = StepStatus::Failed(result.message);
            }

            // Step 3: Remove settings
            state.steps[3].status = StepStatus::Running;
            match hook::reset_settings() {
                Ok(n) => {
                    state.steps[3].label = format!("Settings removed ({} file{})", n, if n == 1 { "" } else { "s" });
                    state.steps[3].status = StepStatus::Done;
                }
                Err(e) => {
                    state.steps[3].status = StepStatus::Failed(e);
                    // Non-fatal — continue with file removal
                }
            }

            // Step 4: Remove files
            state.steps[4].status = StepStatus::Running;
            match hook::remove_files() {
                Ok(()) => state.steps[4].status = StepStatus::Done,
                Err(e) => {
                    state.steps[4].status = StepStatus::Failed(e);
                    state.finished = true;
                    state.message = "Uninstall had errors.".to_string();
                    return;
                }
            }

            state.finished = true;
            state.success = true;
            state.message = "Uprooted removed.".to_string();
        });
    }

    pub fn run_repair() {
        let state = AppState {
            steps: vec![
                Step::new("Check for running Root process"),
                Step::new("Reset settings (plugins, themes, preferences)"),
                Step::new("Re-deploy hook files"),
                Step::new("Set environment variables"),
                Step::new("Re-patch HTML files"),
                Step::new("Verify installation"),
            ],
            title: "Repair (resets all settings)",
            finished: false,
            success: false,
            message: String::new(),
            spinner_tick: 0,
        };

        run_tui(state, |state| {
            // Step 0: Check for running Root process
            state.steps[0].status = StepStatus::Running;
            if hook::check_root_running() {
                let killed = hook::kill_root_processes();
                state.steps[0].label = format!("Closed Root ({} process{})", killed, if killed == 1 { "" } else { "es" });
            } else {
                state.steps[0].label = "Root is not running".to_string();
            }
            state.steps[0].status = StepStatus::Done;

            // Step 1: Reset settings
            state.steps[1].status = StepStatus::Running;
            match hook::reset_settings() {
                Ok(n) => {
                    state.steps[1].label = format!("Settings reset ({} file{} removed)", n, if n == 1 { "" } else { "s" });
                    state.steps[1].status = StepStatus::Done;
                }
                Err(e) => {
                    state.steps[1].status = StepStatus::Failed(e);
                    state.finished = true;
                    state.message = "Repair failed.".to_string();
                    return;
                }
            }

            // Step 2: Deploy files
            state.steps[2].status = StepStatus::Running;
            match hook::deploy_files() {
                Ok(()) => state.steps[2].status = StepStatus::Done,
                Err(e) => {
                    state.steps[2].status = StepStatus::Failed(e);
                    state.finished = true;
                    state.message = "Repair failed.".to_string();
                    return;
                }
            }

            // Step 3: Set env vars
            state.steps[3].status = StepStatus::Running;
            match hook::set_env_vars() {
                Ok(()) => state.steps[3].status = StepStatus::Done,
                Err(e) => {
                    state.steps[3].status = StepStatus::Failed(e);
                    state.finished = true;
                    state.message = "Repair failed.".to_string();
                    return;
                }
            }

            // Step 4: Repair HTML
            state.steps[4].status = StepStatus::Running;
            let result = patcher::repair();
            if result.success {
                state.steps[4].status = StepStatus::Done;
            } else {
                state.steps[4].status = StepStatus::Failed(result.message);
                state.finished = true;
                state.message = "Repair failed.".to_string();
                return;
            }

            // Step 5: Verify
            state.steps[5].status = StepStatus::Running;
            let final_check = detection::detect();
            if final_check.hook_status.files_ok && final_check.is_installed {
                state.steps[5].status = StepStatus::Done;
            } else {
                state.steps[5].status = StepStatus::Failed("Verification found issues".to_string());
            }

            state.finished = true;
            state.success = true;
            state.message = "Repair complete — restart Root to load Uprooted.".to_string();
        });
    }

    pub fn run_mode_selector() -> crate::InstallerMode {
        const ITEMS: &[(&str, &str, crate::InstallerMode)] = &[
            ("Install", "", crate::InstallerMode::Install),
            ("Uninstall", "", crate::InstallerMode::Uninstall),
            ("Repair", "resets all settings & plugins", crate::InstallerMode::Repair),
        ];

        let _ = enable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, EnterAlternateScreen);
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(_) => {
                let _ = disable_raw_mode();
                return crate::InstallerMode::Install;
            }
        };

        let mut selected: usize = 0;

        loop {
            let _ = terminal.draw(|frame| {
                let area = frame.area();
                let box_width = 50u16.min(area.width);
                let box_height = 10u16.min(area.height);
                let x = (area.width.saturating_sub(box_width)) / 2;
                let y = (area.height.saturating_sub(box_height)) / 2;
                let centered = Rect::new(x, y, box_width, box_height);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(Span::styled(
                        " Uprooted Installer ",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ));

                let inner = block.inner(centered);
                frame.render_widget(block, centered);

                let mut lines: Vec<Line> = Vec::new();
                lines.push(Line::from(Span::styled(
                    format!("  Uprooted v{}", env!("CARGO_PKG_VERSION")),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));

                for (i, (label, desc, _)) in ITEMS.iter().enumerate() {
                    if i == selected {
                        let mut spans = vec![
                            Span::styled("  \u{25b6} ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled(*label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        ];
                        if !desc.is_empty() {
                            spans.push(Span::styled(format!("  ({})", desc), Style::default().fg(Color::Yellow)));
                        }
                        lines.push(Line::from(spans));
                    } else {
                        let mut spans = vec![
                            Span::raw("    "),
                            Span::styled(*label, Style::default().fg(Color::DarkGray)),
                        ];
                        if !desc.is_empty() {
                            spans.push(Span::styled(format!("  ({})", desc), Style::default().fg(Color::DarkGray)));
                        }
                        lines.push(Line::from(spans));
                    }
                }

                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  \u{2191}\u{2193} Navigate   Enter Select   Q Quit",
                    Style::default().fg(Color::DarkGray),
                )));

                frame.render_widget(Paragraph::new(lines), inner);
            });

            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Up => {
                                if selected > 0 { selected -= 1; }
                            }
                            KeyCode::Down => {
                                if selected < ITEMS.len() - 1 { selected += 1; }
                            }
                            KeyCode::Enter => break,
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                                let _ = disable_raw_mode();
                                let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                                std::process::exit(0);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let _ = disable_raw_mode();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        ITEMS[selected].2
    }
}
