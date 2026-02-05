/*!
    A simple example demonstrating the Pager control.

    The Pager control provides a scrollable container for child controls,
    commonly used with toolbars that are too wide to fit in the available space.

    Requires the `pager` and `toolbar` features.
*/

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct PagerExample {
    #[nwg_control(size: (400, 180), position: (300, 300), title: "Pager Example", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [PagerExample::exit], OnInit: [PagerExample::on_init])]
    window: nwg::Window,

    #[nwg_control(text: "Horizontal Pager with Toolbar:", position: (10, 10), size: (200, 20))]
    label1: nwg::Label,

    // Horizontal pager - smaller than the toolbar it contains
    #[nwg_control(
        position: (10, 35),
        size: (200, 50),
        flags: "VISIBLE|HORIZONTAL"
    )]
    #[nwg_events(OnPagerScroll: [PagerExample::on_scroll])]
    pager: nwg::Pager,

    // Toolbar that will be contained in the pager
    // Note: parent is the pager, not the window
    // LIST shows text on buttons, NORESIZE prevents auto-sizing to parent
    #[nwg_control(
        parent: pager,
        size: (500, 50),
        flags: "VISIBLE|LIST|NORESIZE"
    )]
    toolbar: nwg::Toolbar,

    #[nwg_control(text: "Scroll pos: 0", position: (220, 50), size: (100, 25))]
    pos_label: nwg::Label,

    #[nwg_control(text: "Button Size:", position: (10, 100), size: (80, 20))]
    label2: nwg::Label,

    #[nwg_control(text: "12", position: (90, 98), size: (40, 25))]
    btn_size_input: nwg::TextInput,

    #[nwg_control(text: "Set", position: (135, 98), size: (40, 25))]
    #[nwg_events(OnButtonClick: [PagerExample::set_button_size])]
    set_btn_size: nwg::Button,

    #[nwg_control(text: "Recalc", position: (180, 98), size: (60, 25))]
    #[nwg_events(OnButtonClick: [PagerExample::recalc])]
    recalc_btn: nwg::Button,

    #[nwg_control(text: "Scroll to 0", position: (250, 98), size: (70, 25))]
    #[nwg_events(OnButtonClick: [PagerExample::scroll_start])]
    scroll_start_btn: nwg::Button,

    #[nwg_control(text: "Scroll to 100", position: (325, 98), size: (70, 25))]
    #[nwg_events(OnButtonClick: [PagerExample::scroll_end])]
    scroll_end_btn: nwg::Button,

    #[nwg_control(text: "Status: Ready", position: (10, 140), size: (380, 25))]
    status_label: nwg::Label,
}

impl PagerExample {
    fn on_init(&self) {
        // Add buttons to the toolbar
        self.toolbar.add_button(nwg::ToolbarButton {
            text: Some("File".to_string()),
            id: 1,
            image_index: -1,
            style: nwg::ToolbarButtonStyle::Button,
            enabled: true,
        });
        self.toolbar.add_button(nwg::ToolbarButton {
            text: Some("Edit".to_string()),
            id: 2,
            image_index: -1,
            style: nwg::ToolbarButtonStyle::Button,
            enabled: true,
        });
        self.toolbar.add_button(nwg::ToolbarButton {
            text: Some("View".to_string()),
            id: 3,
            image_index: -1,
            style: nwg::ToolbarButtonStyle::Button,
            enabled: true,
        });
        self.toolbar.add_button(nwg::ToolbarButton {
            text: Some("Insert".to_string()),
            id: 4,
            image_index: -1,
            style: nwg::ToolbarButtonStyle::Button,
            enabled: true,
        });
        self.toolbar.add_button(nwg::ToolbarButton {
            text: Some("Format".to_string()),
            id: 5,
            image_index: -1,
            style: nwg::ToolbarButtonStyle::Button,
            enabled: true,
        });
        self.toolbar.add_button(nwg::ToolbarButton {
            text: Some("Tools".to_string()),
            id: 6,
            image_index: -1,
            style: nwg::ToolbarButtonStyle::Button,
            enabled: true,
        });
        self.toolbar.add_button(nwg::ToolbarButton {
            text: Some("Help".to_string()),
            id: 7,
            image_index: -1,
            style: nwg::ToolbarButtonStyle::Button,
            enabled: true,
        });

        // Make toolbar auto-size to fit all buttons
        self.toolbar.auto_size();

        // Set the toolbar as the pager's child
        self.pager.set_child(Some(&self.toolbar));

        // Recalculate the pager size
        self.pager.recalc_size();

        self.status_label.set_text(&format!(
            "Button size: {}, Border: {}",
            self.pager.button_size(),
            self.pager.border()
        ));
    }

    fn on_scroll(&self) {
        let pos = self.pager.position();
        self.pos_label.set_text(&format!("Scroll pos: {}", pos));
    }

    fn set_button_size(&self) {
        let text = self.btn_size_input.text();
        if let Ok(size) = text.parse::<u32>() {
            self.pager.set_button_size(size);
            self.status_label.set_text(&format!("Button size set to {}", size));
        } else {
            self.status_label.set_text("Invalid button size");
        }
    }

    fn recalc(&self) {
        self.pager.recalc_size();
        self.status_label.set_text("Pager recalculated");
    }

    fn scroll_start(&self) {
        self.pager.set_position(0);
        self.on_scroll();
        self.status_label.set_text("Scrolled to start");
    }

    fn scroll_end(&self) {
        self.pager.set_position(100);
        self.on_scroll();
        self.status_label.set_text("Scrolled to position 100");
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _app = PagerExample::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
