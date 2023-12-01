use std::{collections::HashMap, io::Write, process::ChildStdin};

use crate::KeyHandler;
use penrose::{
    builtin::actions::key_handler,
    core::{bindings::KeyEventHandler, State, WindowManager},
    custom_error,
    extensions::{
        hooks::ToggleNamedScratchPad,
        util::dmenu::{DMenu, DMenuConfig, MenuMatch},
    },
    pure::Workspace,
    x::{XConn, XConnExt},
    x11rb::RustConn,
    Result, Xid,
};
use tracing::warn;
use tracing_subscriber::{reload::Handle, EnvFilter};

struct FixedWorkspaces(HashMap<String, usize>);
struct XmobarHandle(ChildStdin);
struct NamedScratchpadsState(HashMap<String, bool>);

pub fn add_fixed_workspaces_state<X>(mut wm: WindowManager<X>) -> WindowManager<X>
where
    X: XConn + 'static,
{
    let screen_count = wm.state.client_set.screens().count();
    let mut map = HashMap::new();
    for (k, v) in (1..=12).zip((0..screen_count).cycle()) {
        map.insert(k.to_string(), v);
    }
    wm.state.add_extension(FixedWorkspaces(map));
    wm
}

pub fn add_xmobar_handle<X>(mut wm: WindowManager<X>, handle: ChildStdin) -> WindowManager<X>
where
    X: XConn + 'static,
{
    wm.state.add_extension(XmobarHandle(handle));
    wm.state.config.compose_or_set_refresh_hook(refresh_hook);
    wm
}

pub fn add_namedscratchpads_state<X>(
    mut wm: WindowManager<X>,
    sratchpads: Vec<&str>,
) -> WindowManager<X>
where
    X: XConn + 'static,
{
    let mut map = HashMap::new();
    for scratchpad in sratchpads {
        map.insert(scratchpad.to_string(), false);
    }
    wm.state.add_extension(NamedScratchpadsState(map));

    wm
}

pub fn toggle_namedscratchpad(
    class: String,
    mut toggle_scratch: ToggleNamedScratchPad,
) -> KeyHandler {
    key_handler(move |state, x: &RustConn| {
        let _s = state.extension::<NamedScratchpadsState>()?;
        let mut s = _s.borrow_mut();

        if let Some(visible) = s.0.get_mut(class.as_str()) {
            *visible = !*visible;

            drop(s);

            toggle_scratch.call(state, x)?;
        }

        Ok(())
    })
}

pub fn show_workspace(ws: &str) -> KeyHandler {
    let ws = ws.to_owned();

    key_handler(move |state, x: &RustConn| {
        let _s = state.extension::<FixedWorkspaces>()?;
        let s = _s.borrow();

        let screen_idx = s.0.get(&ws).unwrap();
        let target_screen = state
            .client_set
            .screens_mut()
            .find(|s| s.index() == *screen_idx)
            .unwrap();
        let current_ws = &mut target_screen.workspace as *mut Workspace<Xid>;
        let target_ws = state.client_set.workspace_mut(&ws).unwrap() as *mut Workspace<Xid>;

        unsafe {
            std::ptr::swap(current_ws, target_ws);
        }

        state.client_set.focus_screen(*screen_idx);

        drop(s);
        x.refresh(state)?;

        Ok(())
    })
}

pub fn set_tracing_filter<L, S>(handle: Handle<L, S>) -> KeyHandler
where
    L: From<EnvFilter> + 'static,
    S: 'static,
{
    key_handler(move |state, _| {
        let options = vec!["trace", "debug", "info"];
        let screen_index = state.client_set.current_screen().index();
        let menu = DMenu::new(&DMenuConfig::with_prompt("filter: "), screen_index);

        let new_filter = match menu.build_menu(options)? {
            MenuMatch::Line(_, level) => level,
            MenuMatch::UserInput(custom) => custom,
            MenuMatch::NoMatch => return Ok(()),
        };

        warn!(?new_filter, "attempting to update tracing filter");
        let f = new_filter
            .parse::<EnvFilter>()
            .map_err(|e| custom_error!("invalid filter: {}", e))?;
        warn!("reloading tracing handle");
        handle
            .reload(f)
            .map_err(|e| custom_error!("unable to set filter: {}", e))
    })
}

pub fn spawn_dmenu() -> KeyHandler {
    key_handler(|state, _| {
        let current_screen_idx = state.client_set.current_screen().index();
        penrose::util::spawn(format!("dmenu_run -m {current_screen_idx}"))
    })
}

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
                "restart-wm" => std::process::exit(0),
                _ => unimplemented!(),
            }
        } else {
            Ok(())
        }
    })
}

fn refresh_hook<X: XConn>(state: &mut State<X>, x: &X) -> Result<()> {
    let _s = state.extension::<XmobarHandle>()?;
    _s.borrow_mut()
        .0
        .write_all(display_workspaces(state)?.as_bytes())?;

    x.refresh(state)?;

    Ok(())
}

fn display_workspaces<X: XConn>(state: &mut State<X>) -> Result<String> {
    let scratchpad_names = vec!["term", "vpn"];

    let workspaces = state
        .client_set
        .ordered_workspaces()
        .map(|w| w.tag())
        .collect::<Vec<_>>();

    let active_ws = state.client_set.current_workspace().tag();
    let inactive_but_visible_ws = state
        .client_set
        .screens()
        .filter(|screen| screen.index() != state.client_set.current_screen().index())
        .map(|screen| screen.workspace.tag())
        .collect::<Vec<_>>();

    let empty_workspaces = state
        .client_set
        .workspaces()
        .filter(|ws| ws.is_empty())
        .map(|ws| ws.tag())
        .collect::<Vec<_>>();

    let occupied_workspaces = state
        .client_set
        .workspaces()
        .filter(|ws| !ws.is_empty())
        .map(|ws| ws.tag())
        .collect::<Vec<_>>();

    let mut s = vec![];

    for ws in workspaces {
        let color = if occupied_workspaces.contains(&ws) {
            "white"
        } else if empty_workspaces.contains(&ws) {
            "gray"
        } else {
            unreachable!();
        };

        if ws == active_ws {
            s.push(format_active_ws(&format_action(ws), color));
        } else if inactive_but_visible_ws.contains(&ws) {
            s.push(format_inactive_but_visible_ws(&format_action(ws), color));
        } else {
            s.push(format_inactive_ws(&format_action(ws), color));
        }
    }

    for scratchpad_name in scratchpad_names {
        s.push(format_named_scratchpad(state, scratchpad_name)?);
    }

    s.push(" ".to_string());
    s.push("\n".to_string());

    let s = s.join(" ");

    Ok(s)
}

fn format_action(ws: &str) -> String {
    let idx = ws.parse::<usize>().unwrap().saturating_sub(1);
    format!("<action=`xdotool set_desktop {idx}`>{ws}</action>")
}

fn format_active_ws(ws: &str, color: &str) -> String {
    format!("<fc=#42cbf5>[</fc><fc={color}>{ws}</fc><fc=#42cbf5>]</fc>")
}

fn format_inactive_ws(ws: &str, color: &str) -> String {
    format!(r#"<fc={color}>{ws}</fc>"#)
}

fn format_inactive_but_visible_ws(ws: &str, color: &str) -> String {
    format!("<fc=gray>(</fc><fc={color}>{ws}</fc><fc=gray>)</fc>")
}

fn format_named_scratchpad<X: XConn>(state: &State<X>, scratchpad: &str) -> Result<String> {
    if is_named_scratchpad_active(state, scratchpad)? {
        Ok(format!(
            "<fc=#42cbf5>[</fc><fc=white>{scratchpad}</fc><fc=#42cbf5>]</fc>"
        ))
    } else {
        Ok(format!("<fc=gray>({scratchpad})</fc>"))
    }
}

fn is_named_scratchpad_active<X: XConn>(state: &State<X>, class: &str) -> Result<bool> {
    let s = state.extension::<NamedScratchpadsState>()?;
    let s = s.borrow();
    Ok(*s.0.get(class).unwrap_or(&false))
}
