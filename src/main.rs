//! My personal penrose config
use penrose::{
    builtin::{
        actions::{key_handler, modify_with, send_layout_message, spawn},
        hooks::SpacingHook,
        layout::messages::{ExpandMain, IncMain, ShrinkMain},
    },
    core::{
        bindings::{parse_keybindings_with_xmodmap, KeyEventHandler},
        Config, WindowManager,
    },
    extensions::{
        hooks::{
            add_ewmh_hooks,
            manage::{FloatingCentered, SetWorkspace},
            SpawnOnStartup,
        },
        util::dmenu::{DMenu, DMenuConfig, MenuMatch},
    },
    manage_hooks, map,
    util::spawn as _spawn,
    x::query::ClassName,
    x11rb::RustConn,
};
use std::collections::HashMap;
use tracing_subscriber::{self, prelude::*};
use wm::layouts::layouts;

fn raw_key_bindings() -> HashMap<String, Box<dyn KeyEventHandler<RustConn>>> {
    let mut raw_bindings = map! {
        map_keys: |k: &str| k.to_string();

        "A-Return" => modify_with(|cs| cs.swap_down()),
        "A-S-Return" => modify_with(|cs| cs.swap_up()),
        "A-S-c" => modify_with(|cs| cs.kill_focused()),
        "A-Tab" => modify_with(|cs| cs.focus_down()),
        "A-S-Tab" => modify_with(|cs| cs.focus_up()),
        "A-w" => modify_with(|cs| cs.previous_screen()),
        "A-e" => modify_with(|cs| cs.next_screen()),
        "A-S-w" => modify_with(|cs| cs.move_focused_to_screen(0)),
        "A-S-e" => modify_with(|cs| cs.move_focused_to_screen(1)),
        "A-space" => modify_with(|cs| cs.next_layout()),
        "A-S-space" => modify_with(|cs| cs.previous_layout()),
        "A-S-Up" => send_layout_message(|| IncMain(1)),
        "A-S-Down" => send_layout_message(|| IncMain(-1)),
        "A-S-Right" => send_layout_message(|| ExpandMain),
        "A-S-Left" => send_layout_message(|| ShrinkMain),
        "A-p" => spawn("dmenu_run"),
        "A-S-Return" => spawn("tabbed alacritty --embed"),
        "A-Escape" => power_menu(),
    };

    for tag in &["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
        raw_bindings.extend([
            (
                format!("A-{tag}"),
                modify_with(move |client_set| client_set.focus_tag(tag)),
            ),
            (
                format!("A-S-{tag}"),
                modify_with(move |client_set| client_set.move_focused_to_tag(tag)),
            ),
        ]);
    }

    raw_bindings
}

pub const OUTER_PX: u32 = 5;
pub const INNER_PX: u32 = 5;
pub const BAR_HEIGHT_PX: u32 = 30;
pub type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

pub fn power_menu() -> KeyHandler {
    key_handler(|state, _| {
        let options = vec!["lock", "logout", "restart-wm", "shutdown", "reboot"];
        let screen_index = state.client_set.current_screen().index();
        let menu = DMenu::new(&DMenuConfig::with_prompt(">>> "), screen_index);

        if let Ok(MenuMatch::Line(_, choice)) = menu.build_menu(options) {
            match choice.as_ref() {
                "lock" => penrose::util::spawn("xflock4"),
                "logout" => penrose::util::spawn("pkill -fi wm"),
                "shutdown" => penrose::util::spawn("sudo shutdown -h now"),
                "reboot" => penrose::util::spawn("sudo reboot"),
                "restart-wm" => std::process::exit(0), // Wrapper script then handles restarting us
                _ => unimplemented!(),
            }
        } else {
            Ok(())
        }
    })
}

fn main() -> penrose::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .finish()
        .init();

    let conn = RustConn::new()?;
    let key_bindings = parse_keybindings_with_xmodmap(raw_key_bindings())?;

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

    let config = add_ewmh_hooks(Config {
        default_layouts: layouts(),
        floating_classes: vec!["mpv-float".to_owned()],
        manage_hook: Some(manage_hook),
        startup_hook: Some(startup_hook),
        layout_hook: Some(Box::new(layout_hook)),
        ..Config::default()
    });

    _spawn("polybar left")?;
    _spawn("polybar right")?;

    let wm = WindowManager::new(config, key_bindings, HashMap::new(), conn)?;

    wm.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_parse_correctly_with_xmodmap() {
        let res = parse_keybindings_with_xmodmap(raw_key_bindings());

        if let Err(e) = res {
            panic!("{e}");
        }
    }
}
