use winapi::shared::minwindef::{WPARAM, LPARAM, DWORD, TRUE};
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use crate::win32::window_helper as wh;
use crate::win32::base_helper::{check_hwnd, to_utf16};
use crate::{Font, NwgError, RawEventHandler, unbind_raw_event_handler};
use super::{ControlHandle, ControlBase};
use std::cell::RefCell;
use std::ptr;

#[cfg(feature = "image-list")]
use crate::ImageList;

const NOT_BOUND: &'static str = "Toolbar is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: Toolbar handle is not HWND!";

bitflags! {
    /// Toolbar style flags
    pub struct ToolbarFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        const FLAT = TBSTYLE_FLAT;
        const LIST = TBSTYLE_LIST;
        const TOOLTIPS = TBSTYLE_TOOLTIPS;
        const WRAPABLE = TBSTYLE_WRAPABLE;
        const ADJUSTABLE = CCS_ADJUSTABLE;
        const NODIVIDER = CCS_NODIVIDER;
        const NOPARENTALIGN = CCS_NOPARENTALIGN;
        const NORESIZE = CCS_NORESIZE;
        const VERT = CCS_VERT;
    }
}

/// Represents the style of a toolbar button
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarButtonStyle {
    /// A standard push button
    Button,
    /// A separator (small gap)
    Separator,
    /// A check button (toggleable)
    Check,
    /// A button with a dropdown arrow
    Dropdown,
    /// A button that is part of a group
    Group,
    /// A button that is part of a check group
    CheckGroup,
    /// Dropdown with separate arrow area
    WholeDropdown,
}

impl Default for ToolbarButtonStyle {
    fn default() -> Self {
        ToolbarButtonStyle::Button
    }
}

impl ToolbarButtonStyle {
    fn to_tbstyle(&self) -> u8 {
        match self {
            ToolbarButtonStyle::Button => BTNS_BUTTON as u8,
            ToolbarButtonStyle::Separator => BTNS_SEP as u8,
            ToolbarButtonStyle::Check => BTNS_CHECK as u8,
            ToolbarButtonStyle::Dropdown => BTNS_DROPDOWN as u8,
            ToolbarButtonStyle::Group => BTNS_GROUP as u8,
            ToolbarButtonStyle::CheckGroup => BTNS_CHECKGROUP as u8,
            ToolbarButtonStyle::WholeDropdown => BTNS_WHOLEDROPDOWN as u8,
        }
    }
}

/// Represents a button to be inserted into a toolbar
#[derive(Clone, Debug)]
pub struct ToolbarButton {
    /// The command identifier for the button (used in events)
    pub id: i32,
    /// The image index in the toolbar's image list (-1 for no image)
    pub image_index: i32,
    /// The button style
    pub style: ToolbarButtonStyle,
    /// The button state (enabled, checked, etc.)
    pub enabled: bool,
    /// Text for the button (if LIST style is used)
    pub text: Option<String>,
}

impl Default for ToolbarButton {
    fn default() -> Self {
        ToolbarButton {
            id: 0,
            image_index: -1,
            style: ToolbarButtonStyle::Button,
            enabled: true,
            text: None,
        }
    }
}

impl ToolbarButton {
    pub fn new(id: i32) -> Self {
        ToolbarButton {
            id,
            ..Default::default()
        }
    }

    pub fn with_image(mut self, index: i32) -> Self {
        self.image_index = index;
        self
    }

    pub fn with_style(mut self, style: ToolbarButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/**
A toolbar is a control that contains one or more buttons. Each button can have an icon, text, or both.
Toolbars are typically placed at the top of a window below the menu bar.

Requires the `toolbar` feature.

**Builder parameters:**
  * `parent`:     **Required.** The toolbar parent container.
  * `size`:       The toolbar size (width is usually ignored as it auto-sizes).
  * `position`:   The toolbar position.
  * `enabled`:    If the toolbar is enabled.
  * `flags`:      Toolbar style flags.
  * `ex_flags`:   Extended window style flags.
  * `font`:       The font used for button text.
  * `buttons`:    Initial buttons to add.

**Control events:**
  * `OnToolbarClick`: When a toolbar button is clicked (event data contains button ID)
  * `OnToolbarDropDown`: When a dropdown button's arrow is clicked

```rust
use native_windows_gui as nwg;
fn build_toolbar(tb: &mut nwg::Toolbar, window: &nwg::Window) {
    nwg::Toolbar::builder()
        .parent(window)
        .build(tb);
}
```
*/
#[derive(Default)]
pub struct Toolbar {
    pub handle: ControlHandle,
    handler0: RefCell<Option<RawEventHandler>>,
}

impl Toolbar {
    pub fn builder<'a>() -> ToolbarBuilder<'a> {
        ToolbarBuilder {
            size: (0, 32),
            position: (0, 0),
            enabled: true,
            flags: None,
            ex_flags: 0,
            font: None,
            parent: None,
            buttons: Vec::new(),
            button_size: None,
        }
    }

    /// Add a button to the toolbar
    pub fn add_button(&self, button: ToolbarButton) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut tb_button: TBBUTTON = unsafe { std::mem::zeroed() };
        tb_button.iBitmap = button.image_index;
        tb_button.idCommand = button.id;
        tb_button.fsState = if button.enabled { TBSTATE_ENABLED as u8 } else { 0 };
        tb_button.fsStyle = button.style.to_tbstyle();

        // Handle button text
        if let Some(ref text) = button.text {
            let text_wide = to_utf16(text);
            tb_button.iString = unsafe {
                wh::send_message(handle, TB_ADDSTRINGW, 0, text_wide.as_ptr() as LPARAM) as isize
            };
        }

        unsafe {
            wh::send_message(handle, TB_BUTTONSTRUCTSIZE, std::mem::size_of::<TBBUTTON>() as WPARAM, 0);
            wh::send_message(handle, TB_ADDBUTTONSW, 1, &tb_button as *const TBBUTTON as LPARAM);
        }
    }

    /// Add multiple buttons to the toolbar
    pub fn add_buttons(&self, buttons: &[ToolbarButton]) {
        for button in buttons {
            self.add_button(button.clone());
        }
    }

    /// Add a separator to the toolbar
    pub fn add_separator(&self) {
        self.add_button(ToolbarButton {
            style: ToolbarButtonStyle::Separator,
            ..Default::default()
        });
    }

    /// Remove a button by its command ID
    pub fn remove_button(&self, id: i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let index = self.button_index(id);
        if index >= 0 {
            unsafe {
                wh::send_message(handle, TB_DELETEBUTTON, index as WPARAM, 0);
            }
        }
    }

    /// Get the index of a button by its command ID
    pub fn button_index(&self, id: i32) -> i32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, TB_COMMANDTOINDEX, id as WPARAM, 0) as i32
        }
    }

    /// Get the number of buttons in the toolbar
    pub fn button_count(&self) -> u32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, TB_BUTTONCOUNT, 0, 0) as u32
        }
    }

    /// Enable or disable a button by its command ID
    pub fn set_button_enabled(&self, id: i32, enabled: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, TB_ENABLEBUTTON, id as WPARAM, if enabled { TRUE as LPARAM } else { 0 });
        }
    }

    /// Check if a button is enabled
    pub fn button_enabled(&self, id: i32) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, TB_ISBUTTONENABLED, id as WPARAM, 0) != 0
        }
    }

    /// Set the checked state of a check button
    pub fn set_button_checked(&self, id: i32, checked: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, TB_CHECKBUTTON, id as WPARAM, if checked { TRUE as LPARAM } else { 0 });
        }
    }

    /// Get the checked state of a check button
    pub fn button_checked(&self, id: i32) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, TB_ISBUTTONCHECKED, id as WPARAM, 0) != 0
        }
    }

    /// Set the size of buttons in the toolbar
    pub fn set_button_size(&self, width: u16, height: u16) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let size = ((height as DWORD) << 16) | (width as DWORD);
        unsafe {
            wh::send_message(handle, TB_SETBUTTONSIZE, 0, size as LPARAM);
        }
    }

    /// Set the image list for the toolbar
    #[cfg(feature = "image-list")]
    pub fn set_image_list(&self, list: Option<&ImageList>) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let list_handle = list.map(|l| l.handle).unwrap_or(ptr::null_mut());
        unsafe {
            wh::send_message(handle, TB_SETIMAGELIST, 0, list_handle as LPARAM);
        }
    }

    /// Auto-size the toolbar to fit its buttons
    pub fn auto_size(&self) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, TB_AUTOSIZE, 0, 0);
        }
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

    /// Winapi class name
    pub fn class_name(&self) -> &'static str {
        TOOLBARCLASSNAME_S
    }

    /// Winapi flags
    pub fn flags(&self) -> u32 {
        WS_VISIBLE | TBSTYLE_FLAT | CCS_NODIVIDER
    }

    /// Required flags
    pub fn forced_flags(&self) -> u32 {
        WS_CHILD
    }

    /// Hook into parent resize to auto-size the toolbar
    fn hook_parent_resize(&self) {
        use crate::bind_raw_event_handler_inner;

        if self.handle.blank() { panic!("{}", NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        let parent_handle = ControlHandle::Hwnd(wh::get_window_parent(handle));
        let handler = bind_raw_event_handler_inner(&parent_handle, handle as usize, move |_hwnd, msg, _w, _l| {
            if msg == WM_SIZE {
                wh::send_message(handle, TB_AUTOSIZE, 0, 0);
            }
            None
        });

        *self.handler0.borrow_mut() = Some(handler.unwrap());
    }
}

impl Drop for Toolbar {
    fn drop(&mut self) {
        let handler = self.handler0.borrow();
        if let Some(h) = handler.as_ref() {
            drop(unbind_raw_event_handler(h));
        }
        self.handle.destroy();
    }
}

impl PartialEq for Toolbar {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

// String constant for toolbar class
const TOOLBARCLASSNAME_S: &'static str = "ToolbarWindow32";

pub struct ToolbarBuilder<'a> {
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    flags: Option<ToolbarFlags>,
    ex_flags: u32,
    font: Option<&'a Font>,
    parent: Option<ControlHandle>,
    buttons: Vec<ToolbarButton>,
    button_size: Option<(u16, u16)>,
}

impl<'a> ToolbarBuilder<'a> {
    pub fn size(mut self, size: impl Into<(i32, i32)>) -> ToolbarBuilder<'a> {
        self.size = size.into();
        self
    }

    pub fn position(mut self, position: impl Into<(i32, i32)>) -> ToolbarBuilder<'a> {
        self.position = position.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> ToolbarBuilder<'a> {
        self.enabled = enabled;
        self
    }

    pub fn flags(mut self, flags: ToolbarFlags) -> ToolbarBuilder<'a> {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, ex_flags: u32) -> ToolbarBuilder<'a> {
        self.ex_flags = ex_flags;
        self
    }

    pub fn font(mut self, font: Option<&'a Font>) -> ToolbarBuilder<'a> {
        self.font = font;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> ToolbarBuilder<'a> {
        self.parent = Some(p.into());
        self
    }

    pub fn buttons(mut self, buttons: Vec<ToolbarButton>) -> ToolbarBuilder<'a> {
        self.buttons = buttons;
        self
    }

    pub fn button_size(mut self, width: u16, height: u16) -> ToolbarBuilder<'a> {
        self.button_size = Some((width, height));
        self
    }

    pub fn build(self, out: &mut Toolbar) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("Toolbar"))
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
            .parent(Some(parent))
            .build()?;

        let handle = out.handle.hwnd().expect(BAD_HANDLE);

        // Initialize toolbar
        unsafe {
            wh::send_message(handle, TB_BUTTONSTRUCTSIZE, std::mem::size_of::<TBBUTTON>() as WPARAM, 0);
        }

        // Set button size if specified
        if let Some((w, h)) = self.button_size {
            out.set_button_size(w, h);
        }

        // Set font
        if self.font.is_some() {
            out.set_font(self.font);
        } else {
            out.set_font(Font::global_default().as_ref());
        }

        // Add initial buttons
        for button in self.buttons {
            out.add_button(button);
        }

        // Auto-size after adding buttons
        out.auto_size();

        // Hook parent resize
        out.hook_parent_resize();

        if !self.enabled {
            out.set_enabled(false);
        }

        Ok(())
    }
}
