use winapi::shared::minwindef::{WPARAM, LPARAM, TRUE};
use winapi::shared::windef::SIZE;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use crate::win32::window_helper as wh;
use crate::win32::base_helper::check_hwnd;
use crate::{Font, NwgError};
use super::{ControlHandle, ControlBase};

const NOT_BOUND: &'static str = "SysLink is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: SysLink handle is not HWND!";

bitflags! {
    /// SysLink style flags
    pub struct SysLinkFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        /// Transparent background
        const TRANSPARENT = LWS_TRANSPARENT;
        /// The background mixture of the link changes to a lighter color when hovered
        const IGNORE_RETURN = LWS_IGNORERETURN;
        /// Windows Vista. Do not underline hover link
        const NO_UNDERLINE_HOVER = LWS_NOPREFIX;
        /// Windows Vista. Uses custom text and link colors
        const USE_CUSTOM_TEXT = LWS_USEVISUALSTYLE;
        /// Links are right-aligned
        const RIGHT = LWS_RIGHT;
    }
}

/**
A SysLink control displays hyperlinks within text. It renders marked-up text
containing anchor tags (`<a href="...">link text</a>`) and notifies the
application when users click the embedded hyperlinks.

SysLink supports the anchor tag `<a>` with HREF and ID attributes:
- HREF can be any protocol: http, https, ftp, mailto, etc.
- ID is an optional name associated with an individual link

Example markup: `"Visit <a href=\"https://example.com\">our website</a> for more info."`

Requires the `syslink` feature.

**Builder parameters:**
  * `parent`:     **Required.** The syslink parent container.
  * `text`:       The text with hyperlink markup.
  * `size`:       The syslink size.
  * `position`:   The syslink position.
  * `enabled`:    If the syslink is enabled.
  * `flags`:      SysLink style flags.
  * `ex_flags`:   Extended window style flags.
  * `font`:       The font used for the text.

**Control events:**
  * `OnSysLinkClick`: When a link in the control is clicked

```rust
use native_windows_gui as nwg;
fn build_syslink(link: &mut nwg::SysLink, window: &nwg::Window) {
    nwg::SysLink::builder()
        .text("Click <a href=\"https://github.com\">here</a> to visit GitHub")
        .parent(window)
        .build(link);
}
```
*/
#[derive(Default)]
pub struct SysLink {
    pub handle: ControlHandle,
}

impl SysLink {
    pub fn builder<'a>() -> SysLinkBuilder<'a> {
        SysLinkBuilder {
            text: "",
            size: (200, 25),
            position: (0, 0),
            enabled: true,
            flags: None,
            ex_flags: 0,
            font: None,
            parent: None,
        }
    }

    /// Return the text of the syslink (including markup)
    pub fn text(&self) -> String {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_text(handle) }
    }

    /// Set the text of the syslink (can include markup)
    pub fn set_text(&self, text: &str) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_text(handle, text); }
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
        unsafe { EnableWindow(handle, if enabled { TRUE } else { 0 }); }
    }

    /// Return the position of the control in the parent window
    pub fn position(&self) -> (i32, i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_position(handle) }
    }

    /// Set the position of the control in the parent window
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

    /// Get the ideal size to display the link text
    pub fn ideal_size(&self) -> (i32, i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let mut size = SIZE { cx: 0, cy: 0 };

        // Get the current size as max width
        let (w, _) = unsafe { wh::get_window_size(handle) };
        size.cx = w as i32;

        wh::send_message(handle, LM_GETIDEALSIZE, w as WPARAM, &mut size as *mut SIZE as LPARAM);

        (size.cx, size.cy)
    }

    /// Winapi class name
    pub fn class_name(&self) -> &'static str {
        "SysLink"
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

impl Drop for SysLink {
    fn drop(&mut self) {
        self.handle.destroy();
    }
}

impl PartialEq for SysLink {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

pub struct SysLinkBuilder<'a> {
    text: &'a str,
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    flags: Option<SysLinkFlags>,
    ex_flags: u32,
    font: Option<&'a Font>,
    parent: Option<ControlHandle>,
}

impl<'a> SysLinkBuilder<'a> {
    pub fn text(mut self, text: &'a str) -> SysLinkBuilder<'a> {
        self.text = text;
        self
    }

    pub fn size(mut self, size: impl Into<(i32, i32)>) -> SysLinkBuilder<'a> {
        self.size = size.into();
        self
    }

    pub fn position(mut self, position: impl Into<(i32, i32)>) -> SysLinkBuilder<'a> {
        self.position = position.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> SysLinkBuilder<'a> {
        self.enabled = enabled;
        self
    }

    pub fn flags(mut self, flags: SysLinkFlags) -> SysLinkBuilder<'a> {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, ex_flags: u32) -> SysLinkBuilder<'a> {
        self.ex_flags = ex_flags;
        self
    }

    pub fn font(mut self, font: Option<&'a Font>) -> SysLinkBuilder<'a> {
        self.font = font;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> SysLinkBuilder<'a> {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut SysLink) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("SysLink"))
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
            .text(self.text)
            .parent(Some(parent))
            .build()?;

        // Set font
        if self.font.is_some() {
            out.set_font(self.font);
        } else {
            out.set_font(Font::global_default().as_ref());
        }

        if !self.enabled {
            out.set_enabled(false);
        }

        Ok(())
    }
}
