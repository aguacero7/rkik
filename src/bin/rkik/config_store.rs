use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use toml::Value;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Parse(toml::de::Error),
    Invalid(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "filesystem error: {}", err),
            ConfigError::Parse(err) => write!(f, "invalid config file: {}", err),
            ConfigError::Invalid(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        ConfigError::Io(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::Parse(value)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(value: toml::ser::Error) -> Self {
        ConfigError::Invalid(value.to_string())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Defaults {
    pub timeout: Option<f64>,
    pub format: Option<String>,
    pub ipv6_only: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct PresetRecord {
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigData {
    pub defaults: Defaults,
    pub presets: HashMap<String, PresetRecord>,
}

pub struct ConfigStore {
    path: PathBuf,
    pub data: ConfigData,
}

impl ConfigStore {
    pub fn load() -> Result<Self, ConfigError> {
        let path = default_path();
        if !path.exists() {
            return Ok(Self {
                path,
                data: ConfigData::default(),
            });
        }
        let content = fs::read_to_string(&path)?;
        let parsed: Value = content.parse::<Value>()?;
        let data = parse_value(parsed)?;
        Ok(Self { path, data })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut root = Value::Table(Default::default());
        if let Value::Table(table) = &mut root {
            if let Some(defaults_table) = defaults_to_toml(&self.data.defaults) {
                table.insert("defaults".into(), Value::Table(defaults_table));
            }
            if !self.data.presets.is_empty() {
                let mut presets = toml::map::Map::new();
                for (name, preset) in &self.data.presets {
                    let value = Value::Array(
                        preset
                            .args
                            .iter()
                            .map(|s| Value::String(s.clone()))
                            .collect(),
                    );
                    let mut preset_table = toml::map::Map::new();
                    preset_table.insert("args".into(), value);
                    presets.insert(name.clone(), Value::Table(preset_table));
                }
                table.insert("presets".into(), Value::Table(presets));
            }
        }
        let serialized = toml::to_string_pretty(&root)?;
        fs::write(&self.path, serialized)?;
        Ok(())
    }

    pub fn defaults(&self) -> &Defaults {
        &self.data.defaults
    }

    pub fn update_timeout(&mut self, value: Option<f64>) {
        self.data.defaults.timeout = value;
    }

    pub fn update_format(&mut self, value: Option<String>) {
        self.data.defaults.format = value;
    }

    pub fn update_ipv6(&mut self, value: Option<bool>) {
        self.data.defaults.ipv6_only = value;
    }

    pub fn add_preset(&mut self, name: String, args: Vec<String>) {
        self.data.presets.insert(name, PresetRecord { args });
    }

    pub fn remove_preset(&mut self, name: &str) -> bool {
        self.data.presets.remove(name).is_some()
    }

    pub fn presets(&self) -> &HashMap<String, PresetRecord> {
        &self.data.presets
    }

    pub fn preset(&self, name: &str) -> Option<&PresetRecord> {
        self.data.presets.get(name)
    }

    pub fn empty() -> Self {
        Self {
            path: default_path(),
            data: ConfigData::default(),
        }
    }
}

pub fn default_path() -> PathBuf {
    resolve_config_dir().join("config.toml")
}

fn parse_value(root: Value) -> Result<ConfigData, ConfigError> {
    let mut data = ConfigData::default();
    if let Some(defaults) = root.get("defaults").and_then(|val| val.as_table()) {
        if let Some(timeout_value) = defaults.get("timeout") {
            if let Some(timeout) = timeout_value.as_float() {
                data.defaults.timeout = Some(timeout);
            } else if let Some(int_timeout) = timeout_value.as_integer() {
                data.defaults.timeout = Some(int_timeout as f64);
            }
        }
        if let Some(format) = defaults.get("format").and_then(Value::as_str) {
            data.defaults.format = Some(format.to_string());
        }
        if let Some(ipv6) = defaults.get("ipv6_only").and_then(Value::as_bool) {
            data.defaults.ipv6_only = Some(ipv6);
        }
    }
    if let Some(presets) = root.get("presets").and_then(|val| val.as_table()) {
        for (name, entry) in presets {
            if let Some(table) = entry.as_table() {
                if let Some(args) = table.get("args").and_then(Value::as_array) {
                    let parsed_args: Vec<String> = args
                        .iter()
                        .filter_map(Value::as_str)
                        .map(|s| s.to_string())
                        .collect();
                    if !parsed_args.is_empty() {
                        data.presets
                            .insert(name.to_string(), PresetRecord { args: parsed_args });
                    }
                }
            }
        }
    }
    Ok(data)
}

fn defaults_to_toml(defaults: &Defaults) -> Option<toml::map::Map<String, Value>> {
    if defaults.timeout.is_none() && defaults.format.is_none() && defaults.ipv6_only.is_none() {
        return None;
    }
    let mut table = toml::map::Map::new();
    if let Some(timeout) = defaults.timeout {
        table.insert("timeout".into(), Value::Float(timeout));
    }
    if let Some(format) = &defaults.format {
        table.insert("format".into(), Value::String(format.clone()));
    }
    if let Some(ipv6) = defaults.ipv6_only {
        table.insert("ipv6_only".into(), Value::Boolean(ipv6));
    }
    Some(table)
}

fn resolve_config_dir() -> PathBuf {
    if let Some(val) = env::var_os("RKIK_CONFIG_DIR") {
        let path = PathBuf::from(val);
        if path.is_absolute() {
            return path;
        }
        return env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| PathBuf::from("."));
    }
    if let Some(base) = dirs::config_dir() {
        return base.join("rkik");
    }
    PathBuf::from(".rkik")
}
