use crate::args::*;
use crate::cargo;
use crate::curl;
use crate::git;
use std::fmt::Display;

const CLEAR_PROXY: &str = "Clear proxy";
const SET_PROXY: &str = "Set proxy";
const PROMPT_NO_PROXY: &str = "No proxy is set!";
const CLEAR_MIRROR: &str = "Clear mirror";
const SET_MIRROR: &str = "Set mirror";
const PROMPT_NO_MIRROR: &str = "No mirror is set!";

fn print_proxy_option<D>(app: D, proxy_option: &Option<impl Display>)
where
    D: AsRef<str>,
{
    match proxy_option {
        None => println!("{}: {}", app.as_ref(), PROMPT_NO_PROXY),
        Some(proxy) => println!("{}: {}", app.as_ref(), proxy),
    }
}

fn print_proxies_result<D>(app: D, proxies_result: &Result<Vec<impl Display>, impl Display>)
where
    D: AsRef<str>,
{
    match proxies_result {
        Err(e) => println!("{}: Failed to get proxies! Error: {}", app.as_ref(), e),
        Ok(proxies) => match proxies.len() {
            0 => println!("{}: {}", app.as_ref(), PROMPT_NO_PROXY),
            1 => println!("{}: {}", app.as_ref(), proxies[0]),
            _ => {
                println!("{}:", app.as_ref());
                for proxy in proxies {
                    println!("    {}", proxy);
                }
            }
        },
    }
}

fn print_mirror_option<D1, D2>(app: D1, mirror_option: &Option<D2>)
where
    D1: AsRef<str>,
    D2: AsRef<str>,
{
    match mirror_option {
        None => println!("{}: {}", app.as_ref(), PROMPT_NO_MIRROR),
        Some(mirror) => println!("{}: Using mirror: {}", app.as_ref(), mirror.as_ref()),
    }
}

fn print_result<D>(app: D, work: &str, result: &Result<(), impl Display>)
where
    D: AsRef<str>,
{
    match result {
        Ok(_) => println!("{}: {} succeeded!", app.as_ref(), work),
        Err(e) => println!("{}: {} failed! Error: {}", app.as_ref(), work, e),
    }
}

macro_rules! do_work {
    ($args: ident, $app: ident, $func: expr, $printer: ident) => {
        if $args.app == None || $args.app == Some($app) {
            $printer($app, &$func());
        }
    };
    ($args: ident, $app: ident, $func: expr, $work: ident, $printer: ident) => {
        if $args.app == None || $args.app == Some($app) {
            $printer($app, $work, &$func());
        }
    };
    ($args: ident, $app: ident, $func: expr, $param: ident, $work: ident, $printer: ident) => {
        if $args.app == None || $args.app == Some($app) {
            $printer($app, $work, &$func(&$args.$param));
        }
    };
}

fn show_proxy(args: &ProxyableAppArgs) {
    use ProxyableApps::*;
    do_work!(args, Curl, curl::get_proxy, print_proxy_option);
    do_work!(args, Git, git::get_proxies, print_proxies_result);
    do_work!(args, Cargo, cargo::get_proxy, print_proxy_option);
}

fn clear_proxy(args: &ProxyableAppArgs) {
    use ProxyableApps::*;
    do_work!(args, Curl, curl::unset_proxy, CLEAR_PROXY, print_result);
    do_work!(args, Git, git::unset_proxy, CLEAR_PROXY, print_result);
    do_work!(args, Cargo, cargo::unset_proxy, CLEAR_PROXY, print_result);
}

fn set_proxy(args: &SetProxyArgs) {
    use ProxyableApps::*;
    do_work!(args, Curl, curl::set_proxy, proxy, SET_PROXY, print_result);
    do_work!(args, Git, git::set_proxy, proxy, SET_PROXY, print_result);
    do_work!(args, Cargo, cargo::set_proxy, proxy, SET_PROXY, print_result);
}

fn show_mirror(args: &MirrorableAppArgs) {
    use MirrorableApps::*;
    do_work!(args, Cargo, cargo::get_mirror, print_mirror_option);
}

fn clear_mirror(args: &MirrorableAppArgs) {
    use MirrorableApps::*;
    do_work!(args, Cargo, cargo::unset_mirror, CLEAR_MIRROR, print_result);
}

fn set_mirror(app: &MirrorableAppsWithParam) {
    use MirrorableApps::*;
    macro_rules! gen_match {
        ($app: ident, $($mirror:ident, $func:expr), *) => {
            match $app {
                $(MirrorableAppsWithParam::$mirror {mirror} => {
                    print_result($mirror, SET_MIRROR, &$func(mirror));
                }) *
            }
        }
    }
    gen_match!(app, Cargo, cargo::set_mirror);
}

pub fn handle_cli_args(cli: Cli) {
    use Commands::*;
    match cli.command {
        ShowProxy(args) => show_proxy(&args),
        ClearProxy(args) => clear_proxy(&args),
        SetProxy(args) => set_proxy(&args),
        ShowMirror(args) => show_mirror(&args),
        ClearMirror(args) => clear_mirror(&args),
        SetMirror { app } => set_mirror(&app),
    }
}
