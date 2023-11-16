//! My personal penrose config
use penrose::{
    builtin::hooks::SpacingHook,
    core::{bindings::parse_keybindings_with_xmodmap, Config, WindowManager},
    extensions::hooks::{
        add_ewmh_hooks,
        manage::{FloatingCentered, SetWorkspace},
        SpawnOnStartup,
    },
    manage_hooks,
    x::query::ClassName,
    x11rb::RustConn,
    Result,
};
use std::{collections::HashMap, process::Stdio};
use tracing_subscriber::{self, prelude::*};
use wm::actions::{add_fixed_workspaces_state, add_xmobar_handle};
use wm::bindings::raw_key_bindings;
use wm::layouts::layouts;

pub const OUTER_PX: u32 = 5;
pub const INNER_PX: u32 = 5;
pub const BAR_HEIGHT_PX: u32 = 30;

fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("/home/alex/wmlogs", "log.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let tracing_builder = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(non_blocking)
        .with_filter_reloading();

    let reload_handle = tracing_builder.reload_handle();
    tracing_builder.finish().init();

    let conn = RustConn::new()?;
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings(reload_handle))?;

    let startup_hook = SpawnOnStartup::boxed("/usr/bin/.wmwrc");
    let manage_hook = manage_hooks![
        ClassName("floatTerm") => FloatingCentered::new(0.8, 0.6),
        ClassName("discord")  => SetWorkspace("9"),
    ];
    let layout_hook = SpacingHook {
        inner_px: INNER_PX,
        outer_px: OUTER_PX,
        top_px: BAR_HEIGHT_PX,
        bottom_px: 0,
    };

    use std::process::Command;

    Command::new("xmobar")
        .args(["/home/alex/.config/xmobar/xmobarrc_0", "-x", "0"])
        .spawn()?;

    let mut xmobar_right = Command::new("xmobar")
        .args(["/home/alex/.config/xmobar/xmobarrc_1", "-x", "1"])
        .stdin(Stdio::piped())
        .spawn()?;

    let xmobar_handle = xmobar_right.stdin.take().unwrap();

    let config = add_ewmh_hooks(Config {
        default_layouts: layouts(),
        floating_classes: vec!["mpv-float".to_owned()],
        manage_hook: Some(manage_hook),
        startup_hook: Some(startup_hook),
        layout_hook: Some(Box::new(layout_hook)),
        ..Config::default()
    });

    let wm = add_xmobar_handle(
        add_fixed_workspaces_state(WindowManager::new(
            config,
            key_bindings,
            HashMap::new(),
            conn,
        )?),
        xmobar_handle,
    );

    wm.run()
}
