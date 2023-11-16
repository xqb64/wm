//! My personal penrose config
use penrose::{
    builtin::{
        actions::{key_handler, modify_with, send_layout_message, spawn, log_current_state},
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
    x::{query::ClassName, XConn, XConnExt},
    x11rb::RustConn, pure::Workspace,
    custom_error, Xid,
};
use std::collections::HashMap;
use tracing_subscriber::{self, prelude::*};
use tracing_subscriber::{reload::Handle, EnvFilter};
use wm::layouts::layouts;
use tracing::warn;

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

fn raw_key_bindings<L, S>(handle: Handle<L, S>) -> HashMap<String, KeyHandler>
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
        "A-p" => spawn("dmenu_run"),
        "A-S-Return" => spawn("tabbed alacritty --embed"),
        "A-Escape" => power_menu(),

        // Debugging
        "M-A-t" => set_tracing_filter(handle),
        "M-A-d" => log_current_state(),

    };

    for tag in &["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
        raw_bindings.extend([
            (
                format!("A-{tag}"),
                show_workspace(tag),
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

struct FixedWorkspaces(HashMap<String, usize>);

fn show_workspace(ws: &str) -> KeyHandler {
    let ws = ws.to_owned();

    key_handler(move |state, x: &RustConn| {
        let _s = state.extension::<FixedWorkspaces>()?;
        let s = _s.borrow();
        
        let screen_idx = s.0.get(&ws).unwrap();
        let target_screen = state.client_set.screens_mut().find(|s| s.index() == *screen_idx).unwrap();
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


fn add_workspace_screen_tracker_state<X>(mut wm: WindowManager<X>) -> WindowManager<X>
where
    X: XConn + 'static
{
    let mut map = HashMap::new();
    map.insert("1".to_string(), 0);
    map.insert("2".to_string(), 1);
    map.insert("3".to_string(), 0);
    map.insert("4".to_string(), 1);
    map.insert("5".to_string(), 0);
    map.insert("6".to_string(), 1);
    map.insert("7".to_string(), 0);
    map.insert("8".to_string(), 1);
    map.insert("9".to_string(), 0);
    wm.state.add_extension(FixedWorkspaces(map));
    wm
}


fn main() -> penrose::Result<()> {
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
