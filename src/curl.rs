// curl: https://everything.curl.dev/cmdline/configfile

use crate::error::*;
use dirs::home_dir;
use std::env;
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};
use trace::trace;

trace::init_depth_var!();

const CURL_CONFIG_FILE: &str = ".curlrc";
#[cfg(target_os = "windows")]
const CURL_CONFIG_FILE2: &str = "_curlrc";
const CURL_KEY_PROXY: &str = "proxy";

fn test_file_existence(dir_option: &Option<PathBuf>) -> Option<PathBuf> {
    macro_rules! return_when_file_exists {
        ($dir: ident, $filename: ident) => {
            let file = $dir.join($filename);
            if file.exists() && file.is_file() {
                return Some(file);
            }
        };
    }
    if let Some(dir) = dir_option {
        // when dir exists, join file and test existence
        return_when_file_exists!(dir, CURL_CONFIG_FILE);
        #[cfg(target_os = "windows")]
        return_when_file_exists!(dir, CURL_CONFIG_FILE2);
    }
    None
}

#[cfg(target_os = "windows")]
fn get_config_file_list() -> Vec<PathBuf> {
    //TODO: add windows part?
    vec![]
}

#[cfg(not(target_os = "windows"))]
fn get_config_file_list() -> Vec<PathBuf> {
    vec![
        env::var("CURL_HOME").ok().map(PathBuf::from),
        env::var("XDG_CONFIG_HOME").ok().map(PathBuf::from),
        home_dir(),
    ]
    .iter()
    .filter_map(test_file_existence)
    .collect()
}

#[trace(logging)]
fn parse_line(line: &str) -> Option<(String, String)> {
    // refer to parseconfig() in https://github.com/curl/curl/blob/master/src/tool_parsecfg.c
    // line with # in the first non-blank column is a comment!
    let line = line.trim_start();
    if line.starts_with(&['#', '/', '\r', '\n', '\0'][..]) {
        return None;
    }
    // dashed option starts with '-'
    let dashed_option = line.starts_with('-');
    // for option and dashed option, the definition of separator is different
    let is_separator = |ch: char| ch == ' ' || (!dashed_option && (ch == '=' || ch == ':'));
    // split from first separator to get the option
    let (option, last_part) = line.split_once(is_separator)?;
    // remove separator to get the parameter part
    let (_, last_part) = last_part.split_at(last_part.find(|ch| !is_separator(ch))?);
    // trim spaces in the end
    let last_part = last_part.trim_end();
    // get parameter from last_part
    let param = {
        match last_part.chars().next() {
            // should handle backslash part here, but don't need that in this case
            Some('\"') => last_part.split_terminator('\"').nth(1).unwrap_or_default(),
            _ => last_part.split_whitespace().next().unwrap_or_default(),
        }
    };
    // skipped param check because don't need that either
    Some((option.to_string(), param.to_string()))
}

fn push_line(target: &mut String, line: &str) {
    target.push_str(line);
    target.push('\n');
}

fn push_proxy_line(target: &mut String, proxy: &Option<&str>) {
    if let Some(url) = proxy {
        let new_line = format!("proxy = \"{}\"", *url);
        push_line(target, &new_line);
    }
}

#[trace(logging)]
fn get_proxy_from_config(path: &Path) -> Result<Option<String>, LaneError> {
    let file_content = read_to_string(path).map_err(|_| make_invalid_file_error(path))?;
    for line in file_content.lines() {
        if let Some((option, param)) = parse_line(line) {
            if option.as_str() == CURL_KEY_PROXY {
                return Ok(Some(param));
            }
        }
    }
    Ok(None)
}

#[trace(logging)]
fn update_proxy_to_file(
    path: &Path,
    proxy: &Option<&str>,
    force: bool,
) -> Result<(), anyhow::Error> {
    // when not force, write to file only when file read to string successfully
    // when force, generated a new file when file not found
    let file_content: String = match force {
        false => read_to_string(path)?,
        true => read_to_string(path).ok().unwrap_or_default(),
    };
    let mut updated = false;
    let mut target = String::new();
    for line in file_content.lines() {
        // after updated, just push rest lines into target
        if updated {
            push_line(&mut target, line);
            continue;
        }
        // parse line and update
        match parse_line(line) {
            Some((option, _)) => {
                if option.as_str() != CURL_KEY_PROXY {
                    push_line(&mut target, line);
                    continue;
                }
                // update url or skip this line as url_option implied
                push_proxy_line(&mut target, proxy);
                updated = true;
            }
            _ => push_line(&mut target, line),
        }
    }
    if !updated {
        // not found in lines
        push_proxy_line(&mut target, proxy);
    }
    // write back to file
    write(path, target)?;
    Ok(())
}

pub fn get_proxy() -> Option<String> {
    for file in get_config_file_list() {
        if let Ok(Some(proxy)) = get_proxy_from_config(&file) {
            return Some(proxy);
        }
    }
    None
}

pub fn set_proxy(proxy: &str) -> Result<(), LaneError> {
    validate_proxy_url(proxy)?;
    let proxy_option = Some(proxy);
    for file in get_config_file_list() {
        if update_proxy_to_file(&file, &proxy_option, false).is_ok() {
            return Ok(());
        }
    }
    // When failed to update existed proxy, force create a new config file
    let home_dir = home_dir().ok_or(LaneError::NoHomeDir())?;
    let default_config = home_dir.join(CURL_CONFIG_FILE);
    update_proxy_to_file(&default_config, &proxy_option, true).map_err(make_failure_error)
}

pub fn unset_proxy() -> Result<(), LaneError> {
    for file in get_config_file_list() {
        if update_proxy_to_file(&file, &None, false).is_ok() {
            return Ok(());
        }
    }
    Err(LaneError::NothingToDo())
}
