# Native Windows GUI - Development Guide

This document contains architectural findings and implementation guidelines for contributing to native-windows-gui.

## Project Structure

```
native-windows-gui-fork/
├── native-windows-gui/          # Main library
│   ├── src/
│   │   ├── lib.rs               # Public API surface
│   │   ├── events.rs            # Event enum definitions
│   │   ├── controls/            # All widget implementations
│   │   │   ├── mod.rs           # Control exports
│   │   │   ├── control_base.rs  # Low-level builders
│   │   │   ├── control_handle.rs# Handle abstraction
│   │   │   ├── window.rs        # Window control
│   │   │   ├── button.rs        # Button control
│   │   │   └── ...              # Other controls
│   │   ├── win32/               # Win32 FFI layer
│   │   │   ├── mod.rs           # Init, message loop
│   │   │   ├── window.rs        # Event dispatch (~1000 lines)
│   │   │   ├── base_helper.rs   # String conversion, validation
│   │   │   └── window_helper.rs # Window creation helpers
│   │   ├── resources/           # Font, Bitmap, Icon, Cursor
│   │   └── layouts/             # GridLayout, FlexboxLayout
│   ├── examples/                # Example applications
│   └── Cargo.toml               # Features and dependencies
├── native-windows-derive/       # Proc macros (#[derive(NwgUi)])
├── native-windows-canvas/       # Direct2D wrapper
└── docs/                        # Documentation
```

## Build Commands (WSL2)

```bash
# From project root
../scripts/win-cargo.sh build              # Build library
../scripts/win-cargo.sh build --examples   # Build all examples
../scripts/win-cargo.sh run --example basic_d  # Run example
```

## Architecture Overview

### Layered Design

```
┌─────────────────────────────────────────────────────────┐
│  Application Code                                        │
│  #[derive(NwgUi)] struct MyApp { window, button, ... }  │
├─────────────────────────────────────────────────────────┤
│  Derive Macros (native-windows-derive)                   │
│  - Generates NativeUi::build_ui()                       │
│  - Expands #[nwg_control] → builder calls               │
│  - Expands #[nwg_events] → event bindings               │
├─────────────────────────────────────────────────────────┤
│  Control Layer (controls/)                               │
│  - Widget structs: Window, Button, ListView, etc.       │
│  - Builder pattern: Button::builder().text().build()    │
│  - Bitflags: WindowFlags, ButtonFlags                   │
├─────────────────────────────────────────────────────────┤
│  Win32 FFI Layer (win32/)                                │
│  - ControlHandle enum (Hwnd, Menu, Timer, etc.)         │
│  - HwndBuilder for CreateWindowExW                      │
│  - Event dispatch via SetWindowSubclass                 │
│  - UTF-8 ↔ UTF-16 string conversion                     │
├─────────────────────────────────────────────────────────┤
│  winapi crate (raw Win32 bindings)                       │
└─────────────────────────────────────────────────────────┘
```

### Key Files for Adding Controls

| File | Purpose |
|------|---------|
| `controls/mod.rs` | Export new control module |
| `controls/your_control.rs` | Control implementation |
| `events.rs` | Add new event types if needed |
| `win32/window.rs` | Add event handling in `process_events()` |
| `Cargo.toml` | Add feature flag |
| `lib.rs` | Re-export control |

## How to Implement a New Control

### Step 1: Create Control File

```rust
// controls/toolbar.rs

use crate::win32::window_helper as wh;
use crate::win32::base_helper as bh;
use crate::{NwgError, RawEventHandler, unbind_raw_event_handler};
use super::{ControlBase, ControlHandle};
use winapi::um::commctrl::*;
use winapi::um::winuser::*;

bitflags! {
    pub struct ToolbarFlags: u32 {
        const VISIBLE = WS_VISIBLE;
        const FLAT = TBSTYLE_FLAT;
        const LIST = TBSTYLE_LIST;
        const TOOLTIPS = TBSTYLE_TOOLTIPS;
        // ... etc
    }
}

/// A toolbar control
#[derive(Default)]
pub struct Toolbar {
    pub handle: ControlHandle,
    handler: Option<RawEventHandler>,
}

impl Toolbar {
    pub fn builder() -> ToolbarBuilder {
        ToolbarBuilder {
            // ... default values
        }
    }

    /// Add a button to the toolbar
    pub fn add_button(&self, /* params */) {
        // Implementation
    }

    // ... other methods
}

impl Drop for Toolbar {
    fn drop(&mut self) {
        if let Some(h) = self.handler.take() {
            unbind_raw_event_handler(&h);
        }
        self.handle.destroy();
    }
}

pub struct ToolbarBuilder {
    parent: Option<ControlHandle>,
    // ... fields
}

impl ToolbarBuilder {
    // Fluent setters returning Self
    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> Self {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut Toolbar) -> Result<(), NwgError> {
        // Use ControlBase::build_hwnd() or direct CreateWindowExW
        out.handle = ControlBase::build_hwnd()
            .class_name("ToolbarWindow32")
            .parent(self.parent)
            .build()?;
        Ok(())
    }
}
```

### Step 2: Add to controls/mod.rs

```rust
#[cfg(feature = "toolbar")]
mod toolbar;

#[cfg(feature = "toolbar")]
pub use toolbar::*;
```

### Step 3: Add Feature to Cargo.toml

```toml
[features]
toolbar = []
all = ["toolbar", ...]  # Add to 'all' feature
```

### Step 4: Re-export in lib.rs

```rust
#[cfg(feature = "toolbar")]
pub use controls::Toolbar;
```

### Step 5: Add Events (if needed)

In `events.rs`:
```rust
pub enum Event {
    // ...
    OnToolbarClick,
    OnToolbarDropDown,
}
```

In `win32/window.rs` `process_events()`:
```rust
WM_NOTIFY => {
    let nmhdr = ...;
    match nmhdr.code {
        TBN_DROPDOWN => callback(Event::OnToolbarDropDown, ...),
        // ...
    }
}
```

## Control Patterns

### ControlHandle

```rust
pub enum ControlHandle {
    NoHandle,
    Hwnd(HWND),
    Menu(HMENU, HMENU),
    MenuItem(HMENU, u32),
    Timer(HWND, u32),
    Notice(HWND, u32),
    SystemTray(HWND),
    // Add new variants if needed
}
```

### Builder Pattern

All controls use the builder pattern:
```rust
nwg::Button::builder()
    .text("Click me")
    .size((100, 30))
    .position((10, 10))
    .parent(&window)
    .build(&mut self.button)?;
```

### Event Handling

Events are routed through `process_events()` in `win32/window.rs`:
```rust
unsafe extern "system" fn process_events(
    hwnd: HWND,
    msg: UINT,
    w: WPARAM,
    l: LPARAM,
    _id: UINT_PTR,
    data: DWORD_PTR
) -> LRESULT {
    match msg {
        WM_COMMAND => { /* Button clicks, menu items */ }
        WM_NOTIFY => { /* ListView, TreeView, Toolbar notifications */ }
        WM_SIZE => { /* Resize events */ }
        // ... etc
    }
}
```

### String Conversion

```rust
use crate::win32::base_helper::{to_utf16, from_utf16};

// Rust → Win32
let wide: Vec<u16> = to_utf16("Hello");

// Win32 → Rust
let text: String = from_utf16(&wide_buffer);
```

### Handle Validation

```rust
use crate::win32::base_helper::check_hwnd;

pub fn set_text(&self, text: &str) {
    let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
    // Now safe to use handle
}
```

## Win32 Control Classes

| Control | Win32 Class | Init Flag |
|---------|-------------|-----------|
| Toolbar | `ToolbarWindow32` | `ICC_BAR_CLASSES` |
| Month Calendar | `SysMonthCal32` | `ICC_DATE_CLASSES` |
| Rebar | `ReBarWindow32` | `ICC_COOL_CLASSES` |
| SysLink | `SysLink` | `ICC_LINK_CLASS` |
| Animation | `SysAnimate32` | `ICC_ANIMATE_CLASS` |
| IP Address | `SysIPAddress32` | `ICC_INTERNET_CLASSES` |
| Hot Key | `msctls_hotkey32` | `ICC_HOTKEY_CLASS` |
| Pager | `SysPager` | `ICC_PAGESCROLLER_CLASS` |

## Adding Common Control Classes

In `win32/mod.rs` `init_common_controls()`:
```rust
pub fn init_common_controls() -> Result<(), NwgError> {
    let mut classes = ICC_BAR_CLASSES | ICC_STANDARD_CLASSES;

    #[cfg(feature = "toolbar")]
    { classes |= ICC_BAR_CLASSES; }

    #[cfg(feature = "month-calendar")]
    { classes |= ICC_DATE_CLASSES; }

    // ... etc
}
```

## Testing

1. Create example in `examples/`:
```rust
// examples/toolbar_d.rs
#[derive(Default, NwgUi)]
pub struct ToolbarExample {
    #[nwg_control(...)]
    window: nwg::Window,

    #[nwg_control(parent: window)]
    toolbar: nwg::Toolbar,
}
```

2. Build and run:
```bash
../scripts/win-cargo.sh run --example toolbar_d
```

## Current Coverage

### Implemented (27 controls)
- Window, Frame, MessageWindow
- Button, CheckBox, RadioButton
- TextInput, TextBox, RichTextBox
- Label, RichLabel, ImageFrame
- ListBox, ListView, TreeView, ComboBox
- TabsContainer, Tab
- Menu, MenuItem, MenuSeparator
- ProgressBar, TrackBar, ScrollBar
- DatePicker, NumberSelect
- StatusBar, Tooltip
- TrayNotification, Timer, AnimationTimer, Notice
- ExternCanvas, Plotters

### Missing (High Priority)
- [ ] Toolbar (`ToolbarWindow32`)
- [ ] Month Calendar (`SysMonthCal32`)
- [ ] Rebar (`ReBarWindow32`)
- [ ] SysLink (`SysLink`)

### Missing (Medium Priority)
- [ ] Animation (`SysAnimate32`)
- [ ] ComboBoxEx (`ComboBoxEx32`)
- [ ] IP Address (`SysIPAddress32`)
- [ ] Hot Key (`msctls_hotkey32`)
- [ ] Pager (`SysPager`)

## References

- [Win32 Common Controls](https://docs.microsoft.com/en-us/windows/win32/controls/common-controls-intro)
- [winapi crate docs](https://docs.rs/winapi)
- [Existing control implementations](native-windows-gui/src/controls/)
