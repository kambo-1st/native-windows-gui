/*!
    A simple example demonstrating the ComboBoxEx control with images.

    Requires the `combobox-ex` and `image-list` features.
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

// Shell32.dll icon indices
mod shell_icons {
    pub const DOCUMENT: i32 = 1;
    pub const PROGRAM: i32 = 2;
    pub const FOLDER: i32 = 3;
    pub const FOLDER_OPEN: i32 = 4;
    pub const DRIVE: i32 = 8;
    pub const WORLD: i32 = 13;
    pub const SEARCH: i32 = 22;
    pub const HELP: i32 = 23;
}

fn to_utf16(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn add_shell_icon(image_list: &nwg::ImageList, icon_index: i32) -> bool {
    let shell32 = to_utf16("shell32.dll");
    let mut large_icon: HICON = ptr::null_mut();
    let mut small_icon: HICON = ptr::null_mut();

    let count = unsafe {
        ExtractIconExW(shell32.as_ptr(), icon_index, &mut large_icon, &mut small_icon, 1)
    };

    if count == 0 || small_icon.is_null() {
        return false;
    }

    let result = unsafe {
        ImageList_ReplaceIcon(image_list.handle as _, -1, small_icon)
    };

    unsafe {
        if !small_icon.is_null() { DestroyIcon(small_icon); }
        if !large_icon.is_null() { DestroyIcon(large_icon); }
    }

    result >= 0
}

#[derive(Default, NwgUi)]
pub struct ComboBoxExExample {
    #[nwg_control(size: (400, 220), position: (300, 300), title: "ComboBoxEx Example", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [ComboBoxExExample::exit], OnInit: [ComboBoxExExample::on_init])]
    window: nwg::Window,

    #[nwg_resource(size: (16, 16))]
    image_list: nwg::ImageList,

    #[nwg_control(text: "Dropdown list (non-editable):", position: (20, 10), size: (200, 20))]
    label1: nwg::Label,

    #[nwg_control(
        position: (20, 30),
        size: (250, 200),
        flags: "VISIBLE|TAB_STOP|DROPDOWN_LIST|VSCROLL"
    )]
    #[nwg_events(OnComboxBoxSelection: [ComboBoxExExample::on_selection])]
    combobox: nwg::ComboBoxEx,

    #[nwg_control(text: "Dropdown (editable):", position: (20, 70), size: (200, 20))]
    label2: nwg::Label,

    #[nwg_control(
        position: (20, 90),
        size: (250, 200),
        flags: "VISIBLE|TAB_STOP|DROPDOWN|VSCROLL"
    )]
    combobox_editable: nwg::ComboBoxEx,

    #[nwg_control(text: "Select an item...", position: (20, 140), size: (360, 25))]
    status_label: nwg::Label,

    #[nwg_control(text: "Add Item", position: (280, 30), size: (100, 25))]
    #[nwg_events(OnButtonClick: [ComboBoxExExample::add_item])]
    add_btn: nwg::Button,

    #[nwg_control(text: "Remove Item", position: (280, 60), size: (100, 25))]
    #[nwg_events(OnButtonClick: [ComboBoxExExample::remove_item])]
    remove_btn: nwg::Button,

    #[nwg_control(text: "Clear All", position: (280, 90), size: (100, 25))]
    #[nwg_events(OnButtonClick: [ComboBoxExExample::clear_items])]
    clear_btn: nwg::Button,
}

impl ComboBoxExExample {
    fn on_init(&self) {
        use shell_icons::*;

        // Add shell icons to image list
        add_shell_icon(&self.image_list, DOCUMENT);    // 0
        add_shell_icon(&self.image_list, FOLDER);      // 1
        add_shell_icon(&self.image_list, FOLDER_OPEN); // 2
        add_shell_icon(&self.image_list, DRIVE);       // 3
        add_shell_icon(&self.image_list, WORLD);       // 4
        add_shell_icon(&self.image_list, SEARCH);      // 5
        add_shell_icon(&self.image_list, HELP);        // 6
        add_shell_icon(&self.image_list, PROGRAM);     // 7

        // Set image list on combo boxes
        self.combobox.set_image_list(Some(&self.image_list));
        self.combobox_editable.set_image_list(Some(&self.image_list));

        // Add items to dropdown list
        self.combobox.push_item(&nwg::ComboBoxExItem::with_image("Documents", 0));
        self.combobox.push_item(&nwg::ComboBoxExItem::with_image("Folders", 1));
        self.combobox.push_item(&nwg::ComboBoxExItem::with_image("Open Folder", 2));
        self.combobox.push_item(&nwg::ComboBoxExItem::with_image("Drives", 3));
        self.combobox.push_item(&nwg::ComboBoxExItem::with_image("Network", 4));

        // Add items to editable combo with indentation
        self.combobox_editable.push_item(&nwg::ComboBoxExItem::with_image("Search", 5));
        self.combobox_editable.push_item(&nwg::ComboBoxExItem::with_image("Help", 6).indent(1));
        self.combobox_editable.push_item(&nwg::ComboBoxExItem::with_image("Programs", 7).indent(1));

        // Select first item
        self.combobox.set_selection(Some(0));
    }

    fn on_selection(&self) {
        if let Some(index) = self.combobox.selection() {
            let items = ["Documents", "Folders", "Open Folder", "Drives", "Network"];
            if index < items.len() {
                self.status_label.set_text(&format!("Selected: {} (index {})", items[index], index));
            }
        }
    }

    fn add_item(&self) {
        let count = self.combobox.len();
        let icon = (count % 8) as i32;
        self.combobox.push_item(&nwg::ComboBoxExItem::with_image(&format!("Item {}", count), icon));
        self.status_label.set_text(&format!("Added item at index {}", count));
    }

    fn remove_item(&self) {
        if let Some(index) = self.combobox.selection() {
            self.combobox.remove_item(index);
            self.status_label.set_text(&format!("Removed item at index {}", index));
        } else {
            self.status_label.set_text("Select an item to remove");
        }
    }

    fn clear_items(&self) {
        self.combobox.clear();
        self.status_label.set_text("Cleared all items");
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _app = ComboBoxExExample::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
