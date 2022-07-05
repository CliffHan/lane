use std::{ffi::OsStr, process::Command, str::from_utf8};
use crate::error::*;

pub fn exec<S, I, S2>(cmd: S, args: I) -> bool
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = S2>,
    S2: AsRef<OsStr>,
{
    if let Ok(status) = Command::new(cmd).args(args).status() {
        return status.success();
    }
    false
}

pub fn exec2<S, I, S2>(cmd: S, args: I) -> Result<String, LaneError>
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = S2>,
    S2: AsRef<OsStr>,
{
    let output = Command::new(cmd).args(args).output().map_err(make_command_failed_error)?;
    // if output.status.success() {
    //     return Err(LaneError::CommandFailed());
    // }
    from_utf8(&output.stdout).map(|s| s.to_string()).map_err(make_command_failed_error)
}
