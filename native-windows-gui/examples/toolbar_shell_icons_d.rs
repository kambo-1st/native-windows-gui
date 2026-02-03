/*!
    A toolbar example with proper 16x16 icons from shell32.dll.

    This example demonstrates how to load standard Windows shell icons
    at the correct 16x16 size for toolbars.
*/

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
extern crate winapi;

use nwd::NwgUi;
use nwg::NativeUi;
use winapi::um::shellapi::ExtractIconExW;
use winapi::um::commctrl::ImageList_ReplaceIcon;
use winapi::um::winuser::DestroyIcon;
use winapi::shared::windef::HICON;
use std::ptr;

// Shell32.dll icon indices for common actions
mod shell_icons {
    pub const NEW_FILE: i32 = 0;           // New/Unknown file
    pub const FOLDER_CLOSED: i32 = 3;      // Closed folder
    pub const FOLDER_OPEN: i32 = 4;        // Open folder
    pub const FLOPPY: i32 = 6;             // Floppy/Save
    pub const PRINT: i32 = 16;             // Printer
    pub const FIND: i32 = 22;              // Find/Search
    pub const HELP: i32 = 23;              // Help
    pub const DELETE: i32 = 31;            // Delete/Recycle
    pub const INFO: i32 = 221;             // Information
    pub const CUT: i32 = 260;              // Cut (scissors)
    pub const COPY: i32 = 261;             // Copy
    pub const PASTE: i32 = 262;            // Paste (clipboard)
    pub const UNDO: i32 = 263;             // Undo
    pub const REDO: i32 = 264;             // Redo
}

/// Convert a Rust string to null-terminated UTF-16
fn to_utf16(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Load a 16x16 icon from shell32.dll and add it directly to an ImageList
fn add_shell_icon_to_imagelist(image_list: &nwg::ImageList, icon_index: i32) -> bool {
    let shell32 = to_utf16("shell32.dll");

    let mut large_icon: HICON = ptr::null_mut();
    let mut small_icon: HICON = ptr::null_mut();

    let count = unsafe {
        ExtractIconExW(
            shell32.as_ptr(),
            icon_index,
            &mut large_icon,
            &mut small_icon,
            1
        )
    };

    if count == 0 || small_icon.is_null() {
        return false;
    }

    // Add the small (16x16) icon directly to the image list
    let result = unsafe {
        ImageList_ReplaceIcon(image_list.handle as _, -1, small_icon)
    };

    // Clean up the icons (ImageList makes a copy)
    unsafe {
        if !small_icon.is_null() { DestroyIcon(small_icon); }
        if !large_icon.is_null() { DestroyIcon(large_icon); }
    }

    result >= 0
}

#[derive(Default, NwgUi)]
pub struct ToolbarShellIconsApp {
    #[nwg_control(size: (700, 400), position: (300, 300), title: "Toolbar with 16x16 Shell Icons", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [ToolbarShellIconsApp::exit] )]
    window: nwg::Window,

    // 16x16 image list - matches shell small icons exactly
    #[nwg_resource(size: (16, 16))]
    toolbar_image_list: nwg::ImageList,

    #[nwg_control(parent: window, flags: "VISIBLE|FLAT|NODIVIDER")]
    toolbar: nwg::Toolbar,

    #[nwg_control(size: (680, 300), position: (10, 45), text: "")]
    status_label: nwg::Label,
}

impl ToolbarShellIconsApp {
    fn setup_toolbar(&self) {
        use nwg::{ToolbarButton, ToolbarButtonStyle};
        use shell_icons::*;

        // Load 16x16 icons from shell32.dll into the image list
        // The order here determines the image index (0, 1, 2, ...)
        let icons = [
            NEW_FILE,     // 0: New
            FOLDER_OPEN,  // 1: Open
            FLOPPY,       // 2: Save
            CUT,          // 3: Cut
            COPY,         // 4: Copy
            PASTE,        // 5: Paste
            UNDO,         // 6: Undo
            DELETE,       // 7: Delete
            FIND,         // 8: Find
            HELP,         // 9: Help
        ];

        for icon_index in &icons {
            if !add_shell_icon_to_imagelist(&self.toolbar_image_list, *icon_index) {
                eprintln!("Failed to load shell icon {}", icon_index);
            }
        }

        // Set the image list on the toolbar
        self.toolbar.set_image_list(Some(&self.toolbar_image_list));

        // File operations group
        self.toolbar.add_button(ToolbarButton::new(1001).with_image(0).with_text("New"));
        self.toolbar.add_button(ToolbarButton::new(1002).with_image(1).with_text("Open"));
        self.toolbar.add_button(ToolbarButton::new(1003).with_image(2).with_text("Save"));

        self.toolbar.add_separator();

        // Edit operations group
        self.toolbar.add_button(ToolbarButton::new(1004).with_image(3).with_text("Cut"));
        self.toolbar.add_button(ToolbarButton::new(1005).with_image(4).with_text("Copy"));
        self.toolbar.add_button(ToolbarButton::new(1006).with_image(5).with_text("Paste"));
        self.toolbar.add_button(ToolbarButton::new(1007).with_image(6).with_text("Undo"));

        self.toolbar.add_separator();

        // Other actions
        self.toolbar.add_button(ToolbarButton::new(1008).with_image(7).with_text("Delete"));
        self.toolbar.add_button(ToolbarButton::new(1009).with_image(8).with_text("Find"));

        self.toolbar.add_separator();

        // Toggle button example
        self.toolbar.add_button(
            ToolbarButton::new(1010)
                .with_image(9)
                .with_text("Help")
        );

        self.toolbar.auto_size();

        // Update status
        self.status_label.set_text(
            "Toolbar with proper 16x16 icons extracted from shell32.dll\n\n\
             Icons: New, Open, Save | Cut, Copy, Paste, Undo | Delete, Find | Help\n\n\
             These are the same icons used by Windows Explorer and other native apps."
        );
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let app = ToolbarShellIconsApp::build_ui(Default::default()).expect("Failed to build UI");
    app.setup_toolbar();

    nwg::dispatch_thread_events();
}
