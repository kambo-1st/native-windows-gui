# Missing Win32 Controls

This document tracks Win32 API controls and features not yet implemented in native-windows-gui.

## Common Controls (ComCtl32)

### High Priority

| Control | Win32 Class | Description | Status |
|---------|-------------|-------------|--------|
| Toolbar | `ToolbarWindow32` | Button bar with icons/text | ✅ Implemented |
| Month Calendar | `SysMonthCal32` | Standalone calendar widget | ✅ Implemented |
| Rebar | `ReBarWindow32` | Dockable toolbar container | ✅ Implemented |
| SysLink | `SysLink` | Hyperlink control (clickable URLs) | ✅ Implemented |

### Medium Priority

| Control | Win32 Class | Description |
|---------|-------------|-------------|
| Animation | `SysAnimate32` | AVI clip playback |
| ComboBoxEx | `ComboBoxEx32` | ComboBox with image support |
| IP Address | `SysIPAddress32` | IP address input with validation |
| Hot Key | `msctls_hotkey32` | Keyboard shortcut capture |
| Pager | `SysPager` | Scrollable container for controls |

## Modern Controls (Vista+)

| Control | Description |
|---------|-------------|
| Task Dialog | Modern message box with more options |
| Command Link | Large button with description text |
| Split Button | Button with dropdown arrow |
| Network Address | Network address input control |

## Dialogs

| Dialog | API | Description |
|--------|-----|-------------|
| Print Dialog | `PrintDlg` | Printer selection |
| Page Setup | `PageSetupDlg` | Page margins/orientation |
| Find/Replace | `FindText`/`ReplaceText` | Text search dialogs |

## System Features

| Feature | Description |
|---------|-------------|
| Accelerators | Keyboard shortcut tables (`LoadAccelerators`, `TranslateAccelerator`) |
| Owner-Draw | Custom rendering for menus, listbox, combobox items |
| Custom Draw | NM_CUSTOMDRAW for ListView, TreeView, etc. |
| Drag & Drop (OLE) | Full OLE drag-drop (beyond file drop) |
| Property Sheets | Tabbed dialog pages |

## Implementation Notes

### Toolbar
- Uses `CreateToolbarEx` or `CreateWindowEx` with `TOOLBARCLASSNAME`
- Requires `TBBUTTON` structures for buttons
- Supports separators, dropdown buttons, chevron overflow

### Month Calendar
- Similar to DatePicker but displays full calendar
- Supports range selection with `MCM_SETSELRANGE`
- Can highlight specific days

### Rebar
- Container for toolbar bands
- Supports drag-to-reorder, gripper handles
- Often combined with Toolbar and ComboBox

### SysLink
- Renders `<a href="...">text</a>` style markup
- Sends `NM_CLICK` with link info
- Useful for "Learn more" links, URLs

### Accelerators
- Requires accelerator table resource or `CreateAcceleratorTable`
- Must call `TranslateAccelerator` in message loop
- Maps key combos (Ctrl+S) to command IDs
