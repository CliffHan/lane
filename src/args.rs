use clap::{clap_derive::ArgEnum, Args, Parser, Subcommand};
use strum_macros::AsRefStr;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
// #[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    GetProxy(ProxyableAppArgs),
    ClearProxy(ProxyableAppArgs),
    SetProxy(SetProxyArgs),
    GetMirror(MirrorableAppArgs),
    ClearMirror(MirrorableAppArgs),
    SetMirror {
        #[clap(subcommand)]
        app: MirrorableAppsWithParam,
    },
}

#[derive(Debug, Args)]
pub struct ProxyableAppArgs {
    /// Supported apps
    #[clap(value_enum)]
    pub app: Option<ProxyableApps>,
}

#[derive(Debug, Args)]
pub struct MirrorableAppArgs {
    /// Supported apps
    #[clap(value_enum)]
    pub app: Option<MirrorableApps>,
}

#[derive(Debug, Args)]
pub struct SetProxyArgs {
    #[clap(value_enum)]
    pub app: Option<ProxyableApps>,
    #[clap(short, long, value_parser)]
    pub proxy: String,
}

#[derive(AsRefStr, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum ProxyableApps {
    Cargo,
    Curl,
    Git,
}

#[derive(AsRefStr, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum MirrorableApps {
    Cargo,
    Npm,
}

#[derive(Debug, Subcommand)]
pub enum MirrorableAppsWithParam {
    Cargo {
        #[clap(value_enum)]
        mirror: crate::cargo::Mirrors,
    },
    Npm {
        #[clap(value_enum)]
        mirror: crate::npm::Mirrors,
    },
}
