use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use crate::apphistory::AppHistory;
use crate::models::{PerformanceSnapshot, ProcessInfo, TabId};

pub struct AppState {
    pub processes: RwLock<Vec<ProcessInfo>>,
    pub performance: RwLock<Option<PerformanceSnapshot>>,
    pub history: Mutex<AppHistory>,
    pub active_tab: RwLock<TabId>,
    pub force_services: AtomicBool,
    pub elevated: bool,
    pub cores: u32,
}

impl AppState {
    pub fn new(cores: u32) -> Arc<Self> {
        Arc::new(AppState {
            processes: RwLock::new(Vec::new()),
            performance: RwLock::new(None),
            history: Mutex::new(AppHistory::new(cores)),
            active_tab: RwLock::new(TabId::Processes),
            force_services: AtomicBool::new(false),
            elevated: crate::util::is_elevated(),
            cores,
        })
    }
}
