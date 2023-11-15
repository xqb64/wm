//! My personal penrose config
use penrose::{
    builtin::{
        actions::{key_handler, modify_with, send_layout_message, spawn},
        hooks::SpacingHook,
        layout::messages::{ExpandMain, IncMain, ShrinkMain},
    },
    core::{
        bindings::{parse_keybindings_with_xmodmap, KeyEventHandler},
        Config, WindowManager, State,
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
    x::{query::ClassName, XConn},
    x11rb::RustConn, pure::StackSet,
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
        "A-1" => show_workspace(1),
        "A-2" => show_workspace(2),
        "A-3" => show_workspace(3),
        "A-4" => show_workspace(4),
        "A-5" => show_workspace(5),
        "A-6" => show_workspace(6),
        "A-7" => show_workspace(7),
        "A-8" => show_workspace(8),
        "A-8" => show_workspace(9),

    };

    for tag in &["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
        raw_bindings.extend([
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

// A state extension for tracking last screen of each workspace
struct WorkspaceScreenTracker(HashMap<String, usize>);

fn show_workspace(ws: usize) -> KeyHandler {
    key_handler(move |state, x: &RustConn| {
        let _s = state.extension::<WorkspaceScreenTracker>()?;
        let s = _s.borrow();
        
        let ws_id = ws.to_string();

        if let Some(previous_screen) = s.0.get(&ws_id) {
            let mut cs = state.client_set.clone();
            
            cs.focus_screen(*previous_screen);
        }
        
        Ok(())
    })
}


fn add_workspace_screen_tracker_state<X>(mut wm: WindowManager<X>) -> WindowManager<X>
where
    X: XConn + 'static
{
    wm.state.add_extension(WorkspaceScreenTracker(HashMap::new()));
    wm.state.config.compose_or_set_refresh_hook(refresh_hook);
    wm
}


fn refresh_hook<X: XConn>(state: &mut State<X>, x: &X) -> penrose::Result<()> {
    let s = state.extension::<WorkspaceScreenTracker>()?;
   
    let ws_id = state.client_set.current_tag();
    let screen_id = state.client_set.current_screen().index();

    s.borrow_mut().0.insert(ws_id.to_string(), screen_id);

    Ok(())
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

    let wm = add_workspace_screen_tracker_state(WindowManager::new(config, key_bindings, HashMap::new(), conn)?);

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
