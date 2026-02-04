/*!
    A simple example demonstrating the SysLink control for hyperlinks.

    Requires the `syslink` feature.
*/

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct SysLinkExample {
    #[nwg_control(size: (400, 300), position: (300, 300), title: "SysLink Example", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [SysLinkExample::exit])]
    window: nwg::Window,

    #[nwg_control(text: "Welcome to SysLink Demo!", position: (20, 20), size: (360, 25))]
    title_label: nwg::Label,

    #[nwg_control(
        text: "Visit <a href=\"https://github.com\">GitHub</a> for code hosting.",
        position: (20, 60),
        size: (360, 25)
    )]
    #[nwg_events(OnSysLinkClick: [SysLinkExample::on_link_click(SELF, EVT_DATA)])]
    link1: nwg::SysLink,

    #[nwg_control(
        text: "Check out <a href=\"https://www.rust-lang.org\">Rust</a> programming language.",
        position: (20, 100),
        size: (360, 25)
    )]
    #[nwg_events(OnSysLinkClick: [SysLinkExample::on_link_click(SELF, EVT_DATA)])]
    link2: nwg::SysLink,

    #[nwg_control(
        text: "Learn about <a id=\"nwg\" href=\"https://github.com/gabdube/native-windows-gui\">Native Windows GUI</a>!",
        position: (20, 140),
        size: (360, 25)
    )]
    #[nwg_events(OnSysLinkClick: [SysLinkExample::on_link_click(SELF, EVT_DATA)])]
    link3: nwg::SysLink,

    #[nwg_control(
        text: "Multiple links: <a href=\"https://google.com\">Google</a> or <a href=\"https://bing.com\">Bing</a>",
        position: (20, 180),
        size: (360, 25)
    )]
    #[nwg_events(OnSysLinkClick: [SysLinkExample::on_link_click(SELF, EVT_DATA)])]
    link4: nwg::SysLink,

    #[nwg_control(text: "Click a link above to see its URL", position: (20, 230), size: (360, 50))]
    status_label: nwg::Label,
}

impl SysLinkExample {
    fn on_link_click(&self, data: &nwg::EventData) {
        let (url, id) = data.on_syslink_click();

        let msg = if id.is_empty() {
            format!("Link clicked!\nURL: {}", url)
        } else {
            format!("Link clicked!\nID: {}\nURL: {}", id, url)
        };

        self.status_label.set_text(&msg);

        // Open the URL in the default browser
        use std::process::Command;
        let _ = Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _app = SysLinkExample::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
