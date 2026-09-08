use anyhow::Result;
use cloudfits_common::{
    HardwareProfile, ProviderKind, ProviderPrice, RunScenario, default_providers,
    estimate_model_cost, parse_params_billions,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use std::io;

struct App {
    providers: Vec<ProviderPrice>,
    hardware: HardwareProfile,
    scenario: RunScenario,
    exit_requested: bool,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    // If a CLI subcommand is passed, delegate to the CLI binary crate logic is intentionally
    // minimal here; the TUI is the primary experience. For scripting use `cloudfits`.
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" | "help" => {
                println!("Usage: cloudfits-tui  (interactive terminal UI)");
                println!("       cloudfits ...  (scripting/CLI — see that binary)");
                return Ok(());
            }
            _ => {}
        }
    }

    let hardware = HardwareProfile::detect()?;
    let providers = default_providers();
    let app = App {
        providers,
        hardware,
        scenario: RunScenario::default(),
        exit_requested: false,
    };

    run_tui(app)?;
    Ok(())
}

fn run_tui(app: App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    while !app.exit_requested {
        terminal.draw(|f| draw(f, &app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.exit_requested = true,
                _ => {}
            }
        }
    }
    Ok(())
}

fn draw(frame: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(app.providers.len() as u16 + 2),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new(
        Line::from("cloudfits — TUI (q to quit)").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    );
    frame.render_widget(title, chunks[0]);

    // Hardware summary
    let hw_text = format!(
        "{} cores | {:.0}GB RAM | GPU: {} | Bandwidth: {:.0} GB/s | TDP: {:.0}W",
        app.hardware.cpu_cores,
        app.hardware.total_ram_gb,
        app.hardware.gpu_name.as_deref().unwrap_or("none"),
        app.hardware.memory_bandwidth_gbps,
        app.hardware.tdp_watts,
    );
    let hw_block = Block::default().borders(Borders::ALL).title(" Hardware ");
    frame.render_widget(Paragraph::new(hw_text).block(hw_block), chunks[1]);

    // Provider cost comparison for a representative model
    let model = "llama-3.1-8b-instruct";
    let params = parse_params_billions(model);
    let items: Vec<ListItem> = app
        .providers
        .iter()
        .map(|p| {
            let est = if p.kind == ProviderKind::Local {
                let e = estimate_model_cost(
                    model,
                    params,
                    &app.hardware,
                    &app.scenario,
                    &app.providers[0],
                );
                format!(
                    "{:>12}  $/hr {:>8.4}  $/mo {:>10.2}",
                    p.name, e.local.cost_per_hour, e.local.cost_per_month
                )
            } else {
                let e = estimate_model_cost(model, params, &app.hardware, &app.scenario, p);
                format!(
                    "{:>12}  $/hr {:>8.4}  $/mo {:>10.2}  (input ${:.2}/M)",
                    p.name, e.cloud.cost_per_hour, e.cloud.cost_per_month, p.input_per_mtok
                )
            };
            ListItem::new(Line::from(est))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Estimate — {} ", model)),
    );
    frame.render_widget(list, chunks[2]);

    let footer = Paragraph::new(
        Line::from("Use the CLI: `cloudfits compare --model <m>` for full table")
            .style(Style::default().fg(Color::Gray)),
    );
    frame.render_widget(footer, chunks[3]);
}
