use clap::clap_derive::ArgEnum;
use strum::{AsRefStr, EnumMessage, EnumIter, IntoEnumIterator};

use crate::{utils::*, error::{LaneError, make_command_failed_error}};

// Note: registry must end with slash('/') because "npm config set registry" command will add one
const DEFAULT_REGISTRY: &str = "https://registry.npmjs.org/";
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum, AsRefStr, EnumMessage, EnumIter)]
pub enum Mirrors {
    #[strum(message = "https://registry.npmmirror.com/")]
    Taobao,
    #[strum(message = "https://repo.huaweicloud.com/repository/npm/")]
    Huawei,
}

fn from_message(registry: &str) -> Result<Mirrors, LaneError> {
    for mirror in Mirrors::iter() {
        if mirror.get_message().unwrap() == registry {
            return Ok(mirror);
        }
    }
    Err(LaneError::UnknownMirror(registry.to_string()))
}

pub fn get_mirror() -> Result<Option<Mirrors>, LaneError> {
    let registry_string = exec2("npm", ["config", "get", "registry"])?;
    let registry = registry_string.trim();
    if registry == DEFAULT_REGISTRY {
        return Ok(None);
    }
    let mirror = from_message(registry)?;
    Ok(Some(mirror))
}

pub fn set_mirror(mirror: &Mirrors) -> Result<(), LaneError> {
    if !exec("npm", ["config", "set", "registry", mirror.get_message().unwrap()]) {
        return Err(make_command_failed_error("npm set registry"));
    }
    Ok(())
}

pub fn unset_mirror() -> Result<(), LaneError> {
    if !exec("npm", ["config", "delete", "registry"]) {
        return Err(make_command_failed_error("npm delete registry"));
    }
    Ok(())
}
