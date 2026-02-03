/*!
    A simple example demonstrating the MonthCalendar control.
*/

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct MonthCalendarApp {
    #[nwg_control(size: (400, 400), position: (300, 300), title: "Month Calendar Example", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [MonthCalendarApp::exit] )]
    window: nwg::Window,

    // Size set to accommodate the calendar (minimum ~170x235)
    #[nwg_control(position: (10, 10), size: (200, 250), flags: "VISIBLE|TAB_STOP")]
    calendar: nwg::MonthCalendar,

    #[nwg_control(text: "Selected: (none)", position: (10, 270), size: (380, 25))]
    selected_label: nwg::Label,

    #[nwg_control(text: "Get Selection", position: (10, 305), size: (120, 30))]
    #[nwg_events( OnButtonClick: [MonthCalendarApp::on_get_selection] )]
    get_btn: nwg::Button,

    #[nwg_control(text: "Set to Today", position: (140, 305), size: (120, 30))]
    #[nwg_events( OnButtonClick: [MonthCalendarApp::on_set_today] )]
    today_btn: nwg::Button,

    #[nwg_control(text: "Set Range", position: (270, 305), size: (120, 30))]
    #[nwg_events( OnButtonClick: [MonthCalendarApp::on_set_range] )]
    range_btn: nwg::Button,

    #[nwg_control(text: "Min Size: calculating...", position: (10, 345), size: (380, 25))]
    info_label: nwg::Label,
}

impl MonthCalendarApp {
    fn on_get_selection(&self) {
        let date = self.calendar.selection();
        self.selected_label.set_text(&format!(
            "Selected: {}/{}/{}",
            date.month, date.day, date.year
        ));
    }

    fn on_set_today(&self) {
        let today = self.calendar.today();
        self.calendar.set_selection(today);
        self.on_get_selection();
    }

    fn on_set_range(&self) {
        use nwg::MonthCalendarDate;

        // Set selectable range to 2020-2030
        self.calendar.set_date_range(
            Some(MonthCalendarDate::new(2020, 1, 1)),
            Some(MonthCalendarDate::new(2030, 12, 31)),
        );

        self.info_label.set_text("Date range set: 2020-01-01 to 2030-12-31");
    }

    fn show_min_size(&self) {
        let (w, h) = self.calendar.minimum_size();
        self.info_label.set_text(&format!("Min size: {}x{}", w, h));
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let app = MonthCalendarApp::build_ui(Default::default()).expect("Failed to build UI");

    // Show the minimum required size
    app.show_min_size();

    nwg::dispatch_thread_events();
}
