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
        "A-comma" => send_layout_message(|| IncMain(1)),
        "A-period" => send_layout_message(|| IncMain(-1)),
        "A-h" => send_layout_message(|| ShrinkMain),
        "A-l" => send_layout_message(|| ExpandMain),
        "A-p" => spawn_dmenu(),
        "A-S-Return" => spawn("tabbed alacritty --embed"),
        "A-Escape" => power_menu(),

        // Spawners
        "A-F2" => spawn("thunar"),
        "A-F3" => spawn("firefox"),
        "A-F4" => spawn("code"),

        // Some more controls
        "C-A-space" => spawn("playerctl play-pause"),
        "C-A-Left" => spawn("playerctl previous"),
        "C-A-Right" => spawn("playerctl next"),
        "C-KP_Add" => spawn("amixer -D pulse sset Master 5%+"),
        "C-KP_Subtract" => spawn("amixer -D pulse sset Master 5%-"),


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

#[cfg(test)]
mod tests {
    use super::*;
    use penrose::core::bindings::parse_keybindings_with_xmodmap;
    use tracing_subscriber::{self, prelude::*};

    #[test]
    fn bindings_parse_correctly_with_xmodmap() {
        let file_appender = tracing_appender::rolling::daily("/home/alex/wmlogs", "log_");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        let tracing_builder = tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_writer(non_blocking)
            .with_filter_reloading();

        let reload_handle = tracing_builder.reload_handle();
        tracing_builder.finish().init();

        let res = parse_keybindings_with_xmodmap(raw_key_bindings(reload_handle));

        if let Err(e) = res {
            panic!("{e}");
        }
    }
}
