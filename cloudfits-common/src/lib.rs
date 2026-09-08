use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Provider pricing catalog
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPrice {
    pub name: String,
    pub kind: ProviderKind,
    /// USD per 1M input tokens
    pub input_per_mtok: f64,
    /// USD per 1M output tokens
    pub output_per_mtok: f64,
    /// Optional hourly rate (for rented GPUs / servers), else None for pure per-token APIs
    pub hourly_rate: Option<f64>,
    /// 0.0 (trivial) .. 1.0 (very hard) — how hard it is to switch to this provider
    pub switching_effort: f64,
    /// 0.0 (trivial) .. 1.0 — how hard it is to set up from scratch
    pub setup_effort: f64,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderKind {
    Cloud,
    Api,
    Local,
    GpuRental,
}

impl ProviderKind {
    pub fn label(&self) -> &'static str {
        match self {
            ProviderKind::Cloud => "cloud",
            ProviderKind::Api => "api",
            ProviderKind::Local => "local",
            ProviderKind::GpuRental => "gpu-rental",
        }
    }
}

pub fn default_providers() -> Vec<ProviderPrice> {
    vec![
        ProviderPrice {
            name: "Ollama".into(),
            kind: ProviderKind::Local,
            input_per_mtok: 0.0,
            output_per_mtok: 0.0,
            hourly_rate: None,
            switching_effort: 0.1,
            setup_effort: 0.3,
            endpoints: vec!["http://localhost:11434".into()],
        },
        ProviderPrice {
            name: "Together AI".into(),
            kind: ProviderKind::Api,
            input_per_mtok: 0.15,
            output_per_mtok: 0.15,
            hourly_rate: None,
            switching_effort: 0.2,
            setup_effort: 0.2,
            endpoints: vec!["https://api.together.xyz/v1".into()],
        },
        ProviderPrice {
            name: "OpenRouter".into(),
            kind: ProviderKind::Api,
            input_per_mtok: 0.25,
            output_per_mtok: 1.25,
            hourly_rate: None,
            switching_effort: 0.15,
            setup_effort: 0.2,
            endpoints: vec!["https://openrouter.ai/api/v1".into()],
        },
        ProviderPrice {
            name: "Groq".into(),
            kind: ProviderKind::Api,
            input_per_mtok: 0.59,
            output_per_mtok: 0.79,
            hourly_rate: None,
            switching_effort: 0.2,
            setup_effort: 0.25,
            endpoints: vec!["https://api.groq.com/openai/v1".into()],
        },
        ProviderPrice {
            name: "Groq-Llama-3.3".into(),
            kind: ProviderKind::Api,
            input_per_mtok: 0.59,
            output_per_mtok: 0.79,
            hourly_rate: None,
            switching_effort: 0.2,
            setup_effort: 0.25,
            endpoints: vec!["https://api.groq.com/openai/v1".into()],
        },
        ProviderPrice {
            name: "Fireworks".into(),
            kind: ProviderKind::Api,
            input_per_mtok: 0.10,
            output_per_mtok: 0.10,
            hourly_rate: None,
            switching_effort: 0.2,
            setup_effort: 0.25,
            endpoints: vec!["https://api.fireworks.ai/inference/v1".into()],
        },
        ProviderPrice {
            name: "AWS Bedrock".into(),
            kind: ProviderKind::Cloud,
            input_per_mtok: 0.80,
            output_per_mtok: 2.40,
            hourly_rate: None,
            switching_effort: 0.6,
            setup_effort: 0.7,
            endpoints: vec!["https://bedrock-runtime.amazonaws.com".into()],
        },
        ProviderPrice {
            name: "Google Vertex".into(),
            kind: ProviderKind::Cloud,
            input_per_mtok: 0.35,
            output_per_mtok: 1.25,
            hourly_rate: None,
            switching_effort: 0.5,
            setup_effort: 0.6,
            endpoints: vec!["https://us-central1-aiplatform.googleapis.com".into()],
        },
        ProviderPrice {
            name: "Azure OpenAI".into(),
            kind: ProviderKind::Cloud,
            input_per_mtok: 0.60,
            output_per_mtok: 1.80,
            hourly_rate: None,
            switching_effort: 0.6,
            setup_effort: 0.7,
            endpoints: vec!["https://api.openai.com".into()],
        },
        ProviderPrice {
            name: "Lambda GPU".into(),
            kind: ProviderKind::GpuRental,
            input_per_mtok: 0.0,
            output_per_mtok: 0.0,
            hourly_rate: Some(0.99),
            switching_effort: 0.4,
            setup_effort: 0.4,
            endpoints: vec!["https://cloud.lambdalabs.com".into()],
        },
        ProviderPrice {
            name: "RunPod".into(),
            kind: ProviderKind::GpuRental,
            input_per_mtok: 0.0,
            output_per_mtok: 0.0,
            hourly_rate: Some(0.69),
            switching_effort: 0.4,
            setup_effort: 0.4,
            endpoints: vec!["https://www.runpod.io".into()],
        },
    ]
}

// ---------------------------------------------------------------------------
// Hardware profile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub cpu_cores: usize,
    pub total_ram_gb: f64,
    pub available_ram_gb: f64,
    pub gpu_vram_gb: Option<f64>,
    pub gpu_name: Option<String>,
    pub has_gpu: bool,
    /// Estimated memory bandwidth in GB/s
    pub memory_bandwidth_gbps: f64,
    /// Peak system draw in watts (drives local electricity cost)
    pub tdp_watts: f64,
}

impl HardwareProfile {
    pub fn detect() -> anyhow::Result<Self> {
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_ram_gb = sys.total_memory() as f64 / 1073741824.0;
        let available_ram_gb = sys.available_memory() as f64 / 1073741824.0;

        let (gpu_vram_gb, gpu_name) = detect_gpu();

        // Fallback bandwidth estimate: unified/GPU ~ total RAM scaled, CPU ~ 25 GB/s
        let memory_bandwidth_gbps = if gpu_vram_gb.is_some() {
            (total_ram_gb * 1.5).clamp(50.0, 800.0)
        } else {
            25.0
        };

        let tdp_watts = match gpu_name.as_deref() {
            Some(name) if name.to_lowercase().contains("a100") => 400.0,
            Some(name) if name.to_lowercase().contains("h100") => 700.0,
            Some(name) if name.to_lowercase().contains("4090") => 450.0,
            Some(name) if name.to_lowercase().contains("3090") => 350.0,
            Some(name) if name.to_lowercase().contains("a10") => 150.0,
            Some(_) => 250.0,
            None => {
                // CPU only
                let cores = num_cpus_rs();
                30.0 + cores as f64 * 5.0
            }
        };

        Ok(HardwareProfile {
            cpu_cores: num_cpus_rs(),
            total_ram_gb,
            available_ram_gb,
            gpu_vram_gb,
            gpu_name,
            has_gpu: gpu_vram_gb.is_some(),
            memory_bandwidth_gbps,
            tdp_watts,
        })
    }
}

fn num_cpus_rs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

fn detect_gpu() -> (Option<f64>, Option<String>) {
    #[cfg(target_os = "macos")]
    {
        // Apple Silicon: unified memory pool usable for GPU
        if std::env::consts::ARCH == "aarch64" {
            let sys = sysinfo::System::new_all();
            let ram_gb = sys.total_memory() as f64 / 1073741824.0;
            return (Some(ram_gb * 0.75), Some("Apple Silicon".to_string()));
        }
    }

    // NVIDIA
    if let Ok(out) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        && out.status.success()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(line) = text.lines().next() {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() == 2
                && let Ok(mb) = parts[1].parse::<f64>()
            {
                return (Some(mb / 1024.0), Some(parts[0].to_string()));
            }
        }
    }

    // AMD ROCm
    if let Ok(out) = std::process::Command::new("rocm-smi")
        .args(["--showmeminfo", "vram"])
        .output()
        && out.status.success()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(idx) = text.find("Used memory") {
            let rest = &text[idx..];
            if let Some(colon) = rest.find(':') {
                let after = rest[colon + 1..].trim();
                if let Some(space) = after.find(' ')
                    && let Ok(gb) = after[..space].parse::<f64>()
                {
                    // "Used memory" may be (total - used); approximate total
                    return (Some(gb * 2.0), Some("AMD GPU".to_string()));
                }
            }
        }
    }

    (None, None)
}

// ---------------------------------------------------------------------------
// Cost model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunScenario {
    /// USD per kWh (default ~ $0.12 US average)
    pub electricity_per_kwh: f64,
    /// Fraction of time the hardware is actively running the workload
    pub duty_cycle: f64,
    /// Hours of continuous inference per day
    pub hours_per_day: f64,
    /// Days per month
    pub days_per_month: f64,
    /// Typical split: fraction of tokens that are input (rest are output)
    pub input_ratio: f64,
}

impl Default for RunScenario {
    fn default() -> Self {
        RunScenario {
            electricity_per_kwh: 0.12,
            duty_cycle: 1.0,
            hours_per_day: 24.0,
            days_per_month: 30.0,
            input_ratio: 0.6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub model: String,
    pub tokens_per_second: f64,
    pub local: LocalCost,
    pub cloud: CloudCost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalCost {
    pub watts_drawn: f64,
    pub energy_cost_per_hour: f64,
    pub hardware_depreciation_per_hour: f64,
    pub cost_per_hour: f64,
    pub cost_per_month: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCost {
    pub provider: String,
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    pub hourly_rate: Option<f64>,
    pub cost_per_hour: f64,
    pub cost_per_month: f64,
}

/// Estimate tokens-per-second from a memory-bandwidth model.
/// `model_params_billions` = e.g. 8 (for 8B), params_bytes that must move per token.
pub fn estimate_tps(
    model_params_billions: f64,
    bandwidth_gbps: f64,
    quant_bytes_per_param: f64,
) -> f64 {
    if model_params_billions <= 0.0 {
        return 0.0;
    }
    let bytes_per_token = model_params_billions * 1e9 * quant_bytes_per_param;
    let tps = bandwidth_gbps * 1e9 / bytes_per_token;
    tps.min(2000.0)
}

pub fn estimate_model_cost(
    model: &str,
    params_billions: f64,
    hw: &HardwareProfile,
    scenario: &RunScenario,
    provider: &ProviderPrice,
) -> CostEstimate {
    // quant ~ 0.55 bytes/param (Q4)
    let tps = estimate_tps(params_billions, hw.memory_bandwidth_gbps, 0.55);

    // Local cost = energy + hardware depreciation
    let watts_drawn = hw.tdp_watts * scenario.duty_cycle;
    let energy_per_hour = (watts_drawn / 1000.0) * scenario.electricity_per_kwh;
    // Depreciation: rough $ of hardware / (3yr lifetime * 8760h/yr)
    let hardware_value = hw.gpu_vram_gb.unwrap_or(8.0) * 3000.0 / 24.0 + 1500.0;
    let hardware_per_hour = hardware_value / (3.0 * 8760.0);
    let local_per_hour = energy_per_hour + hardware_per_hour;

    // Cloud cost = tokens moved through the API
    // tokens/hour = tps * 3600 * duty_cycle
    let tokens_per_hour = tps * 3600.0 * scenario.duty_cycle;
    let input_tokens_m = tokens_per_hour * scenario.input_ratio / 1e6;
    let output_tokens_m = tokens_per_hour * (1.0 - scenario.input_ratio) / 1e6;
    let cloud_per_hour =
        input_tokens_m * provider.input_per_mtok + output_tokens_m * provider.output_per_mtok;
    let cloud_per_hour = if let Some(hr) = provider.hourly_rate {
        (cloud_per_hour).max(hr)
    } else {
        cloud_per_hour
    };

    let hours_per_month = scenario.hours_per_day * scenario.days_per_month;

    CostEstimate {
        model: model.to_string(),
        tokens_per_second: tps,
        local: LocalCost {
            watts_drawn,
            energy_cost_per_hour: energy_per_hour,
            hardware_depreciation_per_hour: hardware_per_hour,
            cost_per_hour: local_per_hour,
            cost_per_month: local_per_hour * hours_per_month,
        },
        cloud: CloudCost {
            provider: provider.name.clone(),
            input_per_mtok: provider.input_per_mtok,
            output_per_mtok: provider.output_per_mtok,
            hourly_rate: provider.hourly_rate,
            cost_per_hour: cloud_per_hour,
            cost_per_month: cloud_per_hour * hours_per_month,
        },
    }
}

/// Percent cost change moving from `from` provider to `to` provider.
pub fn switch_cost_change(
    model: &str,
    params_billions: f64,
    hw: &HardwareProfile,
    scenario: &RunScenario,
    from: &ProviderPrice,
    to: &ProviderPrice,
) -> f64 {
    let a = estimate_model_cost(model, params_billions, hw, scenario, from)
        .cloud
        .cost_per_month;
    let b = estimate_model_cost(model, params_billions, hw, scenario, to)
        .cloud
        .cost_per_month;
    if a == 0.0 {
        return 0.0;
    }
    ((b - a) / a) * 100.0
}

pub fn parse_params_billions(name: &str) -> f64 {
    let lower = name.to_lowercase();
    if let Some(pos) = lower.find('b') {
        let part = &lower[..pos];
        if let Ok(n) = part
            .trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
            .parse::<f64>()
        {
            return n;
        }
    }
    // heuristic from name
    if lower.contains("70b") || lower.contains("72b") {
        70.0
    } else if lower.contains("34b") || lower.contains("32b") {
        34.0
    } else if lower.contains("13b") || lower.contains("14b") {
        13.0
    } else {
        8.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_tps_positive() {
        let hw = HardwareProfile {
            cpu_cores: 8,
            total_ram_gb: 32.0,
            available_ram_gb: 24.0,
            gpu_vram_gb: Some(24.0),
            gpu_name: Some("Test".into()),
            has_gpu: true,
            memory_bandwidth_gbps: 800.0,
            tdp_watts: 350.0,
        };
        let tps = estimate_tps(8.0, hw.memory_bandwidth_gbps, 0.55);
        assert!(tps > 50.0 && tps <= 2000.0);
    }

    #[test]
    fn parse_params() {
        assert_eq!(parse_params_billions("llama-3.1-8b-instruct"), 8.0);
        assert_eq!(parse_params_billions("llama-70b"), 70.0);
    }
}
