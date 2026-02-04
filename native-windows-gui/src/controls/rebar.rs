use winapi::shared::minwindef::{WPARAM, LPARAM, UINT, TRUE};
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use crate::win32::window_helper as wh;
use crate::win32::base_helper::{check_hwnd, to_utf16};
use crate::{Font, NwgError, RawEventHandler, unbind_raw_event_handler};
use super::{ControlHandle, ControlBase};
use std::cell::RefCell;
use std::mem;

const NOT_BOUND: &'static str = "Rebar is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: Rebar handle is not HWND!";

bitflags! {
    /// Rebar style flags
    pub struct RebarFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        /// Rebar allows bands to have variable height
        const VAR_HEIGHT = RBS_VARHEIGHT;
        /// Rebar displays narrow lines to separate bands
        const BAND_BORDERS = RBS_BANDBORDERS;
        /// Bands cannot be rearranged
        const FIXED_ORDER = RBS_FIXEDORDER;
        /// Rebar displays tooltips
        const TOOLTIPS = RBS_TOOLTIPS;
        /// Allows rebar to be resized by dragging
        const AUTOSIZE = RBS_AUTOSIZE;
        /// Displays vertical gripper bars on bands
        const VERTICAL_GRIPPER = RBS_VERTICALGRIPPER;
        /// Bands are displayed on multiple rows
        const DBLCLKTOGGLE = RBS_DBLCLKTOGGLE;
        /// No divider above the rebar
        const NODIVIDER = CCS_NODIVIDER;
        /// Rebar does not resize with parent
        const NORESIZE = CCS_NORESIZE;
        /// Rebar does not align to parent edge
        const NOPARENTALIGN = CCS_NOPARENTALIGN;
        /// Vertical rebar
        const VERT = CCS_VERT;
    }
}

bitflags! {
    /// Rebar band style flags
    pub struct RebarBandFlags: u32 {
        /// Band has a background color
        const BACKGROUND = RBBS_NOVERT;
        /// Band has a child window
        const CHILD = RBBS_CHILDEDGE;
        /// Band has fixed size (no gripper)
        const FIXED_SIZE = RBBS_FIXEDSIZE;
        /// Band has gripper
        const GRIPPER = RBBS_GRIPPERALWAYS;
        /// Band is hidden
        const HIDDEN = RBBS_HIDDEN;
        /// Band can have a break
        const BREAK = RBBS_BREAK;
        /// Band has top edge padding
        const TOP_ALIGN = RBBS_TOPALIGN;
        /// Band is not displayed in vertical orientation
        const NO_VERT = RBBS_NOVERT;
        /// Band uses chevron button for overflow
        const USE_CHEVRON = RBBS_USECHEVRON;
        /// Band always has a break before it
        const VARIABLE_HEIGHT = RBBS_VARIABLEHEIGHT;
    }
}

/// Configuration for a rebar band
#[derive(Clone, Debug)]
pub struct RebarBand {
    /// Optional text for the band
    pub text: Option<String>,
    /// The minimum width of the band
    pub min_width: u32,
    /// The minimum height of the band
    pub min_height: u32,
    /// The initial width of the band
    pub width: u32,
    /// The child control handle to embed in the band
    pub child: Option<ControlHandle>,
    /// Band style flags
    pub flags: RebarBandFlags,
    /// Image index for the band (-1 for no image)
    pub image_index: i32,
    /// Band ID
    pub id: u32,
}

impl Default for RebarBand {
    fn default() -> Self {
        RebarBand {
            text: None,
            min_width: 0,
            min_height: 24,
            width: 100,
            child: None,
            flags: RebarBandFlags::GRIPPER | RebarBandFlags::CHILD,
            image_index: -1,
            id: 0,
        }
    }
}

impl RebarBand {
    /// Create a new rebar band with the given ID
    pub fn new(id: u32) -> Self {
        RebarBand {
            id,
            ..Default::default()
        }
    }

    /// Set the text for the band
    pub fn with_text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    /// Set the minimum width
    pub fn with_min_width(mut self, width: u32) -> Self {
        self.min_width = width;
        self
    }

    /// Set the minimum height
    pub fn with_min_height(mut self, height: u32) -> Self {
        self.min_height = height;
        self
    }

    /// Set the initial width
    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Set the child control to embed in this band
    pub fn with_child<C: Into<ControlHandle>>(mut self, child: C) -> Self {
        self.child = Some(child.into());
        self
    }

    /// Set the band flags
    pub fn with_flags(mut self, flags: RebarBandFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set the image index
    pub fn with_image(mut self, index: i32) -> Self {
        self.image_index = index;
        self
    }
}

/**
A rebar control is a container for toolbar bands. It provides a way to organize
toolbars and other controls into resizable, movable bands with optional grippers.

Rebar controls are typically placed at the top of a window below the menu bar
and contain toolbars, combo boxes, or other controls.

Requires the `rebar` feature.

**Builder parameters:**
  * `parent`:     **Required.** The rebar parent container.
  * `size`:       The rebar size (width is usually ignored as it auto-sizes).
  * `position`:   The rebar position.
  * `enabled`:    If the rebar is enabled.
  * `flags`:      Rebar style flags.
  * `ex_flags`:   Extended window style flags.
  * `bands`:      Initial bands to add.

**Control events:**
  * `OnRebarHeightChange`: When the rebar height changes
  * `OnRebarLayoutChanged`: When the rebar layout changes

```rust
use native_windows_gui as nwg;
fn build_rebar(rb: &mut nwg::Rebar, window: &nwg::Window, toolbar: &nwg::Toolbar) {
    nwg::Rebar::builder()
        .parent(window)
        .build(rb);

    rb.add_band(nwg::RebarBand::new(1).with_child(toolbar).with_min_width(100));
}
```
*/
#[derive(Default)]
pub struct Rebar {
    pub handle: ControlHandle,
    handler0: RefCell<Option<RawEventHandler>>,
}

impl Rebar {
    pub fn builder<'a>() -> RebarBuilder<'a> {
        RebarBuilder {
            size: (0, 32),
            position: (0, 0),
            enabled: true,
            flags: None,
            ex_flags: 0,
            font: None,
            parent: None,
            bands: Vec::new(),
        }
    }

    /// Add a band to the rebar (appends to the end)
    pub fn add_band(&self, band: RebarBand) {
        self.insert_band(-1, band);
    }

    /// Insert a band at a specific index (-1 to append)
    pub fn insert_band(&self, index: i32, band: RebarBand) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut rbbi: REBARBANDINFOW = unsafe { mem::zeroed() };
        // cbSize must be 80 bytes for compatibility with common controls
        // The full REBARBANDINFOW struct in winapi is larger (128 bytes) which causes
        // RB_INSERTBAND to fail with ERROR_INVALID_HANDLE
        rbbi.cbSize = 80;
        rbbi.fMask = RBBIM_STYLE | RBBIM_ID | RBBIM_SIZE | RBBIM_CHILDSIZE;
        rbbi.fStyle = band.flags.bits();
        rbbi.wID = band.id;
        rbbi.cx = band.width;
        rbbi.cxMinChild = band.min_width;
        rbbi.cyMinChild = band.min_height;
        rbbi.cyChild = band.min_height; // Set initial child height

        let mut text_wide: Vec<u16> = Vec::new();
        if let Some(ref text) = band.text {
            text_wide = to_utf16(text);
            rbbi.fMask |= RBBIM_TEXT;
            rbbi.lpText = text_wide.as_ptr() as *mut u16;
            rbbi.cch = (text.len() + 1) as UINT;
        }

        if let Some(child) = band.child {
            if let Some(child_hwnd) = child.hwnd() {
                rbbi.fMask |= RBBIM_CHILD;
                rbbi.hwndChild = child_hwnd;

                // Get the child window size for proper sizing
                let (child_width, child_height) = unsafe { wh::get_window_size(child_hwnd) };
                if child_width > 0 && rbbi.cxMinChild == 0 {
                    rbbi.cxMinChild = child_width as u32;
                }
                if child_height > 0 && (rbbi.cyMinChild == 0 || rbbi.cyMinChild < child_height as u32) {
                    rbbi.cyMinChild = child_height as u32;
                    rbbi.cyChild = child_height as u32;
                }
            }
        }

        if band.image_index >= 0 {
            rbbi.fMask |= RBBIM_IMAGE;
            rbbi.iImage = band.image_index;
        }

        // Use -1 to append, otherwise use the provided index
        let insert_index = if index < 0 { -1i32 as WPARAM } else { index as WPARAM };

        wh::send_message(handle, RB_INSERTBANDW, insert_index, &rbbi as *const REBARBANDINFOW as LPARAM);
    }

    /// Remove a band by index
    pub fn remove_band(&self, index: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_DELETEBAND, index as WPARAM, 0);
        }
    }

    /// Get the number of bands in the rebar
    pub fn band_count(&self) -> u32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_GETBANDCOUNT, 0, 0) as u32
        }
    }

    /// Get the total height of the rebar
    pub fn height(&self) -> u32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_GETBARHEIGHT, 0, 0) as u32
        }
    }

    /// Get the number of rows in the rebar
    pub fn row_count(&self) -> u32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_GETROWCOUNT, 0, 0) as u32
        }
    }

    /// Get the height of a specific row
    pub fn row_height(&self, row: u32) -> u32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_GETROWHEIGHT, row as WPARAM, 0) as u32
        }
    }

    /// Show or hide a band
    pub fn set_band_visible(&self, index: u32, visible: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_SHOWBAND, index as WPARAM, if visible { TRUE as LPARAM } else { 0 });
        }
    }

    /// Minimize a band to its title bar only
    pub fn minimize_band(&self, index: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_MINIMIZEBAND, index as WPARAM, 0);
        }
    }

    /// Maximize a band to its ideal size
    pub fn maximize_band(&self, index: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_MAXIMIZEBAND, index as WPARAM, TRUE as LPARAM);
        }
    }

    /// Move a band to a new position
    pub fn move_band(&self, from_index: u32, to_index: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_MOVEBAND, from_index as WPARAM, to_index as LPARAM);
        }
    }

    /// Get the index of the band with the specified ID
    pub fn id_to_index(&self, id: u32) -> Option<u32> {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let result = unsafe {
            wh::send_message(handle, RB_IDTOINDEX, id as WPARAM, 0) as i32
        };
        if result < 0 {
            None
        } else {
            Some(result as u32)
        }
    }

    /// Set the width of a band
    pub fn set_band_width(&self, index: u32, width: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut rbbi: REBARBANDINFOW = unsafe { mem::zeroed() };
        rbbi.cbSize = mem::size_of::<REBARBANDINFOW>() as UINT;
        rbbi.fMask = RBBIM_SIZE;
        rbbi.cx = width;

        unsafe {
            wh::send_message(handle, RB_SETBANDINFOW, index as WPARAM, &rbbi as *const REBARBANDINFOW as LPARAM);
        }
    }

    /// Begin a drag operation on a band
    pub fn begin_drag(&self, index: u32, pos: (i32, i32)) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let lparam = ((pos.1 as u32) << 16) | (pos.0 as u32 & 0xFFFF);
        unsafe {
            wh::send_message(handle, RB_BEGINDRAG, index as WPARAM, lparam as LPARAM);
        }
    }

    /// End a drag operation
    pub fn end_drag(&self) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::send_message(handle, RB_ENDDRAG, 0, 0);
        }
    }

    /// Update drag position
    pub fn drag_move(&self, pos: (i32, i32)) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let lparam = ((pos.1 as u32) << 16) | (pos.0 as u32 & 0xFFFF);
        unsafe {
            wh::send_message(handle, RB_DRAGMOVE, 0, lparam as LPARAM);
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
        REBARCLASSNAME_S
    }

    /// Winapi flags
    pub fn flags(&self) -> u32 {
        WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | CCS_NODIVIDER | CCS_TOP | RBS_VARHEIGHT | RBS_BANDBORDERS
    }

    /// Required flags
    pub fn forced_flags(&self) -> u32 {
        WS_CHILD
    }

    /// Hook into parent resize to auto-size the rebar
    fn hook_parent_resize(&self) {
        use crate::bind_raw_event_handler_inner;

        if self.handle.blank() { panic!("{}", NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        let parent_handle = ControlHandle::Hwnd(wh::get_window_parent(handle));
        let handler = bind_raw_event_handler_inner(&parent_handle, handle as usize, move |_hwnd, msg, _w, _l| {
            if msg == WM_SIZE {
                // Get parent size and resize rebar to match width
                unsafe {
                    let (width, _) = wh::get_window_size(_hwnd);
                    wh::set_window_size(handle, width as u32, 0, false);
                }
            }
            None
        });

        *self.handler0.borrow_mut() = Some(handler.unwrap());
    }
}

impl Drop for Rebar {
    fn drop(&mut self) {
        let handler = self.handler0.borrow();
        if let Some(h) = handler.as_ref() {
            drop(unbind_raw_event_handler(h));
        }
        self.handle.destroy();
    }
}

impl PartialEq for Rebar {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

// String constant for rebar class
const REBARCLASSNAME_S: &'static str = "ReBarWindow32";

pub struct RebarBuilder<'a> {
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    flags: Option<RebarFlags>,
    ex_flags: u32,
    font: Option<&'a Font>,
    parent: Option<ControlHandle>,
    bands: Vec<RebarBand>,
}

impl<'a> RebarBuilder<'a> {
    pub fn size(mut self, size: impl Into<(i32, i32)>) -> RebarBuilder<'a> {
        self.size = size.into();
        self
    }

    pub fn position(mut self, position: impl Into<(i32, i32)>) -> RebarBuilder<'a> {
        self.position = position.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> RebarBuilder<'a> {
        self.enabled = enabled;
        self
    }

    pub fn flags(mut self, flags: RebarFlags) -> RebarBuilder<'a> {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, ex_flags: u32) -> RebarBuilder<'a> {
        self.ex_flags = ex_flags;
        self
    }

    pub fn font(mut self, font: Option<&'a Font>) -> RebarBuilder<'a> {
        self.font = font;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> RebarBuilder<'a> {
        self.parent = Some(p.into());
        self
    }

    pub fn bands(mut self, bands: Vec<RebarBand>) -> RebarBuilder<'a> {
        self.bands = bands;
        self
    }

    pub fn build(self, out: &mut Rebar) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("Rebar"))
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

        let rebar_handle = out.handle.hwnd().expect(BAD_HANDLE);

        // Initialize rebar with RB_SETBARINFO (required before inserting bands)
        #[repr(C)]
        struct REBARINFO {
            cb_size: u32,
            f_mask: u32,
            himl: *mut std::ffi::c_void,
        }

        let rbi = REBARINFO {
            cb_size: std::mem::size_of::<REBARINFO>() as u32,
            f_mask: 0, // No image list
            himl: std::ptr::null_mut(),
        };

        wh::send_message(rebar_handle, RB_SETBARINFO, 0, &rbi as *const REBARINFO as LPARAM);

        // Set font
        if self.font.is_some() {
            out.set_font(self.font);
        } else {
            out.set_font(Font::global_default().as_ref());
        }

        // Add initial bands
        for band in self.bands {
            out.add_band(band);
        }

        // Hook parent resize
        out.hook_parent_resize();

        if !self.enabled {
            out.set_enabled(false);
        }

        Ok(())
    }
}
