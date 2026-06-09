mod dap_log;
pub use dap_log::*;

use dap::debugger_settings::DebuggerSettings;
use gpui::App;
use settings::Settings;

pub fn init(cx: &mut App) {
    if !DebuggerSettings::get_global(cx).enabled {
        return;
    }
    dap_log::init(cx);
}
