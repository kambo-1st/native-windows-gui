/*!
    A simple example demonstrating the Hot Key control.

    The Hot Key control allows users to enter keyboard shortcut combinations
    like Ctrl+S, Alt+F4, etc.

    Requires the `hot-key` feature.
*/

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct HotKeyExample {
    #[nwg_control(size: (350, 200), position: (300, 300), title: "Hot Key Example", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [HotKeyExample::exit])]
    window: nwg::Window,

    #[nwg_control(text: "Enter a keyboard shortcut:", position: (20, 15), size: (200, 20))]
    label1: nwg::Label,

    #[nwg_control(position: (20, 40), size: (150, 23))]
    #[nwg_events(OnHotKeyChanged: [HotKeyExample::on_hotkey_changed])]
    hot_key: nwg::HotKey,

    #[nwg_control(text: "Set Ctrl+S", position: (180, 40), size: (140, 25))]
    #[nwg_events(OnButtonClick: [HotKeyExample::set_ctrl_s])]
    set_ctrl_s_btn: nwg::Button,

    #[nwg_control(text: "Set Ctrl+Shift+N", position: (180, 70), size: (140, 25))]
    #[nwg_events(OnButtonClick: [HotKeyExample::set_ctrl_shift_n])]
    set_ctrl_shift_n_btn: nwg::Button,

    #[nwg_control(text: "Clear", position: (180, 100), size: (140, 25))]
    #[nwg_events(OnButtonClick: [HotKeyExample::clear])]
    clear_btn: nwg::Button,

    #[nwg_control(text: "Require Modifier", position: (180, 130), size: (140, 25))]
    #[nwg_events(OnButtonClick: [HotKeyExample::require_modifier])]
    rules_btn: nwg::Button,

    #[nwg_control(text: "Current: (none)", position: (20, 80), size: (150, 80))]
    status_label: nwg::Label,
}

impl HotKeyExample {
    fn on_hotkey_changed(&self) {
        if let Some(value) = self.hot_key.value() {
            let mut mods = Vec::new();
            if value.modifiers.contains(nwg::HotKeyModifiers::CONTROL) {
                mods.push("Ctrl");
            }
            if value.modifiers.contains(nwg::HotKeyModifiers::SHIFT) {
                mods.push("Shift");
            }
            if value.modifiers.contains(nwg::HotKeyModifiers::ALT) {
                mods.push("Alt");
            }

            let key_name = match value.key {
                0x41..=0x5A => format!("{}", value.key as char), // A-Z
                0x30..=0x39 => format!("{}", (value.key - 0x30) as char), // 0-9
                0x70..=0x7B => format!("F{}", value.key - 0x6F), // F1-F12
                0x08 => "Backspace".to_string(),
                0x09 => "Tab".to_string(),
                0x0D => "Enter".to_string(),
                0x1B => "Escape".to_string(),
                0x20 => "Space".to_string(),
                0x2E => "Delete".to_string(),
                0x2D => "Insert".to_string(),
                0x24 => "Home".to_string(),
                0x23 => "End".to_string(),
                0x21 => "Page Up".to_string(),
                0x22 => "Page Down".to_string(),
                _ => format!("0x{:02X}", value.key),
            };

            let combo = if mods.is_empty() {
                key_name
            } else {
                format!("{}+{}", mods.join("+"), key_name)
            };

            self.status_label.set_text(&format!("Current:\n{}", combo));
        } else {
            self.status_label.set_text("Current: (none)");
        }
    }

    fn set_ctrl_s(&self) {
        self.hot_key.set_value(nwg::HotKeyValue::ctrl(b'S'));
        self.on_hotkey_changed();
    }

    fn set_ctrl_shift_n(&self) {
        self.hot_key.set_value(nwg::HotKeyValue::ctrl_shift(b'N'));
        self.on_hotkey_changed();
    }

    fn clear(&self) {
        self.hot_key.clear();
        self.status_label.set_text("Current: (none)");
    }

    fn require_modifier(&self) {
        // Disallow keys without any modifier - replace with Ctrl
        self.hot_key.set_rules(
            nwg::HotKeyInvalidCombinations::NONE,
            nwg::HotKeyModifiers::CONTROL
        );
        self.status_label.set_text("Rule set:\nKeys without\nmodifier → Ctrl+Key");
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _app = HotKeyExample::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
