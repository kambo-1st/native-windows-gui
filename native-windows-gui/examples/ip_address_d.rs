/*!
    A simple example demonstrating the IP Address control.

    Requires the `ip-address` feature.
*/

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct IpAddressExample {
    #[nwg_control(size: (350, 200), position: (300, 300), title: "IP Address Example", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [IpAddressExample::exit])]
    window: nwg::Window,

    #[nwg_control(text: "Enter an IP address:", position: (20, 15), size: (150, 20))]
    label1: nwg::Label,

    #[nwg_control(position: (20, 40), size: (150, 23))]
    #[nwg_events(OnIpAddressFieldChanged: [IpAddressExample::on_field_changed])]
    ip_address: nwg::IpAddress,

    #[nwg_control(text: "Set to 192.168.1.1", position: (180, 40), size: (140, 25))]
    #[nwg_events(OnButtonClick: [IpAddressExample::set_default])]
    set_btn: nwg::Button,

    #[nwg_control(text: "Clear", position: (180, 70), size: (140, 25))]
    #[nwg_events(OnButtonClick: [IpAddressExample::clear])]
    clear_btn: nwg::Button,

    #[nwg_control(text: "Get Address", position: (180, 100), size: (140, 25))]
    #[nwg_events(OnButtonClick: [IpAddressExample::get_address])]
    get_btn: nwg::Button,

    #[nwg_control(text: "Set Range (0-127)", position: (180, 130), size: (140, 25))]
    #[nwg_events(OnButtonClick: [IpAddressExample::set_range])]
    range_btn: nwg::Button,

    #[nwg_control(text: "Current address: (none)", position: (20, 80), size: (150, 80))]
    status_label: nwg::Label,
}

impl IpAddressExample {
    fn on_field_changed(&self) {
        let (addr, filled) = self.ip_address.address_partial();
        self.status_label.set_text(&format!(
            "Field changed!\n{}.{}.{}.{}\n({} fields filled)",
            addr[0], addr[1], addr[2], addr[3], filled
        ));
    }

    fn set_default(&self) {
        self.ip_address.set_address([192, 168, 1, 1]);
        self.status_label.set_text("Set to 192.168.1.1");
    }

    fn clear(&self) {
        self.ip_address.clear();
        self.status_label.set_text("Address cleared");
    }

    fn get_address(&self) {
        if let Some(addr) = self.ip_address.address() {
            self.status_label.set_text(&format!(
                "Address:\n{}.{}.{}.{}",
                addr[0], addr[1], addr[2], addr[3]
            ));
        } else if self.ip_address.is_blank() {
            self.status_label.set_text("Address is blank");
        } else {
            let (addr, filled) = self.ip_address.address_partial();
            self.status_label.set_text(&format!(
                "Partial address:\n{}.{}.{}.{}\n({}/4 fields)",
                addr[0], addr[1], addr[2], addr[3], filled
            ));
        }
    }

    fn set_range(&self) {
        // Set first field range to 0-127 (class A addresses only)
        self.ip_address.set_field_range(0, 0, 127);
        self.status_label.set_text("Field 0 range set\nto 0-127");
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _app = IpAddressExample::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
