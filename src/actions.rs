use std::collections::HashMap;

use crate::KeyHandler;
use penrose::{
    builtin::actions::key_handler,
    core::WindowManager,
    custom_error,
    extensions::util::dmenu::{DMenu, DMenuConfig, MenuMatch},
    pure::Workspace,
    x::{XConn, XConnExt},
    x11rb::RustConn,
    Xid,
};
use tracing::warn;
use tracing_subscriber::{reload::Handle, EnvFilter};

struct FixedWorkspaces(HashMap<String, usize>);

pub fn add_fixed_workspaces_state<X>(mut wm: WindowManager<X>) -> WindowManager<X>
where
    X: XConn + 'static,
{
    let mut map = HashMap::new();
    for (k, v) in (1..=9).zip((0..=1).cycle()) {
        map.insert(k.to_string(), v);
    }
    wm.state.add_extension(FixedWorkspaces(map));
    wm
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
