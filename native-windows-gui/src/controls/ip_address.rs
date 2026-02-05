use winapi::shared::minwindef::{WPARAM, LPARAM, TRUE, FALSE, DWORD};
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use crate::win32::window_helper as wh;
use crate::win32::base_helper::check_hwnd;
use crate::{Font, NwgError};
use super::{ControlHandle, ControlBase};

const NOT_BOUND: &'static str = "IpAddress is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: IpAddress handle is not HWND!";

bitflags! {
    /// IP Address control style flags
    pub struct IpAddressFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        const TAB_STOP = WS_TABSTOP;
    }
}

/**
An IP Address control allows the user to enter an IPv4 address in an
easily understood format. The control consists of four fields, one for
each octet of the IP address.

Requires the `ip-address` feature.

**Builder parameters:**
  * `parent`:     **Required.** The control parent container.
  * `size`:       The control size.
  * `position`:   The control position.
  * `enabled`:    If the control is enabled.
  * `flags`:      IP Address style flags.
  * `ex_flags`:   Extended window style flags.
  * `font`:       The font used for the control.
  * `focus`:      The control receives focus after being created.

**Control events:**
  * `OnIpAddressFieldChanged`: When a field value changes

```rust
use native_windows_gui as nwg;

fn build_ip_address(ip: &mut nwg::IpAddress, window: &nwg::Window) {
    nwg::IpAddress::builder()
        .size((150, 25))
        .position((10, 10))
        .parent(window)
        .build(ip);

    // Set an address
    ip.set_address([192, 168, 1, 1]);

    // Get the address
    if let Some(addr) = ip.address() {
        println!("Address: {}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]);
    }
}
```
*/
#[derive(Default)]
pub struct IpAddress {
    pub handle: ControlHandle,
}

impl IpAddress {
    pub fn builder() -> IpAddressBuilder {
        IpAddressBuilder {
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

    /// Set the IP address. Each element is an octet (0-255).
    pub fn set_address(&self, addr: [u8; 4]) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let ip = MAKEIPADDRESS(addr[0] as DWORD, addr[1] as DWORD, addr[2] as DWORD, addr[3] as DWORD);
        wh::send_message(handle, IPM_SETADDRESS, 0, ip);
    }

    /// Get the IP address. Returns None if any field is blank.
    /// Returns Some([a, b, c, d]) where each element is an octet.
    pub fn address(&self) -> Option<[u8; 4]> {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let mut ip: DWORD = 0;
        let filled = wh::send_message(handle, IPM_GETADDRESS, 0, &mut ip as *mut _ as LPARAM);

        if filled == 4 {
            Some([
                FIRST_IPADDRESS(ip as LPARAM),
                SECOND_IPADDRESS(ip as LPARAM),
                THIRD_IPADDRESS(ip as LPARAM),
                FOURTH_IPADDRESS(ip as LPARAM),
            ])
        } else {
            None
        }
    }

    /// Get the IP address even if some fields are blank.
    /// Blank fields will be 0. Returns the number of non-blank fields.
    pub fn address_partial(&self) -> ([u8; 4], usize) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let mut ip: DWORD = 0;
        let filled = wh::send_message(handle, IPM_GETADDRESS, 0, &mut ip as *mut _ as LPARAM);

        let addr = [
            FIRST_IPADDRESS(ip as LPARAM),
            SECOND_IPADDRESS(ip as LPARAM),
            THIRD_IPADDRESS(ip as LPARAM),
            FOURTH_IPADDRESS(ip as LPARAM),
        ];

        (addr, filled as usize)
    }

    /// Clear the IP address (all fields become blank)
    pub fn clear(&self) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, IPM_CLEARADDRESS, 0, 0);
    }

    /// Check if all fields are blank
    pub fn is_blank(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, IPM_ISBLANK, 0, 0) != 0
    }

    /// Set the valid range for a field (0-3).
    /// Values outside the range will be clamped.
    pub fn set_field_range(&self, field: u8, min: u8, max: u8) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let range = MAKEIPRANGE(min, max);
        wh::send_message(handle, IPM_SETRANGE, field as WPARAM, range);
    }

    /// Set focus to a specific field (0-3)
    pub fn focus_field(&self, field: u8) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, IPM_SETFOCUS, field as WPARAM, 0);
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
        WC_IPADDRESS
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

impl Drop for IpAddress {
    fn drop(&mut self) {
        self.handle.destroy();
    }
}

impl PartialEq for IpAddress {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

pub struct IpAddressBuilder {
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    focus: bool,
    flags: Option<IpAddressFlags>,
    ex_flags: u32,
    font: Option<Font>,
    parent: Option<ControlHandle>,
}

impl IpAddressBuilder {
    pub fn size(mut self, size: impl Into<(i32, i32)>) -> IpAddressBuilder {
        self.size = size.into();
        self
    }

    pub fn position(mut self, position: impl Into<(i32, i32)>) -> IpAddressBuilder {
        self.position = position.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> IpAddressBuilder {
        self.enabled = enabled;
        self
    }

    pub fn focus(mut self, focus: bool) -> IpAddressBuilder {
        self.focus = focus;
        self
    }

    pub fn flags(mut self, flags: IpAddressFlags) -> IpAddressBuilder {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, ex_flags: u32) -> IpAddressBuilder {
        self.ex_flags = ex_flags;
        self
    }

    pub fn font(mut self, font: Option<&Font>) -> IpAddressBuilder {
        self.font = font.map(|f| Font { handle: f.handle });
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> IpAddressBuilder {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut IpAddress) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("IpAddress"))
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
