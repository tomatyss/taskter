#![allow(clippy::missing_panics_doc)]

use std::ffi::OsString;

use taskter::config::{self, ConfigOverrides};

pub struct EnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    pub fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(prev) = self.previous.as_ref() {
            std::env::set_var(self.key, prev);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

pub fn disable_host_config_guard() -> EnvVarGuard {
    EnvVarGuard::set("TASKTER_DISABLE_HOST_CONFIG", "1")
}

#[allow(dead_code)]
pub fn with_temp_dir<F: FnOnce() -> T, T>(test: F) -> T {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let original_dir = std::env::current_dir().expect("cannot read current dir");
    std::env::set_current_dir(tmp.path()).expect("cannot set current dir");
    let _disable_guard = disable_host_config_guard();

    let data_dir = tmp.path().join(taskter::config::DIR);
    std::fs::create_dir_all(&data_dir).unwrap();

    let config_path = tmp.path().join("config.toml");
    std::fs::write(&config_path, b"").unwrap();

    let overrides = ConfigOverrides {
        config_file: Some(config_path),
        data_dir: Some(data_dir),
        ..ConfigOverrides::default()
    };

    config::init(&overrides).expect("failed to load config for test");

    let result = test();

    std::env::set_current_dir(original_dir).expect("cannot restore current dir");
    config::init(&ConfigOverrides::default()).expect("failed to reset config state");

    result
}
