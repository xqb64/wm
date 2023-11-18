use std::collections::HashMap;

use crate::{
    actions::{power_menu, set_tracing_filter, show_workspace, spawn_dmenu},
    KeyHandler,
};
use penrose::{
    builtin::{
        actions::{log_current_state, modify_with, send_layout_message, spawn},
        layout::messages::{ExpandMain, IncMain, ShrinkMain},
    },
    map,
};
use tracing_subscriber::{reload::Handle, EnvFilter};

pub fn raw_key_bindings<L, S>(handle: Handle<L, S>) -> HashMap<String, KeyHandler>
where
    L: From<EnvFilter> + 'static,
    S: 'static,
{
    let mut raw_bindings = map! {
        map_keys: |k: &str| k.to_string();

        "A-Return" => modify_with(|cs| cs.swap_down()),
        "A-S-Return" => modify_with(|cs| cs.swap_up()),
        "A-S-c" => modify_with(|cs| cs.kill_focused()),
        "A-Tab" => modify_with(|cs| cs.focus_down()),
        "A-S-Tab" => modify_with(|cs| cs.focus_up()),
        "A-w" => modify_with(|cs| cs.focus_screen(0)),
        "A-e" => modify_with(|cs| cs.focus_screen(1)),
        "A-S-w" => modify_with(|cs| cs.move_focused_to_screen(0)),
        "A-S-e" => modify_with(|cs| cs.move_focused_to_screen(1)),
        "A-space" => modify_with(|cs| cs.next_layout()),
        "A-S-space" => modify_with(|cs| cs.previous_layout()),
        "A-S-Up" => send_layout_message(|| IncMain(1)),
        "A-S-Down" => send_layout_message(|| IncMain(-1)),
        "A-S-Left" => send_layout_message(|| ShrinkMain),
        "A-S-Right" => send_layout_message(|| ExpandMain),
        "A-p" => spawn_dmenu(),
        "A-S-Return" => spawn("tabbed alacritty --embed"),
        "A-Escape" => power_menu(),

        // Spawners
        "F2" => spawn("thunar"),
        "F3" => spawn("firefox"),
        "F5" => spawn("code"),

        // Debugging
        "M-A-t" => set_tracing_filter(handle),
        "M-A-d" => log_current_state(),

    };

    for tag in &["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
        raw_bindings.extend([
            (format!("A-{tag}"), show_workspace(tag)),
            (
                format!("A-S-{tag}"),
                modify_with(move |client_set| client_set.move_focused_to_tag(tag)),
            ),
        ]);
    }

    raw_bindings
}
