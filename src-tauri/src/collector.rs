use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::models::TabId;
use crate::performance::PerformanceCollector;
use crate::processes::ProcessCollector;
use crate::services;
use crate::state::AppState;
use crate::users;

pub fn spawn(app: AppHandle, state: Arc<AppState>) {
    std::thread::spawn(move || {
        let mut proc_collector = ProcessCollector::new();
        let perf = PerformanceCollector::new().ok();
        let mut svc_cache = services::ServiceCache::new();
        let mut last = Instant::now();
        let mut tick: u32 = 0;

        loop {
            std::thread::sleep(Duration::from_millis(1000));
            let now = Instant::now();
            let dt = now.duration_since(last).as_secs_f64();
            last = now;
            tick = tick.wrapping_add(1);

            let procs = proc_collector.collect();

            if let Some(perf) = &perf {
                let snap = perf.snapshot();
                *state.performance.write() = Some(snap.clone());
                let _ = app.emit("tm://performance", snap);
            }

            {
                let mut hist = state.history.lock();
                hist.update(&procs, dt);
                if tick % 30 == 0 {
                    hist.save();
                }
            }

            let active = *state.active_tab.read();

            *state.processes.write() = procs.clone();
            let _ = app.emit("tm://processes", procs.clone());

            let force = state
                .force_services
                .swap(false, std::sync::atomic::Ordering::Relaxed);
            if force || (matches!(active, TabId::Services) && tick % 2 == 0) {
                let svcs = svc_cache.enumerate(force);
                let _ = app.emit("tm://services", svcs);
            }

            if matches!(active, TabId::Users | TabId::Details) || tick % 10 == 0 {
                let sessions = users::collect_user_sessions(procs.clone());
                let _ = app.emit("tm://users", sessions);
            }

            if matches!(active, TabId::AppHistory) || tick % 10 == 0 {
                let hist = state.history.lock().snapshot();
                let _ = app.emit("tm://appHistory", hist);
            }
        }
    });
}
