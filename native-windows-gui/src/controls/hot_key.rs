use winapi::shared::minwindef::{WPARAM, LPARAM, TRUE, FALSE};
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use crate::win32::window_helper as wh;
use crate::win32::base_helper::check_hwnd;
use crate::{Font, NwgError};
use super::{ControlHandle, ControlBase};

const NOT_BOUND: &'static str = "HotKey is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: HotKey handle is not HWND!";

bitflags! {
    /// Hot Key control style flags
    pub struct HotKeyFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        const TAB_STOP = WS_TABSTOP;
    }
}

bitflags! {
    /// Modifier keys for hot key combinations
    #[derive(Default)]
    pub struct HotKeyModifiers: u8 {
        const SHIFT = HOTKEYF_SHIFT;
        const CONTROL = HOTKEYF_CONTROL;
        const ALT = HOTKEYF_ALT;
        const EXT = HOTKEYF_EXT;
    }
}

bitflags! {
    /// Invalid key combination flags for set_rules
    pub struct HotKeyInvalidCombinations: usize {
        /// Disallow no modifiers (just the key alone)
        const NONE = HKCOMB_NONE;
        /// Disallow SHIFT as the only modifier
        const SHIFT = HKCOMB_S;
        /// Disallow CTRL as the only modifier
        const CONTROL = HKCOMB_C;
        /// Disallow ALT as the only modifier
        const ALT = HKCOMB_A;
        /// Disallow SHIFT+CTRL
        const SHIFT_CONTROL = HKCOMB_SC;
        /// Disallow SHIFT+ALT
        const SHIFT_ALT = HKCOMB_SA;
        /// Disallow CTRL+ALT
        const CONTROL_ALT = HKCOMB_CA;
        /// Disallow SHIFT+CTRL+ALT
        const SHIFT_CONTROL_ALT = HKCOMB_SCA;
    }
}

/// Represents a hot key combination (virtual key + modifiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HotKeyValue {
    /// The virtual key code (e.g., 'A' = 0x41, VK_F1 = 0x70)
    pub key: u8,
    /// The modifier keys (Shift, Ctrl, Alt, Ext)
    pub modifiers: HotKeyModifiers,
}

impl HotKeyValue {
    /// Create a new hot key value
    pub fn new(key: u8, modifiers: HotKeyModifiers) -> Self {
        HotKeyValue { key, modifiers }
    }

    /// Create a hot key with no modifiers
    pub fn key_only(key: u8) -> Self {
        HotKeyValue { key, modifiers: HotKeyModifiers::empty() }
    }

    /// Create a hot key with Ctrl modifier
    pub fn ctrl(key: u8) -> Self {
        HotKeyValue { key, modifiers: HotKeyModifiers::CONTROL }
    }

    /// Create a hot key with Alt modifier
    pub fn alt(key: u8) -> Self {
        HotKeyValue { key, modifiers: HotKeyModifiers::ALT }
    }

    /// Create a hot key with Shift modifier
    pub fn shift(key: u8) -> Self {
        HotKeyValue { key, modifiers: HotKeyModifiers::SHIFT }
    }

    /// Create a hot key with Ctrl+Shift modifiers
    pub fn ctrl_shift(key: u8) -> Self {
        HotKeyValue { key, modifiers: HotKeyModifiers::CONTROL | HotKeyModifiers::SHIFT }
    }

    /// Create a hot key with Ctrl+Alt modifiers
    pub fn ctrl_alt(key: u8) -> Self {
        HotKeyValue { key, modifiers: HotKeyModifiers::CONTROL | HotKeyModifiers::ALT }
    }

    /// Check if the hot key is empty (no key assigned)
    pub fn is_empty(&self) -> bool {
        self.key == 0
    }

    /// Convert to the WPARAM format used by HKM_SETHOTKEY
    fn to_wparam(&self) -> WPARAM {
        ((self.modifiers.bits() as WPARAM) << 8) | (self.key as WPARAM)
    }

    /// Create from the LPARAM format returned by HKM_GETHOTKEY
    fn from_lparam(value: LPARAM) -> Self {
        HotKeyValue {
            key: (value & 0xFF) as u8,
            modifiers: HotKeyModifiers::from_bits_truncate(((value >> 8) & 0xFF) as u8),
        }
    }
}

/**
A Hot Key control allows the user to enter a keyboard shortcut combination
(like Ctrl+S or Alt+F4). The control displays the key combination and
validates it against configurable rules.

Requires the `hot-key` feature.

**Builder parameters:**
  * `parent`:     **Required.** The control parent container.
  * `size`:       The control size.
  * `position`:   The control position.
  * `enabled`:    If the control is enabled.
  * `flags`:      Hot Key style flags.
  * `ex_flags`:   Extended window style flags.
  * `font`:       The font used for the control.
  * `focus`:      The control receives focus after being created.

**Control events:**
  * `OnHotKeyChanged`: When the hot key combination changes

```rust
use native_windows_gui as nwg;

fn build_hot_key(hk: &mut nwg::HotKey, window: &nwg::Window) {
    nwg::HotKey::builder()
        .size((150, 25))
        .position((10, 10))
        .parent(window)
        .build(hk);

    // Set a hot key (Ctrl+S)
    hk.set_value(nwg::HotKeyValue::ctrl(b'S'));

    // Get the current hot key
    if let Some(value) = hk.value() {
        println!("Key: {}, Modifiers: {:?}", value.key, value.modifiers);
    }
}
```
*/
#[derive(Default)]
pub struct HotKey {
    pub handle: ControlHandle,
}

impl HotKey {
    pub fn builder() -> HotKeyBuilder {
        HotKeyBuilder {
            size: (150, 23),
            position: (0, 0),
            enabled: true,
            focus: false,
            flags: None,
            ex_flags: 0,
            font: None,
            parent: None,
        }
    }

    /// Set the hot key value
    pub fn set_value(&self, value: HotKeyValue) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, HKM_SETHOTKEY, value.to_wparam(), 0);
    }

    /// Get the current hot key value. Returns None if no key is set.
    pub fn value(&self) -> Option<HotKeyValue> {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let result = wh::send_message(handle, HKM_GETHOTKEY, 0, 0);
        let value = HotKeyValue::from_lparam(result);
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    /// Clear the hot key value
    pub fn clear(&self) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, HKM_SETHOTKEY, 0, 0);
    }

    /// Set rules for invalid key combinations.
    /// When an invalid combination is entered, it will be replaced with the
    /// specified replacement modifiers.
    ///
    /// # Arguments
    /// * `invalid` - Combinations to disallow
    /// * `replacement` - Modifiers to use when an invalid combination is entered
    pub fn set_rules(&self, invalid: HotKeyInvalidCombinations, replacement: HotKeyModifiers) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(
            handle,
            HKM_SETRULES,
            invalid.bits(),
            (replacement.bits() as LPARAM) << 16
        );
    }

    /// Return the font of the control
    pub fn font(&self) -> Option<Font> {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let font_handle = wh::get_window_font(handle);
        if font_handle.is_null() {
            None
        } else {
            Some(Font { handle: font_handle })
        }
    }

    /// Set the font of the control
    pub fn set_font(&self, font: Option<&Font>) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_font(handle, font.map(|f| f.handle), true); }
    }

    /// Return true if the control is visible
    pub fn visible(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { IsWindowVisible(handle) != 0 }
    }

    /// Show or hide the control
    pub fn set_visible(&self, visible: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { ShowWindow(handle, if visible { SW_SHOW } else { SW_HIDE }); }
    }

    /// Return true if the control is enabled
    pub fn enabled(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { IsWindowEnabled(handle) != 0 }
    }

    /// Enable or disable the control
    pub fn set_enabled(&self, enabled: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { EnableWindow(handle, if enabled { TRUE } else { FALSE }); }
    }

    /// Return the position of the control
    pub fn position(&self) -> (i32, i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_position(handle) }
    }

    /// Set the position of the control
    pub fn set_position(&self, x: i32, y: i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_position(handle, x, y); }
    }

    /// Return the size of the control
    pub fn size(&self) -> (u32, u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_size(handle) }
    }

    /// Set the size of the control
    pub fn set_size(&self, w: u32, h: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_size(handle, w, h, false); }
    }

    /// Set focus to this control
    pub fn set_focus(&self) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { SetFocus(handle); }
    }

    /// Winapi class name
    pub fn class_name(&self) -> &'static str {
        HOTKEY_CLASS
    }

    /// Winapi flags
    pub fn flags(&self) -> u32 {
        WS_VISIBLE | WS_TABSTOP
    }

    /// Required flags
    pub fn forced_flags(&self) -> u32 {
        WS_CHILD
    }
}

impl Drop for HotKey {
    fn drop(&mut self) {
        self.handle.destroy();
    }
}

impl PartialEq for HotKey {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

pub struct HotKeyBuilder {
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    focus: bool,
    flags: Option<HotKeyFlags>,
    ex_flags: u32,
    font: Option<Font>,
    parent: Option<ControlHandle>,
}

impl HotKeyBuilder {
    pub fn size(mut self, size: impl Into<(i32, i32)>) -> HotKeyBuilder {
        self.size = size.into();
        self
    }

    pub fn position(mut self, position: impl Into<(i32, i32)>) -> HotKeyBuilder {
        self.position = position.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> HotKeyBuilder {
        self.enabled = enabled;
        self
    }

    pub fn focus(mut self, focus: bool) -> HotKeyBuilder {
        self.focus = focus;
        self
    }

    pub fn flags(mut self, flags: HotKeyFlags) -> HotKeyBuilder {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, ex_flags: u32) -> HotKeyBuilder {
        self.ex_flags = ex_flags;
        self
    }

    pub fn font(mut self, font: Option<&Font>) -> HotKeyBuilder {
        self.font = font.map(|f| Font { handle: f.handle });
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> HotKeyBuilder {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut HotKey) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("HotKey"))
        }?;

        *out = Default::default();

        let flags = self.flags.map(|f| f.bits()).unwrap_or(out.flags());

        out.handle = ControlBase::build_hwnd()
            .class_name(out.class_name())
            .forced_flags(out.forced_flags())
            .flags(flags)
            .ex_flags(self.ex_flags)
            .size(self.size)
            .position(self.position)
            .text("")
            .parent(Some(parent))
            .build()?;

        // Set font
        if self.font.is_some() {
            out.set_font(self.font.as_ref());
        } else {
            out.set_font(Font::global_default().as_ref());
        }

        if !self.enabled {
            out.set_enabled(false);
        }

        if self.focus {
            out.set_focus();
        }

        Ok(())
    }
}
