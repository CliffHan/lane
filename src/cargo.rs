use crate::error::*;
use clap::{clap_derive::ArgEnum, ValueEnum};
use dirs::home_dir;
use std::{
    fs::{read_to_string, write},
    path::{Path, PathBuf},
};
use strum::EnumMessage;
use strum_macros::AsRefStr;
use toml::{
    value::{Map, Table},
    Value,
};
use trace::trace;

trace::init_depth_var!();

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum, AsRefStr, EnumMessage)]
pub enum Mirrors {
    #[strum(message = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git")]
    Tuna,
    #[strum(message = "http://mirrors.ustc.edu.cn/crates.io-index")]
    Ustc,
}

const CARGO_CONFIG: &str = ".cargo/config";

fn read_proxy_from_config(path: &Path) -> Result<Option<String>, LaneError> {
    let content = read_to_string(path).map_err(|_| make_invalid_file_error(path))?;
    let value: Value = content.parse().map_err(|_| make_invalid_file_error(path))?;
    let proxy_value = match value.get("http") {
        None => return Ok(None),
        Some(value) => value.get("proxy"),
    };
    match proxy_value {
        None => Ok(None),
        Some(value) => Ok(value.as_str().map(|v| v.to_string())),
    }
}

fn set_proxy_to_value(value: &mut Value, proxy: &str) -> Result<(), LaneError> {
    // clone existed http section or create a new one
    let mut http_value = match value.get("http") {
        Some(v) => v.clone(),
        None => Value::Table(Map::new()), // create if not exists
    };
    // insert proxy into http section
    http_value
        .as_table_mut()
        .ok_or_else(|| make_failure_error("Failed to get http table."))?
        .insert("proxy".into(), Value::String(proxy.to_string()));
    // insert http section to orginal value
    value
        .as_table_mut()
        .ok_or_else(|| make_failure_error("Failed to insert http section."))?
        .insert("http".into(), http_value);
    Ok(())
}

fn unset_proxy_to_value(value: &mut Value) -> Result<(), LaneError> {
    // remove http section from value, return while http section not exists
    let mut http_value = match value
        .as_table_mut()
        .ok_or_else(|| make_failure_error("Failed to get http section."))?
        .remove("http")
    {
        Some(v) => v,
        None => return Err(LaneError::NothingToDo()), // no http section means no need to unset
    };
    // remove proxy from http section, if nothing left, just return
    match http_value.as_table_mut() {
        Some(http_table) => {
            http_table.remove("proxy");
            if http_table.is_empty() {
                return Ok(()); // nothing left
            }
        }
        None => {
            return Err(make_failure_error("Failed to get http table."));
        }
    }
    // insert http section back
    value
        .as_table_mut()
        .ok_or_else(|| make_failure_error("Failed to replace http section."))?
        .insert("http".into(), http_value);
    Ok(())
}

fn read_mirror_from_config(path: &Path) -> Result<Option<Mirrors>, LaneError> {
    let content = read_to_string(path).map_err(|_| make_invalid_file_error(path))?;
    let value: Value = content.parse().map_err(|_| make_invalid_file_error(path))?;
    let replace_with = match value.get("source").and_then(|v| v.get("crates-io")) {
        None => return Ok(None),
        Some(value) => match value.get("replace-with") {
            Some(v) => v,
            None => return Ok(None),
        },
    };
    match replace_with.as_str() {
        None => Ok(None),
        Some(v) => Ok(Some(Mirrors::from_str(v, true).map_err(|_| make_unknown_mirror_error(v))?)),
    }
}

fn create_string_table_value(key: &str, value: &str) -> Value {
    let mut table = Table::new();
    table.insert(key.into(), Value::String(value.into()));
    Value::Table(table)
}

#[trace(logging)]
fn set_mirror_to_value(value: &mut Value, mirror: &Mirrors) -> Result<(), LaneError> {
    // get source section
    let mut source_value = match value.get("source") {
        Some(v) => v.clone(),
        None => Value::Table(Map::new()), // create if not exists
    };
    let source_table_value =
        source_value.as_table_mut().ok_or_else(|| make_failure_error("Invalid source section."))?;

    // insert mirror section to source and set replace-with value of crates-io to mirror
    let mirror_name = mirror.as_ref().to_lowercase();
    let registry = mirror.get_message().unwrap();
    let replace_with_value = create_string_table_value("replace-with", &mirror_name);
    let registry_value = create_string_table_value("registry", registry);
    source_table_value.insert("crates-io".into(), replace_with_value);
    source_table_value.insert(mirror_name, registry_value);

    // insert source section back
    value
        .as_table_mut()
        .ok_or_else(|| make_failure_error("Failed to insert source section."))?
        .insert("source".into(), source_value);
    Ok(())
}

fn unset_mirror_to_value(value: &mut Value) -> Result<(), LaneError> {
    // get source.crates-io section
    let crates_io_value = match value.get_mut("source").and_then(|v| v.get_mut("crates-io")) {
        None => return Err(LaneError::NothingToDo()),
        Some(v) => v,
    };

    // remove replace-with key value pair
    crates_io_value
        .as_table_mut()
        .ok_or_else(|| make_failure_error("Invalid source.crates-io section."))?
        .remove("replace-with");
    Ok(())
}

fn get_config_file_path() -> Result<PathBuf, LaneError> {
    let home_dir = home_dir().ok_or(LaneError::NoHomeDir())?;
    Ok(home_dir.join(CARGO_CONFIG))
}

pub fn get_proxy() -> Option<String> {
    let config_file = home_dir()?.join(CARGO_CONFIG);
    read_proxy_from_config(&config_file).unwrap_or_default()
}

pub fn set_proxy(proxy: &str) -> Result<(), LaneError> {
    validate_proxy_url(proxy)?;
    let config_file = get_config_file_path()?;
    let file_content: String = read_to_string(&config_file).unwrap_or_default();
    let mut value: Value =
        file_content.parse().map_err(|_| make_invalid_file_error(&config_file))?;
    set_proxy_to_value(&mut value, proxy)?;
    write(&config_file, value.to_string()).map_err(|_| make_write_file_error(&config_file))
}

pub fn unset_proxy() -> Result<(), LaneError> {
    let config_file = get_config_file_path()?;
    let file_content: String = match read_to_string(&config_file) {
        Err(_) => return Err(LaneError::NothingToDo()), // no file means no need to unset
        Ok(content) => content,
    };
    let mut value: Value =
        file_content.parse().map_err(|_| make_invalid_file_error(&config_file))?;
    unset_proxy_to_value(&mut value)?;
    write(&config_file, value.to_string()).map_err(|_| make_write_file_error(&config_file))
}

pub fn get_mirror() -> Result<Option<Mirrors>, LaneError> {
    let config_file = get_config_file_path()?;
    read_mirror_from_config(&config_file)
}

pub fn set_mirror(mirror: &Mirrors) -> Result<(), LaneError> {
    let config_file = get_config_file_path()?;
    let file_content: String = read_to_string(&config_file).unwrap_or_default();
    let mut value: Value =
        file_content.parse().map_err(|_| make_invalid_file_error(&config_file))?;
    set_mirror_to_value(&mut value, mirror)?;
    write(&config_file, value.to_string()).map_err(|_| make_write_file_error(&config_file))
}

pub fn unset_mirror() -> Result<(), LaneError> {
    let config_file = get_config_file_path()?;
    let file_content: String = match read_to_string(&config_file) {
        Err(_) => return Err(LaneError::NothingToDo()), // no file means no need to unset
        Ok(content) => content,
    };
    let mut value: Value =
        file_content.parse().map_err(|_| make_invalid_file_error(&config_file))?;
    unset_mirror_to_value(&mut value)?;
    write(&config_file, value.to_string()).map_err(|_| make_write_file_error(&config_file))
}
