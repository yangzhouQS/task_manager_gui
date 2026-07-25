use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::{AppHistoryEntry, ProcessInfo};

#[derive(Serialize, Deserialize, Default, Clone)]
struct HistEntry {
    cpu_sec: f32,
    network_mb: f64,
    metered_mb: f64,
    tile_updates: u32,
    icon: Option<String>,
}

pub struct AppHistory {
    map: HashMap<String, HistEntry>,
    path: PathBuf,
}

impl AppHistory {
    pub fn new(_cores: u32) -> Self {
        let path = history_path();
        let map = load(&path).unwrap_or_default();
        Self { map, path }
    }

    pub fn update(&mut self, processes: &[ProcessInfo], dt: f64) {
        if dt <= 0.0 {
            return;
        }
        for p in processes {
            if p.kind != crate::models::ProcessKind::App || p.pid == 0 {
                continue;
            }
            let name = normalize_name(&p.name);
            let core_seconds = (p.cpu_percent as f64 / 100.0) * dt;
            let entry = self.map.entry(name).or_default();
            entry.cpu_sec += core_seconds as f32;
            entry.icon = p.icon.clone().or(entry.icon.take());
        }
    }

    pub fn snapshot(&self) -> Vec<AppHistoryEntry> {
        let mut out: Vec<AppHistoryEntry> = self
            .map
            .iter()
            .map(|(name, h)| AppHistoryEntry {
                name: name.clone(),
                icon: h.icon.clone(),
                total_cpu_sec: h.cpu_sec,
                network_mb: h.network_mb,
                metered_mb: h.metered_mb,
                tile_updates: h.tile_updates,
            })
            .collect();
        out.sort_by(|a, b| b.total_cpu_sec.partial_cmp(&a.total_cpu_sec).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    pub fn save(&self) {
        let _ = std::fs::write(&self.path, serde_json::to_vec(&self.map).unwrap_or_default());
    }

    pub fn reset(&mut self) {
        self.map.clear();
        let _ = std::fs::remove_file(&self.path);
    }
}

fn normalize_name(name: &str) -> String {
    let lower = name.to_lowercase();
    lower.trim_end_matches(".exe").to_string()
}

fn history_path() -> PathBuf {
    let mut p = dirs_or_appdata();
    p.push("win-task-manager");
    let _ = std::fs::create_dir_all(&p);
    p.push("app_history.json");
    p
}

fn dirs_or_appdata() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn load(path: &PathBuf) -> anyhow::Result<HashMap<String, HistEntry>> {
    let data = std::fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}
