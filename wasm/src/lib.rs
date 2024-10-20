use std::{borrow::BorrowMut, cell::RefCell, rc::Rc, sync::{Arc, Mutex}};
use sysinfo::{Components, System};
use tokio::time::{interval, Duration};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// Hold user configuration
///
/// # Members
///
/// * `update_interval: u64` - The interval in which sysmon should fetch data (in milliseconds).
/// * `use_icons: bool` - Whether to use unicode symbols instead of text labels.
#[derive(Clone)]
struct Config {
    update_interval: u64,
    use_icons: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            update_interval: 2000,
            use_icons: false,
        }
    }
}

/// State for the system monitor.
///
/// # Members
///
/// * `config: Config` - User configuration.
/// * `cached: Arc<Mutex<CachedStats>>` - Cached system stats.
struct SysMonState {
    config: Config,
    sys: System,
    cpu: String,
    mem: String,
    temp: String,
}

/// Icons to use when `use_icons` is true.
const ICONS: (&str, &str, &str) = ("", "", "");

impl SysMonState {
    fn new(config: Config) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        SysMonState {
            config,
            sys,
            cpu: String::new(),
            mem: String::new(),
            temp: String::new(),
        }
    }

    pub fn update_cpu_usage(&mut self) {
        self.sys.refresh_cpu_all();
        let cpu = self.sys.global_cpu_usage();
        if self.config.use_icons {
            self.cpu = format!("{} {:.2}%", ICONS.0, cpu);
        } else {
            self.cpu = format!("CPU: {:.2}%", cpu);
        }
    }

    pub fn update_mem_usage(&mut self) {
        self.sys.refresh_memory();
        let total_mem = self.sys.total_memory() as f64 / 1024.0;
        let used_mem = self.sys.used_memory() as f64 / 1024.0;
        if self.config.use_icons {
            self.mem = format!("{} {:.2}/{:.2} GB", ICONS.1, used_mem, total_mem);
        } else {
            self.mem = format!("Mem: {:.2}/{:.2} GB", used_mem, total_mem);
        }
    }

    pub fn update_temp(&mut self) {
        let components = Components::new_with_refreshed_list();
        if let Some(component) = components.get(0) {
            let temp = component.temperature();
            if self.config.use_icons {
                self.temp = format!("{} {:.2}°C", ICONS.2, temp);
            } else {
                self.temp = format!("Temp: {:.2}°C", temp);
            }
        }
    }

    pub fn update_all(&mut self) {
        self.update_cpu_usage();
        self.update_mem_usage();
        self.update_temp();
    }
}

thread_local! {
    static SYSMON_STATE: Rc<RefCell<Option<SysMonState>>> = Rc::new(RefCell::new(None));
}

#[wasm_bindgen()]
pub fn init(update_interval: u64, use_icons: bool) -> Result<(), JsValue> {
    let config = Config {
        update_interval,
        use_icons,
    };

    SYSMON_STATE.with(|sysmon_state| {
        *std::cell::RefCell::<_>::borrow_mut(&sysmon_state) = Some(SysMonState::new(config));
    });
    Ok(())
}

#[wasm_bindgen()]
pub fn get_sys_stats() -> Result<JsValue, JsValue> {
    SYSMON_STATE.with(|sysmon_state| {
        if let Some(ref mut state) = *std::cell::RefCell::<_>::borrow_mut(sysmon_state) {
            state.update_all();
            let result = format!("{} | {} | {}", state.cpu, state.mem, state.temp);
            Ok(JsValue::from_str(&result))
        } else {
            Err(JsValue::from_str("System monitor not initialized"))
        }
    })
}
