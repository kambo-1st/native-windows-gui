/*!
    A simple example demonstrating the Rebar control with embedded toolbars and icons.

    Requires the following features: `rebar`, `toolbar`, `image-list`
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
    pub const FOLDER_OPEN: i32 = 4;        // Open folder
    pub const FLOPPY: i32 = 6;             // Floppy/Save
    pub const PRINT: i32 = 16;             // Printer
    pub const CUT: i32 = 260;              // Cut (scissors)
    pub const COPY: i32 = 261;             // Copy
    pub const PASTE: i32 = 262;            // Paste (clipboard)
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
pub struct RebarExample {
    #[nwg_control(size: (600, 400), position: (300, 300), title: "Rebar Example with Icons", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [RebarExample::exit], OnInit: [RebarExample::on_init])]
    window: nwg::Window,

    // Image lists for the toolbars (16x16 icons)
    #[nwg_resource(size: (16, 16))]
    file_image_list: nwg::ImageList,

    #[nwg_resource(size: (16, 16))]
    edit_image_list: nwg::ImageList,

    #[nwg_control(parent: window, flags: "VISIBLE|FLAT|NODIVIDER|NORESIZE")]
    toolbar1: nwg::Toolbar,

    #[nwg_control(parent: window, flags: "VISIBLE|FLAT|NODIVIDER|NORESIZE")]
    toolbar2: nwg::Toolbar,

    #[nwg_control(parent: window)]
    rebar: nwg::Rebar,

    #[nwg_control(parent: window, position: (10, 80), size: (580, 280))]
    status_label: nwg::Label,
}

impl RebarExample {
    fn on_init(&self) {
        use shell_icons::*;

        // Load icons for toolbar 1 (File operations)
        let file_icons = [NEW_FILE, FOLDER_OPEN, FLOPPY, PRINT];
        for icon_index in &file_icons {
            add_shell_icon_to_imagelist(&self.file_image_list, *icon_index);
        }

        // Load icons for toolbar 2 (Edit operations)
        let edit_icons = [CUT, COPY, PASTE];
        for icon_index in &edit_icons {
            add_shell_icon_to_imagelist(&self.edit_image_list, *icon_index);
        }

        // Set image lists on toolbars
        self.toolbar1.set_image_list(Some(&self.file_image_list));
        self.toolbar2.set_image_list(Some(&self.edit_image_list));

        // Setup toolbar 1 with file operations
        self.toolbar1.add_button(nwg::ToolbarButton::new(1001).with_image(0).with_text("New"));
        self.toolbar1.add_button(nwg::ToolbarButton::new(1002).with_image(1).with_text("Open"));
        self.toolbar1.add_button(nwg::ToolbarButton::new(1003).with_image(2).with_text("Save"));
        self.toolbar1.add_separator();
        self.toolbar1.add_button(nwg::ToolbarButton::new(1004).with_image(3).with_text("Print"));
        self.toolbar1.auto_size();

        // Setup toolbar 2 with edit operations
        self.toolbar2.add_button(nwg::ToolbarButton::new(2001).with_image(0).with_text("Cut"));
        self.toolbar2.add_button(nwg::ToolbarButton::new(2002).with_image(1).with_text("Copy"));
        self.toolbar2.add_button(nwg::ToolbarButton::new(2003).with_image(2).with_text("Paste"));
        self.toolbar2.auto_size();

        // Add toolbars to rebar as bands
        self.rebar.add_band(
            nwg::RebarBand::new(1)
                .with_child(&self.toolbar1)
                .with_min_width(150)
                .with_min_height(38)
                .with_width(250)
        );

        self.rebar.add_band(
            nwg::RebarBand::new(2)
                .with_child(&self.toolbar2)
                .with_min_width(100)
                .with_min_height(38)
                .with_width(180)
        );

        let status = format!(
            "Rebar with Icons!\n\n\
             Band count: {}\n\
             Rebar height: {}\n\
             Row count: {}\n\n\
             Icons loaded from shell32.dll\n\
             Drag the gripper bars to rearrange bands!",
            self.rebar.band_count(),
            self.rebar.height(),
            self.rebar.row_count()
        );
        self.status_label.set_text(&status);
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _app = RebarExample::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
