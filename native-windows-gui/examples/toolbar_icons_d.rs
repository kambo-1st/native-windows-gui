/*!
    A toolbar example with icons using ImageList.

    This example demonstrates:
    - Creating an ImageList for toolbar icons
    - Loading system icons
    - Setting the image list on a toolbar
    - Creating buttons with icon indices
*/

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct ToolbarIconsApp {
    #[nwg_control(size: (600, 400), position: (300, 300), title: "Toolbar with Icons", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [ToolbarIconsApp::exit] )]
    window: nwg::Window,

    // Image list for toolbar icons
    // Note: OEM system icons are typically 32x32, so we use that size
    // For 16x16 icons, use custom icon files loaded via add_icon_from_filename()
    #[nwg_resource(size: (24, 24))]
    toolbar_image_list: nwg::ImageList,

    #[nwg_control(parent: window)]
    toolbar: nwg::Toolbar,

    #[nwg_control(size: (580, 300), position: (10, 60), text: "Click toolbar buttons to see events")]
    status_label: nwg::Label,
}

impl ToolbarIconsApp {
    fn setup_toolbar(&self) {
        use nwg::{ToolbarButton, ToolbarButtonStyle, Icon, OemIcon};

        // Load system icons into the image list
        // Note: OEM system icons load at their native size (32x32) regardless of size() param
        // For proper 16x16 icons, use custom .ico files with add_icon_from_filename()
        let icons = [
            OemIcon::WinLogo,      // 0: "New" - Windows logo
            OemIcon::Information,  // 1: "Open" - Info icon
            OemIcon::Warning,      // 2: "Save" - Warning icon
            OemIcon::Error,        // 3: "Delete" - Error icon
            OemIcon::Ques,         // 4: "Help" - Question icon
        ];

        for oem_icon in &icons {
            let mut icon = Icon::default();
            Icon::builder()
                .source_system(Some(*oem_icon))
                .build(&mut icon)
                .expect("Failed to load system icon");
            self.toolbar_image_list.add_icon(&icon);
        }

        // Set the image list on the toolbar
        self.toolbar.set_image_list(Some(&self.toolbar_image_list));

        // Add buttons with icons (icon index matches image list order)
        self.toolbar.add_button(
            ToolbarButton::new(1001)
                .with_image(0)  // WinLogo
                .with_text("New")
        );
        self.toolbar.add_button(
            ToolbarButton::new(1002)
                .with_image(1)  // Information
                .with_text("Open")
        );
        self.toolbar.add_button(
            ToolbarButton::new(1003)
                .with_image(2)  // Warning
                .with_text("Save")
        );

        self.toolbar.add_separator();

        self.toolbar.add_button(
            ToolbarButton::new(1004)
                .with_image(3)  // Error
                .with_text("Delete")
        );

        self.toolbar.add_separator();

        self.toolbar.add_button(
            ToolbarButton::new(1005)
                .with_image(4)  // Question
                .with_text("Help")
        );

        // Add a check button (toggleable)
        self.toolbar.add_button(
            ToolbarButton::new(1006)
                .with_image(0)
                .with_text("Toggle")
                .with_style(ToolbarButtonStyle::Check)
        );

        self.toolbar.auto_size();
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let app = ToolbarIconsApp::build_ui(Default::default()).expect("Failed to build UI");
    app.setup_toolbar();

    nwg::dispatch_thread_events();
}
