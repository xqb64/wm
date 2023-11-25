use crate::{actions::toggle_namedscratchpad, MOD_KEY};
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
    extensions::hooks::ToggleNamedScratchPad,
    map,
};
use tracing_subscriber::{reload::Handle, EnvFilter};

pub fn raw_key_bindings<L, S>(
    handle: Handle<L, S>,
    toggle_scratch: ToggleNamedScratchPad,
) -> HashMap<String, KeyHandler>
where
    L: From<EnvFilter> + 'static,
    S: 'static,
{
    let mut raw_bindings = map! {
        map_keys: |k: &str| k.to_string();

        &format!("{MOD_KEY}-Return") => modify_with(|cs| cs.swap_down()),
        &format!("{MOD_KEY}-S-Return") => modify_with(|cs| cs.swap_up()),
        &format!("{MOD_KEY}-S-c") => modify_with(|cs| cs.kill_focused()),
        &format!("{MOD_KEY}-Tab") => modify_with(|cs| cs.focus_down()),
        &format!("{MOD_KEY}-S-Tab") => modify_with(|cs| cs.focus_up()),
        &format!("{MOD_KEY}-w") => modify_with(|cs| cs.focus_screen(0)),
        &format!("{MOD_KEY}-e") => modify_with(|cs| cs.focus_screen(1)),
        &format!("{MOD_KEY}-S-w") => modify_with(|cs| cs.move_focused_to_screen(0)),
        &format!("{MOD_KEY}-S-e") => modify_with(|cs| cs.move_focused_to_screen(1)),
        &format!("{MOD_KEY}-space") => modify_with(|cs| cs.next_layout()),
        &format!("{MOD_KEY}-S-space") => modify_with(|cs| cs.previous_layout()),
        &format!("{MOD_KEY}-comma") => send_layout_message(|| IncMain(1)),
        &format!("{MOD_KEY}-period") => send_layout_message(|| IncMain(-1)),
        &format!("{MOD_KEY}-h") => send_layout_message(|| ShrinkMain),
        &format!("{MOD_KEY}-l") => send_layout_message(|| ExpandMain),
        &format!("{MOD_KEY}-S-Return") => spawn("tabbed alacritty --embed"),

        // Spawners
        &format!("{MOD_KEY}-F2") => spawn("thunar"),
        &format!("{MOD_KEY}-F3") => spawn("firefox"),
        &format!("{MOD_KEY}-F4") => spawn("code"),
        &format!("{MOD_KEY}-p") => spawn_dmenu(),
        &format!("{MOD_KEY}-Escape") => power_menu(),

        // Some more controls
        &format!("{MOD_KEY}-C-space") => spawn("playerctl play-pause"),
        &format!("{MOD_KEY}-C-Left")=> spawn("playerctl previous"),
        &format!("{MOD_KEY}-C-Right") => spawn("playerctl next"),
        "C-KP_Add" => spawn("amixer -D pulse sset Master 5%+"),
        "C-KP_Subtract" => spawn("amixer -D pulse sset Master 5%-"),

        // Debugging
        &format!("{MOD_KEY}-M-t") => set_tracing_filter(handle),
        &format!("{MOD_KEY}-M-d") => log_current_state(),

        // Scratchpads
        &format!("{MOD_KEY}-slash") => toggle_namedscratchpad("term".to_string(), toggle_scratch),

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

    for (tag, key) in &[("10", "0"), ("11", "minus"), ("12", "plus")] {
        raw_bindings.extend([
            (format!("A-{key}"), show_workspace(tag)),
            (
                format!("A-S-{key}"),
                modify_with(move |client_set| client_set.move_focused_to_tag(tag)),
            ),
        ]);
    }

    raw_bindings
}
