use winapi::shared::minwindef::{LPARAM, TRUE, FALSE, DWORD};
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use crate::win32::window_helper as wh;
use crate::win32::base_helper::check_hwnd;
use crate::{Font, NwgError};
use super::{ControlHandle, ControlBase};

const NOT_BOUND: &'static str = "Pager is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: Pager handle is not HWND!";

bitflags! {
    /// Pager control style flags
    pub struct PagerFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        const TAB_STOP = WS_TABSTOP;
        /// Vertical pager (scroll up/down)
        const VERTICAL = PGS_VERT;
        /// Horizontal pager (scroll left/right)
        const HORIZONTAL = PGS_HORZ;
        /// Enable auto-scroll when hovering over scroll buttons
        const AUTOSCROLL = PGS_AUTOSCROLL;
        /// Enable drag and drop support
        const DRAGNDROP = PGS_DRAGNDROP;
    }
}

/// Pager button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagerButton {
    /// Top or left button
    TopLeft,
    /// Bottom or right button
    BottomRight,
}

/// Pager button states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagerButtonState {
    /// Button is invisible
    Invisible,
    /// Button is normal (visible, not pressed)
    Normal,
    /// Button is grayed (disabled)
    Grayed,
    /// Button is depressed (pressed)
    Depressed,
    /// Button is hot (mouse hover)
    Hot,
}

impl PagerButtonState {
    fn from_raw(value: DWORD) -> Self {
        match value {
            PGF_INVISIBLE => PagerButtonState::Invisible,
            PGF_NORMAL => PagerButtonState::Normal,
            PGF_GRAYED => PagerButtonState::Grayed,
            PGF_DEPRESSED => PagerButtonState::Depressed,
            PGF_HOT => PagerButtonState::Hot,
            _ => PagerButtonState::Invisible,
        }
    }
}

/**
A Pager control is a container that provides a scrollable region for a child
control. It displays scroll buttons when the contained control is larger than
the pager's visible area.

Pagers are commonly used to contain toolbars that are too wide to fit in the
available space.

Requires the `pager` feature.

**Builder parameters:**
  * `parent`:      **Required.** The control parent container.
  * `size`:        The control size.
  * `position`:    The control position.
  * `enabled`:     If the control is enabled.
  * `flags`:       Pager style flags.
  * `ex_flags`:    Extended window style flags.

**Control events:**
  * `OnPagerScroll`: When the pager scrolls
  * Mouse events also work

```rust
use native_windows_gui as nwg;

fn build_pager(pager: &mut nwg::Pager, toolbar: &nwg::Toolbar, window: &nwg::Window) {
    nwg::Pager::builder()
        .size((200, 30))
        .position((10, 10))
        .parent(window)
        .flags(nwg::PagerFlags::VISIBLE | nwg::PagerFlags::HORIZONTAL)
        .build(pager);

    // Set the toolbar as the pager's child
    pager.set_child(Some(toolbar));
}
```
*/
#[derive(Default)]
pub struct Pager {
    pub handle: ControlHandle,
}

impl Pager {
    pub fn builder() -> PagerBuilder {
        PagerBuilder {
            size: (200, 30),
            position: (0, 0),
            enabled: true,
            flags: None,
            ex_flags: 0,
            parent: None,
        }
    }

    /// Set the child control contained in the pager.
    /// The child should already be created with the pager as its parent.
    pub fn set_child<C: Into<ControlHandle>>(&self, child: Option<C>) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let child_hwnd = match child {
            Some(c) => {
                let ch: ControlHandle = c.into();
                ch.hwnd().unwrap_or(std::ptr::null_mut())
            },
            None => std::ptr::null_mut(),
        };
        wh::send_message(handle, PGM_SETCHILD, 0, child_hwnd as LPARAM);
    }

    /// Force the pager to recalculate its child's size.
    /// Call this after the child control's size changes.
    pub fn recalc_size(&self) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, PGM_RECALCSIZE, 0, 0);
    }

    /// Enable or disable mouse forwarding to the child control.
    pub fn set_forward_mouse(&self, forward: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, PGM_FORWARDMOUSE, if forward { 1 } else { 0 }, 0);
    }

    /// Get the current scroll position.
    pub fn position(&self) -> i32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, PGM_GETPOS, 0, 0) as i32
    }

    /// Set the scroll position.
    pub fn set_position(&self, pos: i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, PGM_SETPOS, 0, pos as LPARAM);
    }

    /// Get the button size (width for horizontal, height for vertical pager).
    pub fn button_size(&self) -> u32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, PGM_GETBUTTONSIZE, 0, 0) as u32
    }

    /// Set the button size.
    pub fn set_button_size(&self, size: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, PGM_SETBUTTONSIZE, 0, size as LPARAM);
    }

    /// Get the border size.
    pub fn border(&self) -> u32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, PGM_GETBORDER, 0, 0) as u32
    }

    /// Set the border size.
    pub fn set_border(&self, border: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, PGM_SETBORDER, 0, border as LPARAM);
    }

    /// Get the background color as RGB.
    pub fn background_color(&self) -> [u8; 3] {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let color = wh::send_message(handle, PGM_GETBKCOLOR, 0, 0) as u32;
        [
            (color & 0xFF) as u8,
            ((color >> 8) & 0xFF) as u8,
            ((color >> 16) & 0xFF) as u8,
        ]
    }

    /// Set the background color as RGB.
    pub fn set_background_color(&self, color: [u8; 3]) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let rgb = (color[0] as LPARAM) | ((color[1] as LPARAM) << 8) | ((color[2] as LPARAM) << 16);
        wh::send_message(handle, PGM_SETBKCOLOR, 0, rgb);
    }

    /// Get the state of a pager button.
    pub fn button_state(&self, button: PagerButton) -> PagerButtonState {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let btn = match button {
            PagerButton::TopLeft => PGB_TOPORLEFT,
            PagerButton::BottomRight => PGB_BOTTOMORRIGHT,
        };
        let state = wh::send_message(handle, PGM_GETBUTTONSTATE, 0, btn as LPARAM) as DWORD;
        PagerButtonState::from_raw(state)
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

    /// Return the position of the control in the parent
    pub fn window_position(&self) -> (i32, i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_position(handle) }
    }

    /// Set the position of the control in the parent
    pub fn set_window_position(&self, x: i32, y: i32) {
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

    /// Winapi class name
    pub fn class_name(&self) -> &'static str {
        WC_PAGESCROLLER
    }

    /// Winapi flags
    pub fn flags(&self) -> u32 {
        WS_VISIBLE
    }

    /// Required flags
    pub fn forced_flags(&self) -> u32 {
        WS_CHILD
    }
}

impl Drop for Pager {
    fn drop(&mut self) {
        self.handle.destroy();
    }
}

impl PartialEq for Pager {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

pub struct PagerBuilder {
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    flags: Option<PagerFlags>,
    ex_flags: u32,
    parent: Option<ControlHandle>,
}

impl PagerBuilder {
    pub fn size(mut self, size: impl Into<(i32, i32)>) -> PagerBuilder {
        self.size = size.into();
        self
    }

    pub fn position(mut self, position: impl Into<(i32, i32)>) -> PagerBuilder {
        self.position = position.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> PagerBuilder {
        self.enabled = enabled;
        self
    }

    pub fn flags(mut self, flags: PagerFlags) -> PagerBuilder {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, ex_flags: u32) -> PagerBuilder {
        self.ex_flags = ex_flags;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> PagerBuilder {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut Pager) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("Pager"))
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

        if !self.enabled {
            out.set_enabled(false);
        }

        Ok(())
    }
}
