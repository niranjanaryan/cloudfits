use clap::{Parser, Subcommand};
use cloudfits_common::{
    CostEstimate, HardwareProfile, ProviderKind, ProviderPrice, RunScenario, default_providers,
    estimate_model_cost, parse_params_billions, switch_cost_change,
};

#[derive(Parser)]
#[command(name = "cloudfits")]
#[command(
    about = "Estimate LLM inference costs (per hour & per month) and compare/switch providers"
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Estimate cost of running a model
    Cost {
        /// Model name (e.g. llama-3.1-8b-instruct, llama-70b)
        #[arg(short, long)]
        model: String,
        /// Parameters in billions (auto-parsed from --model if 0)
        #[arg(long, default_value_t = 0.0)]
        params: f64,
        /// Provider to compare against (cloud/api/gpu-rental). Defaults to lowest API cost.
        #[arg(short, long)]
        provider: Option<String>,
        /// Electricity cost in USD/kWh
        #[arg(long, default_value_t = 0.12)]
        electricity: f64,
        /// Hours per day (default 24)
        #[arg(short = 'H', long, default_value_t = 24.0)]
        hours_per_day: f64,
        /// Days per month (default 30)
        #[arg(short, long, default_value_t = 30.0)]
        days: f64,
        /// Output format: table | json
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// List available inference providers and their pricing
    Providers {
        /// Filter by kind: cloud | api | local | gpu-rental
        #[arg(short, long)]
        kind: Option<String>,
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Show cost delta and steps to switch between providers
    Switch {
        /// Model name
        #[arg(short, long)]
        model: String,
        /// Provider to switch FROM
        #[arg(short, long)]
        from: String,
        /// Provider to switch TO
        #[arg(short, long)]
        to: String,
        /// Just show the diff without printing migration commands
        #[arg(short, long)]
        diff: bool,
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Print detected hardware profile (JSON)
    System,
    /// Compare local vs every cloud/API provider for a model
    Compare {
        #[arg(short, long)]
        model: String,
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Serve the web UI + JSON API (default port 8787)
    Serve {
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long, default_value_t = 8787)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Cost {
            model,
            params,
            provider,
            electricity,
            hours_per_day,
            days,
            format,
        } => cmd_cost(
            model,
            params,
            provider,
            electricity,
            hours_per_day,
            days,
            format,
        ),
        Commands::Providers { kind, format } => cmd_providers(kind, format),
        Commands::Switch {
            model,
            from,
            to,
            diff,
            format,
        } => cmd_switch(model, from, to, diff, format),
        Commands::System => cmd_system(),
        Commands::Compare { model, format } => cmd_compare(model, format),
        Commands::Serve { host, port } => cmd_serve(host, port).await,
    }
}

fn scenario(electricity: f64, hours_per_day: f64, days: f64) -> RunScenario {
    RunScenario {
        electricity_per_kwh: electricity,
        duty_cycle: 1.0,
        hours_per_day,
        days_per_month: days,
        input_ratio: 0.6,
    }
}

fn cmd_cost(
    model: String,
    params: f64,
    provider: Option<String>,
    electricity: f64,
    hours_per_day: f64,
    days: f64,
    format: String,
) -> anyhow::Result<()> {
    let hw = HardwareProfile::detect()?;
    let sc = scenario(electricity, hours_per_day, days);
    let params = if params > 0.0 {
        params
    } else {
        parse_params_billions(&model)
    };
    let providers = default_providers();

    // Pick provider: explicit, or cheapest API/GPU provider for comparison
    let selected: &ProviderPrice = if let Some(name) = provider {
        providers
            .iter()
            .find(|p| p.name.to_lowercase() == name.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Unknown provider '{}'", name))?
    } else {
        providers
            .iter()
            .filter(|p| p.kind != ProviderKind::Local)
            .min_by(|a, b| {
                let ea = estimate_model_cost(&model, params, &hw, &sc, a)
                    .cloud
                    .cost_per_month;
                let eb = estimate_model_cost(&model, params, &hw, &sc, b)
                    .cloud
                    .cost_per_month;
                ea.partial_cmp(&eb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap()
    };

    let est = estimate_model_cost(&model, params, &hw, &sc, selected);
    let local = estimate_model_cost(&model, params, &hw, &sc, &providers[0]);

    if format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "model": model,
                "params_billions": params,
                "hardware": &hw,
                "local": &local.local,
                "provider": &selected.name,
                "provider_kind": selected.kind.label(),
                "cloud": &est.cloud,
                "tokens_per_second": est.tokens_per_second,
            }))?
        );
        return Ok(());
    }

    print_table(&model, params, &hw, &est, &local);
    Ok(())
}

fn print_table(
    model: &str,
    params: f64,
    hw: &HardwareProfile,
    est: &CostEstimate,
    _local: &CostEstimate,
) {
    let w = 56usize;
    let line = "═".repeat(w);
    let thin = "─".repeat(w);
    println!("┌{line}┐");
    println!("│{:^w$}│", " cloudfits — Inference Cost Report ");
    println!("├{thin}┤");
    println!("│ Model: {:<50}│", model);
    println!("│ Params: {:<49}│", format!("{:.1}B", params));
    println!("│ Hardware: {:<49}│", display_hw(hw));
    println!(
        "│ Tokens/s: {:<49}│",
        format!("{:.0}", est.tokens_per_second)
    );
    println!("├{thin}┤");
    println!("│ {:^54} │", "LOCAL (self-hosted)".to_string());
    println!(
        "│ Electricity cost/hr: {:<36}│",
        format!("${:.4}", est.local.energy_cost_per_hour)
    );
    println!(
        "│ Depreciation/hr:     {:<37}│",
        format!("${:.4}", est.local.hardware_depreciation_per_hour)
    );
    println!(
        "│ COST PER HOUR:       {:<37}│",
        format!("${:.4}", est.local.cost_per_hour)
    );
    println!(
        "│ COST PER MONTH:      {:<37}│",
        format!("${:.2}", est.local.cost_per_month)
    );
    println!("├{thin}┤");
    println!("│ {:^54} │", format!("CLOUD — {}", est.cloud.provider));
    println!(
        "│ Input $/M tok: {:<44}│",
        format!("${:.4}", est.cloud.input_per_mtok)
    );
    println!(
        "│ Output $/M tok: {:<43}│",
        format!("${:.4}", est.cloud.output_per_mtok)
    );
    if let Some(hr) = est.cloud.hourly_rate {
        println!("│ Hourly rate: {:<46}│", format!("${:.2}/hr", hr));
    }
    println!(
        "│ COST PER HOUR:       {:<37}│",
        format!("${:.4}", est.cloud.cost_per_hour)
    );
    println!(
        "│ COST PER MONTH:      {:<37}│",
        format!("${:.2}", est.cloud.cost_per_month)
    );
    println!("└{line}┘");
    println!();
    println!(
        "Tip: to compare providers, run: cloudfits switch --model {} --from ollama --to together",
        model
    );
}

fn display_hw(hw: &HardwareProfile) -> String {
    match (&hw.gpu_name, hw.gpu_vram_gb) {
        (Some(name), Some(vram)) => format!(
            "{} ({:.0}GB VRAM), {} cores, {:.0}GB RAM",
            name, vram, hw.cpu_cores, hw.total_ram_gb
        ),
        _ => format!(
            "{} cores, {:.0}GB RAM (CPU only)",
            hw.cpu_cores, hw.total_ram_gb
        ),
    }
}

fn cmd_providers(kind: Option<String>, format: String) -> anyhow::Result<()> {
    let providers = default_providers();
    let filtered: Vec<_> = providers
        .into_iter()
        .filter(|p| match &kind {
            Some(k) => p.kind.label() == k.to_lowercase(),
            None => true,
        })
        .collect();

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
        return Ok(());
    }

    let w = 92usize;
    println!("┌{}┐", "─".repeat(w));
    println!(
        "│{:<24}{:<14}{:<18}{:<18}{:<10}{:<10}│",
        " Provider", " Kind", " In $/Mtok", " Out $/Mtok", " $/hr", " Switch"
    );
    println!("├{}┤", "─".repeat(w));
    for p in filtered {
        println!(
            "│{:<24}{:<14}{:<18.4}{:<18.4}{:<10}{:<10.0}│",
            p.name,
            p.kind.label(),
            p.input_per_mtok,
            p.output_per_mtok,
            p.hourly_rate
                .map(|h| format!("{:.2}", h))
                .unwrap_or_else(|| "-".into()),
            p.switching_effort * 100.0
        );
    }
    println!("└{}┘", "─".repeat(w));
    Ok(())
}

fn cmd_switch(
    model: String,
    from: String,
    to: String,
    diff: bool,
    format: String,
) -> anyhow::Result<()> {
    let hw = HardwareProfile::detect()?;
    let sc = RunScenario::default();
    let params = parse_params_billions(&model);
    let providers = default_providers();

    let from_p = find_provider(&providers, &from)?;
    let to_p = find_provider(&providers, &to)?;

    let pct = switch_cost_change(&model, params, &hw, &sc, from_p, to_p);
    let est_from = estimate_model_cost(&model, params, &hw, &sc, from_p);
    let est_to = estimate_model_cost(&model, params, &hw, &sc, to_p);

    if format == "json" || diff {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "model": model,
                "from": from_p,
                "to": to_p,
                "monthly_change_pct": pct,
                "from_cost_per_month": est_from.cloud.cost_per_month,
                "to_cost_per_month": est_to.cloud.cost_per_month,
                "switching_effort": to_p.switching_effort,
                "setup_effort": to_p.setup_effort,
            }))?
        );
    } else {
        let w = 70usize;
        println!("┌{}┐", "═".repeat(w));
        println!(
            "│ Switch inference provider ({} → {}) │",
            from_p.name, to_p.name
        );
        println!("├{}┤", "─".repeat(w));
        println!(
            "│ From $/month:  {:<50}│",
            format!("${:.2}", est_from.cloud.cost_per_month)
        );
        println!(
            "│ To   $/month:  {:<50}│",
            format!("${:.2}", est_to.cloud.cost_per_month)
        );
        println!("│ Change:        {:<50}│", format!("{:+.1}%", pct));
        println!(
            "│ Switch effort: {:<50}│",
            format!("{:.0}%", to_p.switching_effort * 100.0)
        );
        println!(
            "│ Setup effort:  {:<50}│",
            format!("{:.0}%", to_p.setup_effort * 100.0)
        );
        println!("├{}┤", "─".repeat(w));
        println!(
            "│ Endpoint: {:<56}│",
            to_p.endpoints.first().cloned().unwrap_or_default()
        );
        println!("└{}┘", "═".repeat(w));
        println!();
        println!("Migration steps:");
        if let Some(endpoint) = to_p.endpoints.first() {
            println!("  1. Set base URL:  {endpoint}");
            println!("  2. Set API key (from provider dashboard)");
            println!("  3. Update model id to '{}'", model);
            println!(
                "  4. Validate: cloudfits cost --model {} --provider {}",
                model, to_p.name
            );
        }
    }
    Ok(())
}

fn find_provider<'a>(
    providers: &'a [ProviderPrice],
    name: &str,
) -> anyhow::Result<&'a ProviderPrice> {
    let norm = name.to_lowercase();
    providers
        .iter()
        .find(|p| p.name.to_lowercase() == norm)
        .or_else(|| {
            providers.iter().find(|p| {
                p.name.to_lowercase().contains(&norm) || norm.contains(&p.name.to_lowercase())
            })
        })
        .ok_or_else(|| anyhow::anyhow!("Unknown provider '{}'", name))
}

fn cmd_system() -> anyhow::Result<()> {
    let hw = HardwareProfile::detect()?;
    println!("{}", serde_json::to_string_pretty(&hw)?);
    Ok(())
}

async fn cmd_serve(host: String, port: u16) -> anyhow::Result<()> {
    use axum::Json;
    use axum::extract::Query;
    use axum::http::StatusCode;
    use axum::response::{Html, IntoResponse};
    use axum::routing::get;

    #[derive(serde::Deserialize)]
    struct CostQuery {
        model: Option<String>,
        provider: Option<String>,
        #[serde(default = "default_elec")]
        electricity: f64,
        #[serde(default = "default_hours")]
        hours_per_day: f64,
        #[serde(default = "default_days")]
        days: f64,
    }
    fn default_elec() -> f64 {
        0.12
    }
    fn default_hours() -> f64 {
        24.0
    }
    fn default_days() -> f64 {
        30.0
    }

    async fn api_cost(Query(q): Query<CostQuery>) -> impl IntoResponse {
        let hw = match HardwareProfile::detect() {
            Ok(h) => h,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        };
        let model = q
            .model
            .clone()
            .unwrap_or_else(|| "llama-3.1-8b-instruct".into());
        let params = parse_params_billions(&model);
        let sc = scenario(q.electricity, q.hours_per_day, q.days);
        let providers = default_providers();

        let selected: &ProviderPrice = if let Some(name) = &q.provider {
            providers
                .iter()
                .find(|p| {
                    p.name.to_lowercase() == name.to_lowercase()
                        || p.name.to_lowercase().contains(&name.to_lowercase())
                })
                .unwrap_or(&providers[0])
        } else {
            &providers[0]
        };
        let est = estimate_model_cost(&model, params, &hw, &sc, selected);
        let body = serde_json::json!({
            "model": model,
            "params_billions": params,
            "hardware": &hw,
            "local": &est.local,
            "cloud": &est.cloud,
            "tokens_per_second": est.tokens_per_second,
        });
        (StatusCode::OK, Json(body)).into_response()
    }

    // Embed the HTML in the binary so `serve` works anywhere.
    let html = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../cloudfits-web/index.html"
    ));
    let html_owned: String = html.to_string();

    let app = axum::Router::new()
        .route("/", get(move || async move { Html(html_owned.clone()) }))
        .route("/api/v1/cost", get(api_cost))
        .route("/health", get(|| async { "OK" }));

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("cloudfits web UI + API listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn cmd_compare(model: String, format: String) -> anyhow::Result<()> {
    let hw = HardwareProfile::detect()?;
    let sc = RunScenario::default();
    let params = parse_params_billions(&model);
    let providers = default_providers();

    let mut rows: Vec<(String, f64, f64)> = Vec::new();
    for p in &providers {
        if p.kind == ProviderKind::Local {
            let est = estimate_model_cost(&model, params, &hw, &sc, &providers[0]);
            rows.push((
                p.name.clone(),
                est.local.cost_per_hour,
                est.local.cost_per_month,
            ));
        } else {
            let est = estimate_model_cost(&model, params, &hw, &sc, p);
            rows.push((
                p.name.clone(),
                est.cloud.cost_per_hour,
                est.cloud.cost_per_month,
            ));
        }
    }
    rows.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &rows.iter()
                    .map(|(n, h, m)| serde_json::json!({"provider": n, "cost_per_hour": h, "cost_per_month": m}))
                    .collect::<Vec<_>>()
            )?
        );
        return Ok(());
    }

    let w = 56usize;
    println!("┌{}┐", "─".repeat(w));
    println!("│{:<28}{:<15}{:<15}│", " Provider", " $/hour", " $/month");
    println!("├{}┤", "─".repeat(w));
    for (n, h, m) in rows {
        println!("│{:<28}{:<15.4}{:<15.2}│", n, h, m);
    }
    println!("└{}┘", "─".repeat(w));
    Ok(())
}
