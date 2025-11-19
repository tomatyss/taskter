#![allow(clippy::missing_panics_doc)]

use std::path::PathBuf;

use taskter::config::{self, ConfigOverrides};

mod common;
use common::with_temp_dir;

#[test]
fn layering_respects_flag_env_and_file_order() {
    with_temp_dir(|| {
        std::env::remove_var("TASKTER__PATHS__DATA_DIR");

        let config_path = PathBuf::from("config.toml");
        std::fs::write(&config_path, "[paths]\ndata_dir = \"./from-config\"\n")
            .expect("failed to write config file");

        let mut overrides = ConfigOverrides {
            config_file: Some(config_path.clone()),
            data_dir: Some(PathBuf::from("./from-flags")),
            ..ConfigOverrides::default()
        };

        std::env::set_var("TASKTER__PATHS__DATA_DIR", "./from-env");
        config::init(&overrides).expect("init with overrides");
        assert_eq!(config::dir().expect("dir"), PathBuf::from("./from-flags"));

        overrides.data_dir = None;
        config::init(&overrides).expect("init without flag override");
        assert_eq!(config::dir().expect("dir"), PathBuf::from("./from-env"));

        std::env::remove_var("TASKTER__PATHS__DATA_DIR");
        config::force_reload().expect("reload without env");
        assert_eq!(config::dir().expect("dir"), PathBuf::from("./from-config"));
    });
}
