use winapi::um::winuser::{WS_VISIBLE, WS_DISABLED, WS_TABSTOP, WS_CHILD, WS_BORDER};
use winapi::um::commctrl::*;
use winapi::um::minwinbase::SYSTEMTIME;
use winapi::shared::minwindef::{WPARAM, LPARAM};
use crate::win32::window_helper as wh;
use crate::win32::base_helper::check_hwnd;
use crate::{Font, NwgError};
use super::{ControlBase, ControlHandle};
use std::mem;

const NOT_BOUND: &'static str = "MonthCalendar is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: MonthCalendar handle is not HWND!";

bitflags! {
    /**
        The MonthCalendar flags

        * VISIBLE:     The calendar is immediately visible after creation
        * DISABLED:    The calendar cannot be interacted with by the user
        * TAB_STOP:    The control can be selected using tab navigation
        * MULTI_SELECT: Allow selecting a range of dates
        * NO_TODAY:    Don't display "Today" at the bottom
        * NO_TODAY_CIRCLE: Don't circle today's date
        * WEEK_NUMBERS: Display week numbers on the left
    */
    pub struct MonthCalendarFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        const TAB_STOP = WS_TABSTOP;
        const BORDER = WS_BORDER;
        const MULTI_SELECT = MCS_MULTISELECT;
        const NO_TODAY = MCS_NOTODAY;
        const NO_TODAY_CIRCLE = MCS_NOTODAYCIRCLE;
        const WEEK_NUMBERS = MCS_WEEKNUMBERS;
    }
}

/// A date value for the MonthCalendar control
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MonthCalendarDate {
    pub year: u16,
    pub month: u16,
    pub day: u16,
}

impl MonthCalendarDate {
    pub fn new(year: u16, month: u16, day: u16) -> Self {
        MonthCalendarDate { year, month, day }
    }

    fn to_systemtime(&self) -> SYSTEMTIME {
        SYSTEMTIME {
            wYear: self.year,
            wMonth: self.month,
            wDay: self.day,
            wDayOfWeek: 0,
            wHour: 0,
            wMinute: 0,
            wSecond: 0,
            wMilliseconds: 0,
        }
    }

    fn from_systemtime(st: &SYSTEMTIME) -> Self {
        MonthCalendarDate {
            year: st.wYear,
            month: st.wMonth,
            day: st.wDay,
        }
    }
}

/**
A month calendar control displays a calendar-like user interface that provides
the user with a very intuitive and recognizable method of entering or selecting a date.

Unlike DatePicker (which shows a dropdown), MonthCalendar displays the full calendar inline.

Requires the `month-calendar` feature.

**Builder parameters:**
  * `parent`:       **Required.** The calendar's parent container.
  * `size`:         The calendar size. Note: the control may adjust this to fit the calendar.
  * `position`:     The calendar position.
  * `enabled`:      If the calendar can be used by the user.
  * `flags`:        A combination of the MonthCalendarFlags values.
  * `ex_flags`:     A combination of win32 window extended flags.
  * `font`:         The font used for the calendar text.
  * `date`:         The initial selected date.
  * `range`:        The min/max selectable date range.
  * `focus`:        The control receives focus after being created.

**Control events:**
  * `OnMonthCalendarSelect`: When the user selects a date
  * `OnMonthCalendarSelectionChanged`: When the selection changes
  * `OnMonthCalendarViewChange`: When the view changes (month/year navigation)
  * `MousePress(_)`: Generic mouse press events
  * `OnMouseMove`: Generic mouse move event

```rust
use native_windows_gui as nwg;
fn build_calendar(cal: &mut nwg::MonthCalendar, window: &nwg::Window) {
    nwg::MonthCalendar::builder()
        .position((10, 10))
        .parent(window)
        .build(cal);
}
```
*/
#[derive(Default, PartialEq, Eq)]
pub struct MonthCalendar {
    pub handle: ControlHandle,
}

impl MonthCalendar {
    pub fn builder<'a>() -> MonthCalendarBuilder<'a> {
        MonthCalendarBuilder {
            size: (250, 200),
            position: (0, 0),
            focus: false,
            flags: None,
            ex_flags: 0,
            font: None,
            parent: None,
            date: None,
            min_date: None,
            max_date: None,
            max_selection_count: None,
        }
    }

    /// Returns the currently selected date.
    /// If multi-select is enabled, returns the first date of the selection.
    pub fn selection(&self) -> MonthCalendarDate {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut st: SYSTEMTIME = unsafe { mem::zeroed() };
        wh::send_message(handle, MCM_GETCURSEL, 0, &mut st as *mut SYSTEMTIME as LPARAM);

        MonthCalendarDate::from_systemtime(&st)
    }

    /// Sets the currently selected date.
    pub fn set_selection(&self, date: MonthCalendarDate) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let st = date.to_systemtime();
        wh::send_message(handle, MCM_SETCURSEL, 0, &st as *const SYSTEMTIME as LPARAM);
    }

    /// Returns the selected date range (for multi-select calendars).
    /// Returns [start, end] dates.
    pub fn selection_range(&self) -> [MonthCalendarDate; 2] {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut range: [SYSTEMTIME; 2] = unsafe { mem::zeroed() };
        wh::send_message(handle, MCM_GETSELRANGE, 0, &mut range as *mut [SYSTEMTIME; 2] as LPARAM);

        [
            MonthCalendarDate::from_systemtime(&range[0]),
            MonthCalendarDate::from_systemtime(&range[1]),
        ]
    }

    /// Sets the selected date range (for multi-select calendars).
    pub fn set_selection_range(&self, start: MonthCalendarDate, end: MonthCalendarDate) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let range = [start.to_systemtime(), end.to_systemtime()];
        wh::send_message(handle, MCM_SETSELRANGE, 0, &range as *const [SYSTEMTIME; 2] as LPARAM);
    }

    /// Returns the minimum selectable date, or None if not set.
    pub fn min_date(&self) -> Option<MonthCalendarDate> {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut range: [SYSTEMTIME; 2] = unsafe { mem::zeroed() };
        let result = wh::send_message(handle, MCM_GETRANGE, 0, &mut range as *mut [SYSTEMTIME; 2] as LPARAM);

        if (result as usize & GDTR_MIN) != 0 {
            Some(MonthCalendarDate::from_systemtime(&range[0]))
        } else {
            None
        }
    }

    /// Returns the maximum selectable date, or None if not set.
    pub fn max_date(&self) -> Option<MonthCalendarDate> {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut range: [SYSTEMTIME; 2] = unsafe { mem::zeroed() };
        let result = wh::send_message(handle, MCM_GETRANGE, 0, &mut range as *mut [SYSTEMTIME; 2] as LPARAM);

        if (result as usize & GDTR_MAX) != 0 {
            Some(MonthCalendarDate::from_systemtime(&range[1]))
        } else {
            None
        }
    }

    /// Sets the minimum and maximum selectable dates.
    /// Pass None to remove a limit.
    pub fn set_date_range(&self, min: Option<MonthCalendarDate>, max: Option<MonthCalendarDate>) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut flags: usize = 0;
        let mut range: [SYSTEMTIME; 2] = unsafe { mem::zeroed() };

        if let Some(min_date) = min {
            flags |= GDTR_MIN;
            range[0] = min_date.to_systemtime();
        }

        if let Some(max_date) = max {
            flags |= GDTR_MAX;
            range[1] = max_date.to_systemtime();
        }

        wh::send_message(handle, MCM_SETRANGE, flags as WPARAM, &range as *const [SYSTEMTIME; 2] as LPARAM);
    }

    /// Returns the "today" date shown by the calendar.
    pub fn today(&self) -> MonthCalendarDate {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut st: SYSTEMTIME = unsafe { mem::zeroed() };
        wh::send_message(handle, MCM_GETTODAY, 0, &mut st as *mut SYSTEMTIME as LPARAM);

        MonthCalendarDate::from_systemtime(&st)
    }

    /// Sets the "today" date shown by the calendar.
    pub fn set_today(&self, date: MonthCalendarDate) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let st = date.to_systemtime();
        wh::send_message(handle, MCM_SETTODAY, 0, &st as *const SYSTEMTIME as LPARAM);
    }

    /// Returns the maximum number of days that can be selected (for multi-select).
    pub fn max_selection_count(&self) -> u32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, MCM_GETMAXSELCOUNT, 0, 0) as u32
    }

    /// Sets the maximum number of days that can be selected (for multi-select).
    pub fn set_max_selection_count(&self, count: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, MCM_SETMAXSELCOUNT, count as WPARAM, 0);
    }

    /// Returns the first day of the week (0 = Monday, 6 = Sunday on most locales).
    pub fn first_day_of_week(&self) -> u32 {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        let result = wh::send_message(handle, MCM_GETFIRSTDAYOFWEEK, 0, 0);
        (result & 0xFFFF) as u32
    }

    /// Sets the first day of the week (0 = Monday, 6 = Sunday).
    pub fn set_first_day_of_week(&self, day: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        wh::send_message(handle, MCM_SETFIRSTDAYOFWEEK, 0, day as LPARAM);
    }

    /// Returns the minimum size required to display a full month.
    pub fn minimum_size(&self) -> (u32, u32) {
        use winapi::shared::windef::RECT;

        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let mut rect: RECT = unsafe { mem::zeroed() };
        wh::send_message(handle, MCM_GETMINREQRECT, 0, &mut rect as *mut RECT as LPARAM);

        ((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32)
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

    /// Return true if the control currently has the keyboard focus
    pub fn focus(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_focus(handle) }
    }

    /// Set the keyboard focus on the calendar
    pub fn set_focus(&self) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_focus(handle); }
    }

    /// Return true if the control is enabled
    pub fn enabled(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_enabled(handle) }
    }

    /// Enable or disable the control
    pub fn set_enabled(&self, v: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_enabled(handle, v) }
    }

    /// Return true if the control is visible
    pub fn visible(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_visibility(handle) }
    }

    /// Show or hide the control
    pub fn set_visible(&self, v: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_visibility(handle, v) }
    }

    /// Return the size of the control
    pub fn size(&self) -> (u32, u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_size(handle) }
    }

    /// Set the size of the control
    pub fn set_size(&self, x: u32, y: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_size(handle, x, y, false) }
    }

    /// Return the position of the control
    pub fn position(&self) -> (i32, i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_position(handle) }
    }

    /// Set the position of the control
    pub fn set_position(&self, x: i32, y: i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_position(handle, x, y) }
    }

    /// Winapi class name
    pub fn class_name(&self) -> &'static str {
        "SysMonthCal32"
    }

    /// Winapi base flags
    pub fn flags(&self) -> u32 {
        WS_VISIBLE | WS_TABSTOP
    }

    /// Required flags
    pub fn forced_flags(&self) -> u32 {
        WS_CHILD
    }
}

impl Drop for MonthCalendar {
    fn drop(&mut self) {
        self.handle.destroy();
    }
}

pub struct MonthCalendarBuilder<'a> {
    size: (i32, i32),
    position: (i32, i32),
    flags: Option<MonthCalendarFlags>,
    ex_flags: u32,
    font: Option<&'a Font>,
    focus: bool,
    parent: Option<ControlHandle>,
    date: Option<MonthCalendarDate>,
    min_date: Option<MonthCalendarDate>,
    max_date: Option<MonthCalendarDate>,
    max_selection_count: Option<u32>,
}

impl<'a> MonthCalendarBuilder<'a> {
    pub fn size(mut self, size: (i32, i32)) -> MonthCalendarBuilder<'a> {
        self.size = size;
        self
    }

    pub fn position(mut self, pos: (i32, i32)) -> MonthCalendarBuilder<'a> {
        self.position = pos;
        self
    }

    pub fn flags(mut self, flags: MonthCalendarFlags) -> MonthCalendarBuilder<'a> {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, flags: u32) -> MonthCalendarBuilder<'a> {
        self.ex_flags = flags;
        self
    }

    pub fn font(mut self, font: Option<&'a Font>) -> MonthCalendarBuilder<'a> {
        self.font = font;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> MonthCalendarBuilder<'a> {
        self.parent = Some(p.into());
        self
    }

    pub fn date(mut self, date: MonthCalendarDate) -> MonthCalendarBuilder<'a> {
        self.date = Some(date);
        self
    }

    pub fn min_date(mut self, date: MonthCalendarDate) -> MonthCalendarBuilder<'a> {
        self.min_date = Some(date);
        self
    }

    pub fn max_date(mut self, date: MonthCalendarDate) -> MonthCalendarBuilder<'a> {
        self.max_date = Some(date);
        self
    }

    pub fn max_selection_count(mut self, count: u32) -> MonthCalendarBuilder<'a> {
        self.max_selection_count = Some(count);
        self
    }

    pub fn focus(mut self, focus: bool) -> MonthCalendarBuilder<'a> {
        self.focus = focus;
        self
    }

    pub fn build(self, out: &mut MonthCalendar) -> Result<(), NwgError> {
        let flags = self.flags.map(|f| f.bits()).unwrap_or(out.flags());

        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("MonthCalendar"))
        }?;

        *out = Default::default();

        out.handle = ControlBase::build_hwnd()
            .class_name(out.class_name())
            .forced_flags(out.forced_flags())
            .flags(flags)
            .ex_flags(self.ex_flags)
            .size(self.size)
            .position(self.position)
            .parent(Some(parent))
            .build()?;

        if self.font.is_some() {
            out.set_font(self.font);
        } else {
            out.set_font(Font::global_default().as_ref());
        }

        if let Some(date) = self.date {
            out.set_selection(date);
        }

        if self.min_date.is_some() || self.max_date.is_some() {
            out.set_date_range(self.min_date, self.max_date);
        }

        if let Some(count) = self.max_selection_count {
            out.set_max_selection_count(count);
        }

        if self.focus {
            out.set_focus();
        }

        Ok(())
    }
}
