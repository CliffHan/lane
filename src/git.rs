use crate::error::*;
use crate::utils::*;
use git_config::parser::Key;
use std::{fmt::Display, str::from_utf8};

const GIT_CONFIG: &str = ".gitconfig";
const SECTION_HTTP: &str = "http";
const KEY_PROXY: &str = "proxy";

//TODO: config for git protocol needs socks5 proxy, and should be setup in .ssh/config

pub struct Proxy {
    subsection: Option<String>,
    proxy: String,
}

impl Display for Proxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.subsection {
            Some(sub) => write!(f, "\"{}\": {}", sub, &self.proxy),
            None => write!(f, "{}", &self.proxy),
        }
    }
}

pub fn get_proxies() -> Result<Vec<Proxy>, LaneError> {
    let mut result: Vec<Proxy> = vec![];
    let home_dir = dirs::home_dir().ok_or(LaneError::NoHomeDir())?;
    let git_config_file = home_dir.join(GIT_CONFIG);
    let file = git_config::File::open(&git_config_file)
        .map_err(|_| make_invalid_file_error(&git_config_file))?;
    // TODO: git/ssh protocol not implemented yet
    let sections = file.sections_by_name_with_header(SECTION_HTTP);
    for (header, body) in sections {
        let proxy = body.value(&Key::from(KEY_PROXY));
        if let Some(value) = proxy {
            result.push(Proxy {
                subsection: header.subsection_name.as_ref().map(|s| s.to_string()),
                proxy: from_utf8(&value)
                    .map_err(|_| make_failure_error("Invalid git config file content"))?
                    .to_owned(),
            })
        }
    }
    Ok(result)
}

pub fn set_proxy(proxy: &str) -> Result<(), LaneError> {
    validate_proxy_url(proxy)?;
    exec("git", ["config", "--global", "http.proxy", proxy])
        .then(|| ())
        .ok_or_else(|| make_failure_error("Failed to execute 'git config' command"))
}

pub fn unset_proxy() -> Result<(), LaneError> {
    exec("git", ["config", "--global", "--unset", "http.proxy"])
        .then(|| ())
        .ok_or_else(|| make_failure_error("Failed to execute 'git config' command"))
}
