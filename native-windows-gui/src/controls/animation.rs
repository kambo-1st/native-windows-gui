use winapi::shared::minwindef::{WPARAM, LPARAM, TRUE, FALSE};
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use crate::win32::window_helper as wh;
use crate::win32::base_helper::{check_hwnd, to_utf16};
use crate::{Font, NwgError};
use super::{ControlHandle, ControlBase};

const NOT_BOUND: &'static str = "Animation is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: Animation handle is not HWND!";

bitflags! {
    /// Animation style flags
    pub struct AnimationFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        /// Centers the animation in the control window
        const CENTER = ACS_CENTER;
        /// Transparent background - uses parent window background
        const TRANSPARENT = ACS_TRANSPARENT;
        /// Automatically starts playing when opened
        const AUTOPLAY = ACS_AUTOPLAY;
        /// Uses a timer to play instead of creating a thread (for compatibility)
        const TIMER = ACS_TIMER;
    }
}

/**
An Animation control displays Audio-Video Interleaved (AVI) clips.
This is commonly used for showing simple animations during operations,
like the file copy animation in Windows Explorer.

Note: The AVI clip must be uncompressed or RLE-compressed. The control
does not support audio - only silent AVI clips are supported.

Requires the `animation` feature.

**Builder parameters:**
  * `parent`:     **Required.** The animation parent container.
  * `size`:       The animation size.
  * `position`:   The animation position.
  * `enabled`:    If the animation is enabled.
  * `flags`:      Animation style flags.
  * `ex_flags`:   Extended window style flags.

**Control events:**
  * `OnAnimationStart`: When the animation starts playing
  * `OnAnimationStop`: When the animation stops playing

```rust
use native_windows_gui as nwg;
fn build_animation(anim: &mut nwg::Animation, window: &nwg::Window) {
    nwg::Animation::builder()
        .size((100, 100))
        .position((10, 10))
        .parent(window)
        .build(anim);
}
```
*/
#[derive(Default)]
pub struct Animation {
    pub handle: ControlHandle,
}

impl Animation {
    pub fn builder() -> AnimationBuilder {
        AnimationBuilder {
            size: (100, 100),
            position: (0, 0),
            enabled: true,
            flags: None,
            ex_flags: 0,
            parent: None,
        }
    }

    /// Opens an AVI clip from a file path
    pub fn open_file(&self, path: &str) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let path_wide = to_utf16(path);
        let result = wh::send_message(handle, ACM_OPENW, 0, path_wide.as_ptr() as LPARAM);
        result != 0
    }

    /// Opens an AVI clip from a resource ID in the application's executable.
    /// Pass the resource ID as an integer.
    pub fn open_resource(&self, resource_id: u16) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let result = wh::send_message(
            handle,
            ACM_OPENW,
            0, // hInstance = NULL means use the application's instance
            resource_id as LPARAM
        );
        result != 0
    }

    /// Opens an AVI clip from a resource ID in a specific module (DLL).
    /// The module_handle should be obtained from LoadLibrary or similar.
    pub fn open_resource_from_module(&self, module_handle: isize, resource_id: u16) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let result = wh::send_message(
            handle,
            ACM_OPENW,
            module_handle as WPARAM,
            resource_id as LPARAM
        );
        result != 0
    }

    /// Closes the currently open AVI clip and frees resources
    pub fn close(&self) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, ACM_OPENW, 0, 0);
    }

    /// Plays the animation.
    /// - `from`: Starting frame (0 for first frame)
    /// - `to`: Ending frame (-1 for last frame)
    /// - `repeat`: Number of times to repeat (-1 for infinite)
    pub fn play(&self, from: i32, to: i32, repeat: i32) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let wparam = ((repeat as u32) << 16) | ((from as u32) & 0xFFFF);
        let result = wh::send_message(handle, ACM_PLAY, wparam as WPARAM, to as LPARAM);
        result != 0
    }

    /// Plays the entire animation once
    pub fn play_once(&self) -> bool {
        self.play(0, -1, 1)
    }

    /// Plays the entire animation in a loop (infinite repeat)
    pub fn play_loop(&self) -> bool {
        self.play(0, -1, -1)
    }

    /// Stops the animation
    pub fn stop(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let result = wh::send_message(handle, ACM_STOP, 0, 0);
        result != 0
    }

    /// Returns true if the animation is currently playing
    pub fn is_playing(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let result = wh::send_message(handle, ACM_ISPLAYING, 0, 0);
        result != 0
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

    /// Winapi class name
    pub fn class_name(&self) -> &'static str {
        ANIMATE_CLASS
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

impl Drop for Animation {
    fn drop(&mut self) {
        self.handle.destroy();
    }
}

impl PartialEq for Animation {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

pub struct AnimationBuilder {
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    flags: Option<AnimationFlags>,
    ex_flags: u32,
    parent: Option<ControlHandle>,
}

impl<'a> AnimationBuilder {
    pub fn size(mut self, size: impl Into<(i32, i32)>) -> AnimationBuilder {
        self.size = size.into();
        self
    }

    pub fn position(mut self, position: impl Into<(i32, i32)>) -> AnimationBuilder {
        self.position = position.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> AnimationBuilder {
        self.enabled = enabled;
        self
    }

    pub fn flags(mut self, flags: AnimationFlags) -> AnimationBuilder {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, ex_flags: u32) -> AnimationBuilder {
        self.ex_flags = ex_flags;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> AnimationBuilder {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut Animation) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("Animation"))
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
