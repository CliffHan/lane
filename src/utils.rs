use std::{ffi::OsStr, process::Command};

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
