/*!
    A simple application that demonstrates the Toolbar control.
*/

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct ToolbarApp {
    #[nwg_control(size: (500, 400), position: (300, 300), title: "Toolbar Example", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [ToolbarApp::exit] )]
    window: nwg::Window,

    #[nwg_control(parent: window)]
    toolbar: nwg::Toolbar,

    #[nwg_control(size: (480, 300), position: (10, 50))]
    #[nwg_events( OnButtonClick: [ToolbarApp::on_status_click] )]
    status_label: nwg::Label,
}

impl ToolbarApp {
    fn setup_toolbar(&self) {
        use nwg::{ToolbarButton, ToolbarButtonStyle};

        // Add buttons to the toolbar
        self.toolbar.add_button(ToolbarButton::new(1001).with_text("New"));
        self.toolbar.add_button(ToolbarButton::new(1002).with_text("Open"));
        self.toolbar.add_button(ToolbarButton::new(1003).with_text("Save"));
        self.toolbar.add_separator();
        self.toolbar.add_button(ToolbarButton::new(1004).with_text("Cut"));
        self.toolbar.add_button(ToolbarButton::new(1005).with_text("Copy"));
        self.toolbar.add_button(ToolbarButton::new(1006).with_text("Paste"));
        self.toolbar.add_separator();
        self.toolbar.add_button(
            ToolbarButton::new(1007)
                .with_text("Toggle")
                .with_style(ToolbarButtonStyle::Check)
        );

        self.toolbar.auto_size();
    }

    fn on_status_click(&self) {
        self.status_label.set_text("Status label clicked!");
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let app = ToolbarApp::build_ui(Default::default()).expect("Failed to build UI");
    app.setup_toolbar();

    // Set initial status
    app.status_label.set_text("Click toolbar buttons to interact.\nButton count: 8");

    nwg::dispatch_thread_events();
}
