use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{LPARAM, WPARAM, TRUE, FALSE};
use winapi::um::winuser::{WS_VISIBLE, WS_DISABLED, WS_TABSTOP, WS_CHILD, WS_VSCROLL};
use winapi::um::winuser::{CB_GETCURSEL, CB_SETCURSEL, CB_ERR, CBS_DROPDOWNLIST, CBS_DROPDOWN, CBS_SIMPLE};
use winapi::um::commctrl::*;
use crate::win32::base_helper::{check_hwnd, to_utf16};
use crate::win32::window_helper as wh;
use crate::{Font, NwgError};
use super::{ControlHandle, ControlBase};
use std::mem;
use std::ptr;

const NOT_BOUND: &'static str = "ComboBoxEx is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: ComboBoxEx handle is not HWND!";

bitflags! {
    /**
        The ComboBoxEx flags

        * VISIBLE:  The control is visible after creation
        * DISABLED: The control cannot be interacted with by the user
        * TAB_STOP: The control can be selected using tab navigation
        * DROPDOWN: A dropdown list (editable text field with dropdown)
        * DROPDOWN_LIST: A dropdown list (non-editable, selection only)
        * SIMPLE: Always shows the list
    */
    pub struct ComboBoxExFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        const TAB_STOP = WS_TABSTOP;
        const VSCROLL = WS_VSCROLL;
        const DROPDOWN = CBS_DROPDOWN;
        const DROPDOWN_LIST = CBS_DROPDOWNLIST;
        const SIMPLE = CBS_SIMPLE;
    }
}

bitflags! {
    /**
        Extended styles for ComboBoxEx

        * NOEDITIMAGE: The edit box and dropdown list will not display item images
        * NOEDITIMAGEINDENT: The edit box will not indent text for images
        * PATHWORDBREAKPROC: Use path handling for word breaks
        * NOSIZELIMIT: Allows the dropdown to be larger than the control
        * CASESENSITIVE: Searches are case-sensitive
        * TEXTENDELLIPSIS: Truncates text with ellipsis
    */
    pub struct ComboBoxExStyleEx: u32 {
        const NOEDITIMAGE = CBES_EX_NOEDITIMAGE;
        const NOEDITIMAGEINDENT = CBES_EX_NOEDITIMAGEINDENT;
        const PATHWORDBREAKPROC = CBES_EX_PATHWORDBREAKPROC;
        const NOSIZELIMIT = CBES_EX_NOSIZELIMIT;
        const CASESENSITIVE = CBES_EX_CASESENSITIVE;
        const TEXTENDELLIPSIS = CBES_EX_TEXTENDELLIPSIS;
    }
}

/// Represents an item to be inserted into a ComboBoxEx control
pub struct ComboBoxExItem<'a> {
    /// The text of the item
    pub text: &'a str,
    /// Index of the image in the image list (or -1 for no image)
    pub image: i32,
    /// Index of the selected image (or -1 to use same as image)
    pub selected_image: i32,
    /// Indentation level (number of image widths)
    pub indent: i32,
}

impl<'a> ComboBoxExItem<'a> {
    /// Create a simple text-only item
    pub fn new(text: &'a str) -> Self {
        ComboBoxExItem {
            text,
            image: -1,
            selected_image: -1,
            indent: 0,
        }
    }

    /// Create an item with an image
    pub fn with_image(text: &'a str, image: i32) -> Self {
        ComboBoxExItem {
            text,
            image,
            selected_image: image,
            indent: 0,
        }
    }

    /// Set the selected image index
    pub fn selected_image(mut self, index: i32) -> Self {
        self.selected_image = index;
        self
    }

    /// Set the indentation level
    pub fn indent(mut self, indent: i32) -> Self {
        self.indent = indent;
        self
    }
}

/**
A ComboBoxEx control is an extension of the ComboBox control that supports
images for each item via an image list.

Requires the `combobox-ex` feature.

**Builder parameters:**
  * `parent`:     **Required.** The control parent container.
  * `size`:       The control size.
  * `position`:   The control position.
  * `enabled`:    If the control is enabled.
  * `flags`:      ComboBoxEx style flags.
  * `ex_flags`:   Extended window style flags.
  * `style_ex`:   ComboBoxEx extended styles.
  * `font`:       The font used for the control.
  * `focus`:      The control receives focus after being created.

**Control events:**
  * `OnComboBoxClosed`: When the dropdown is closed
  * `OnComboBoxDropdown`: When the dropdown is opened
  * `OnComboxBoxSelection`: When a selection changes

```rust
use native_windows_gui as nwg;

fn build_combobox_ex(combo: &mut nwg::ComboBoxEx, window: &nwg::Window, image_list: &nwg::ImageList) {
    nwg::ComboBoxEx::builder()
        .size((200, 300))
        .parent(window)
        .build(combo);

    // Set image list
    combo.set_image_list(Some(image_list));

    // Add items
    combo.insert_item(0, &nwg::ComboBoxExItem::with_image("Item 1", 0));
    combo.insert_item(1, &nwg::ComboBoxExItem::with_image("Item 2", 1));
}
```
*/
#[derive(Default)]
pub struct ComboBoxEx {
    pub handle: ControlHandle,
}

impl ComboBoxEx {
    pub fn builder() -> ComboBoxExBuilder {
        ComboBoxExBuilder {
            size: (100, 200),
            position: (0, 0),
            enabled: true,
            focus: false,
            flags: None,
            ex_flags: 0,
            style_ex: None,
            font: None,
            parent: None,
        }
    }

    /// Set the image list for the control
    #[cfg(feature = "image-list")]
    pub fn set_image_list(&self, image_list: Option<&crate::ImageList>) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let himl = image_list.map(|il| il.handle).unwrap_or(ptr::null_mut());
        wh::send_message(handle, CBEM_SETIMAGELIST, 0, himl as LPARAM);
    }

    /// Get the handle to the child combo box control
    pub fn combo_handle(&self) -> HWND {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, CBEM_GETCOMBOCONTROL, 0, 0) as HWND
    }

    /// Get the handle to the edit control (if the combo has one)
    pub fn edit_handle(&self) -> Option<HWND> {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let edit = wh::send_message(handle, CBEM_GETEDITCONTROL, 0, 0) as HWND;
        if edit.is_null() { None } else { Some(edit) }
    }

    /// Insert an item at the specified index
    pub fn insert_item(&self, index: usize, item: &ComboBoxExItem) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let text_wide = to_utf16(item.text);

        let mut cbei: COMBOBOXEXITEMW = unsafe { mem::zeroed() };
        cbei.mask = CBEIF_TEXT | CBEIF_IMAGE | CBEIF_SELECTEDIMAGE | CBEIF_INDENT;
        cbei.iItem = index as isize;
        cbei.pszText = text_wide.as_ptr() as *mut _;
        cbei.iImage = item.image;
        cbei.iSelectedImage = if item.selected_image >= 0 { item.selected_image } else { item.image };
        cbei.iIndent = item.indent;

        let result = wh::send_message(handle, CBEM_INSERTITEMW, 0, &cbei as *const _ as LPARAM);
        result != -1
    }

    /// Add an item at the end of the list
    pub fn push_item(&self, item: &ComboBoxExItem) -> bool {
        self.insert_item(usize::MAX, item)
    }

    /// Remove an item at the specified index
    pub fn remove_item(&self, index: usize) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let result = wh::send_message(handle, CBEM_DELETEITEM, index as WPARAM, 0);
        result != CB_ERR
    }

    /// Clear all items
    pub fn clear(&self) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        // Remove items from the end to avoid index shifting issues
        while wh::send_message(handle, CBEM_DELETEITEM, 0, 0) != CB_ERR {}
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        use winapi::um::winuser::CB_GETCOUNT;
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let combo = self.combo_handle();
        let count = wh::send_message(combo, CB_GETCOUNT, 0, 0);
        if count == CB_ERR { 0 } else { count as usize }
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the index of the currently selected item, or None if no selection
    pub fn selection(&self) -> Option<usize> {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let combo = self.combo_handle();
        let index = wh::send_message(combo, CB_GETCURSEL, 0, 0);
        if index == CB_ERR { None } else { Some(index as usize) }
    }

    /// Set the current selection by index. Pass None to clear selection.
    pub fn set_selection(&self, index: Option<usize>) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let combo = self.combo_handle();
        let i = index.map(|i| i as isize).unwrap_or(-1);
        wh::send_message(combo, CB_SETCURSEL, i as WPARAM, 0);
    }

    /// Show or hide the dropdown
    pub fn dropdown(&self, show: bool) {
        use winapi::um::winuser::CB_SHOWDROPDOWN;
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let combo = self.combo_handle();
        wh::send_message(combo, CB_SHOWDROPDOWN, show as WPARAM, 0);
    }

    /// Check if the edit text has been changed
    pub fn has_edit_changed(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, CBEM_HASEDITCHANGED, 0, 0) != 0
    }

    /// Set the extended style of the ComboBoxEx
    pub fn set_style_ex(&self, style: ComboBoxExStyleEx) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, CBEM_SETEXTENDEDSTYLE, 0, style.bits() as LPARAM);
    }

    /// Get the extended style of the ComboBoxEx
    pub fn style_ex(&self) -> ComboBoxExStyleEx {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let bits = wh::send_message(handle, CBEM_GETEXTENDEDSTYLE, 0, 0) as u32;
        ComboBoxExStyleEx::from_bits_truncate(bits)
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
        unsafe { winapi::um::winuser::IsWindowVisible(handle) != 0 }
    }

    /// Show or hide the control
    pub fn set_visible(&self, visible: bool) {
        use winapi::um::winuser::{ShowWindow, SW_SHOW, SW_HIDE};
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { ShowWindow(handle, if visible { SW_SHOW } else { SW_HIDE }); }
    }

    /// Return true if the control is enabled
    pub fn enabled(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { winapi::um::winuser::IsWindowEnabled(handle) != 0 }
    }

    /// Enable or disable the control
    pub fn set_enabled(&self, enabled: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { winapi::um::winuser::EnableWindow(handle, if enabled { TRUE } else { FALSE }); }
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
        unsafe { winapi::um::winuser::SetFocus(handle); }
    }

    /// Winapi class name
    pub fn class_name(&self) -> &'static str {
        WC_COMBOBOXEX
    }

    /// Winapi flags
    pub fn flags(&self) -> u32 {
        WS_VISIBLE | CBS_DROPDOWN
    }

    /// Required flags
    pub fn forced_flags(&self) -> u32 {
        WS_CHILD
    }
}

impl Drop for ComboBoxEx {
    fn drop(&mut self) {
        self.handle.destroy();
    }
}

impl PartialEq for ComboBoxEx {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

pub struct ComboBoxExBuilder {
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    focus: bool,
    flags: Option<ComboBoxExFlags>,
    ex_flags: u32,
    style_ex: Option<ComboBoxExStyleEx>,
    font: Option<Font>,
    parent: Option<ControlHandle>,
}

impl ComboBoxExBuilder {
    pub fn size(mut self, size: impl Into<(i32, i32)>) -> ComboBoxExBuilder {
        self.size = size.into();
        self
    }

    pub fn position(mut self, position: impl Into<(i32, i32)>) -> ComboBoxExBuilder {
        self.position = position.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> ComboBoxExBuilder {
        self.enabled = enabled;
        self
    }

    pub fn focus(mut self, focus: bool) -> ComboBoxExBuilder {
        self.focus = focus;
        self
    }

    pub fn flags(mut self, flags: ComboBoxExFlags) -> ComboBoxExBuilder {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, ex_flags: u32) -> ComboBoxExBuilder {
        self.ex_flags = ex_flags;
        self
    }

    pub fn style_ex(mut self, style_ex: ComboBoxExStyleEx) -> ComboBoxExBuilder {
        self.style_ex = Some(style_ex);
        self
    }

    pub fn font(mut self, font: Option<&Font>) -> ComboBoxExBuilder {
        self.font = font.map(|f| Font { handle: f.handle });
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> ComboBoxExBuilder {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut ComboBoxEx) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("ComboBoxEx"))
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

        // Set extended style if specified
        if let Some(style_ex) = self.style_ex {
            out.set_style_ex(style_ex);
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
