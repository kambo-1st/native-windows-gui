/*!
    A simple example demonstrating the Animation control for AVI playback.

    Requires the `animation` feature.

    Note: The animation control only supports uncompressed or RLE-compressed AVI files.
    This example uses shell32.dll built-in animations.
*/

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

/// Shell32.dll resource IDs for common animations
const SHELL32_SEARCH_FLASHLIGHT: u16 = 150;  // Flashlight searching
const SHELL32_SEARCH_COMPUTER: u16 = 152;    // Computer searching
const SHELL32_SEARCH_DOCUMENT: u16 = 151;    // Document searching
const SHELL32_COPY_FILE: u16 = 160;          // File copy animation
const SHELL32_RECYCLE: u16 = 161;            // Recycle bin animation
const SHELL32_DELETE: u16 = 162;             // Delete animation (recycle with hand)
const SHELL32_MOVE: u16 = 163;               // Move animation
const SHELL32_DOWNLOAD: u16 = 164;           // Download animation
const SHELL32_SEARCH: u16 = 165;             // Search magnifying glass

#[derive(Default, NwgUi)]
pub struct AnimationExample {
    // Store the shell32 module handle
    shell32_handle: std::cell::Cell<isize>,

    #[nwg_control(size: (400, 380), position: (300, 300), title: "Animation Example", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [AnimationExample::exit], OnInit: [AnimationExample::on_init])]
    window: nwg::Window,

    #[nwg_control(text: "Animation Control Demo", position: (20, 10), size: (360, 25))]
    title_label: nwg::Label,

    #[nwg_control(
        position: (130, 50),
        size: (140, 140),
        flags: "VISIBLE|CENTER|TRANSPARENT"
    )]
    #[nwg_events(OnAnimationStart: [AnimationExample::on_anim_start], OnAnimationStop: [AnimationExample::on_anim_stop])]
    animation: nwg::Animation,

    #[nwg_control(text: "Play Once", position: (20, 200), size: (80, 30))]
    #[nwg_events(OnButtonClick: [AnimationExample::play_once])]
    play_once_btn: nwg::Button,

    #[nwg_control(text: "Play Loop", position: (110, 200), size: (80, 30))]
    #[nwg_events(OnButtonClick: [AnimationExample::play_loop])]
    play_loop_btn: nwg::Button,

    #[nwg_control(text: "Stop", position: (200, 200), size: (80, 30))]
    #[nwg_events(OnButtonClick: [AnimationExample::stop])]
    stop_btn: nwg::Button,

    #[nwg_control(text: "Animation:", position: (20, 245), size: (70, 25))]
    anim_label: nwg::Label,

    #[nwg_control(text: "Search", position: (90, 240), size: (120, 25))]
    #[nwg_events(OnButtonClick: [AnimationExample::load_search])]
    search_btn: nwg::Button,

    #[nwg_control(text: "Copy", position: (220, 240), size: (80, 25))]
    #[nwg_events(OnButtonClick: [AnimationExample::load_copy])]
    copy_btn: nwg::Button,

    #[nwg_control(text: "Delete", position: (310, 240), size: (70, 25))]
    #[nwg_events(OnButtonClick: [AnimationExample::load_delete])]
    delete_btn: nwg::Button,

    #[nwg_control(text: "Load AVI File...", position: (90, 275), size: (120, 25))]
    #[nwg_events(OnButtonClick: [AnimationExample::load_file])]
    load_file_btn: nwg::Button,

    #[nwg_control(text: "Ready", position: (20, 310), size: (360, 30))]
    status_label: nwg::Label,
}

impl AnimationExample {
    fn on_init(&self) {
        // Load shell32.dll to access built-in animations
        use winapi::um::libloaderapi::LoadLibraryW;
        use winapi::um::errhandlingapi::GetLastError;

        let shell32 = unsafe {
            let lib_name: Vec<u16> = "shell32.dll\0".encode_utf16().collect();
            LoadLibraryW(lib_name.as_ptr()) as isize
        };

        self.shell32_handle.set(shell32);

        // First try to load the extracted Windows XP animation (if available)
        // These are extracted from Windows XP shell32.dll and are RLE8 compressed
        let sample_paths = [
            "examples/extracted_avi_0.avi",
            "extracted_avi_0.avi",
            "../examples/extracted_avi_0.avi",
        ];

        let mut loaded = false;
        for path in &sample_paths {
            if self.animation.open_file(path) {
                self.status_label.set_text(&format!("Loaded: {}\nClick 'Play' to start!", path));
                loaded = true;
                break;
            }
        }

        // If no sample file found, try shell32 resources (probably won't work on Win10/11)
        if !loaded && shell32 != 0 {
            let ids_to_try = [SHELL32_SEARCH, SHELL32_COPY_FILE, SHELL32_DELETE, 150, 151, 152, 160, 161];

            for &id in &ids_to_try {
                if self.animation.open_resource_from_module(shell32, id) {
                    self.status_label.set_text(&format!("Loaded animation (resource {})\nClick 'Play' to start", id));
                    loaded = true;
                    break;
                }
            }
        }

        if !loaded {
            self.status_label.set_text(
                "No animation loaded.\n\
                Click 'Load AVI File...' to load an AVI.\n\
                (Must be uncompressed or RLE8 compressed)"
            );
        }
    }

    fn play_once(&self) {
        if self.animation.play_once() {
            self.status_label.set_text("Playing once...");
        } else {
            self.status_label.set_text("Failed to play (no animation loaded?)");
        }
    }

    fn play_loop(&self) {
        if self.animation.play_loop() {
            self.status_label.set_text("Playing in loop...");
        } else {
            self.status_label.set_text("Failed to play (no animation loaded?)");
        }
    }

    fn stop(&self) {
        if self.animation.stop() {
            self.status_label.set_text("Stopped");
        }
    }

    fn load_search(&self) {
        self.animation.stop();
        let handle = self.shell32_handle.get();
        if handle != 0 && self.animation.open_resource_from_module(handle, SHELL32_SEARCH) {
            self.status_label.set_text("Loaded: Search animation");
        }
    }

    fn load_copy(&self) {
        self.animation.stop();
        let handle = self.shell32_handle.get();
        if handle != 0 && self.animation.open_resource_from_module(handle, SHELL32_COPY_FILE) {
            self.status_label.set_text("Loaded: Copy animation");
        }
    }

    fn load_delete(&self) {
        self.animation.stop();
        let handle = self.shell32_handle.get();
        if handle != 0 && self.animation.open_resource_from_module(handle, SHELL32_DELETE) {
            self.status_label.set_text("Loaded: Delete animation");
        }
    }

    fn load_file(&self) {
        self.animation.stop();

        // Simple file dialog for AVI files
        use winapi::um::commdlg::{GetOpenFileNameW, OPENFILENAMEW, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST};
        use std::mem;

        let mut file_path = [0u16; 260];
        let filter: Vec<u16> = "AVI Files\0*.avi\0All Files\0*.*\0\0".encode_utf16().collect();

        let mut ofn: OPENFILENAMEW = unsafe { mem::zeroed() };
        ofn.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.lpstrFilter = filter.as_ptr();
        ofn.lpstrFile = file_path.as_mut_ptr();
        ofn.nMaxFile = 260;
        ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

        let result = unsafe { GetOpenFileNameW(&mut ofn) };

        if result != 0 {
            let path = String::from_utf16_lossy(&file_path)
                .trim_end_matches('\0')
                .to_string();

            if self.animation.open_file(&path) {
                self.status_label.set_text(&format!("Loaded: {}", path));
            } else {
                self.status_label.set_text("Failed to load AVI file.\nNote: Must be uncompressed or RLE-compressed.");
            }
        }
    }

    fn on_anim_start(&self) {
        let current = self.status_label.text();
        self.status_label.set_text(&format!("{}\n[Animation started]", current));
    }

    fn on_anim_stop(&self) {
        let current = self.status_label.text();
        self.status_label.set_text(&format!("{}\n[Animation stopped]", current));
    }

    fn exit(&self) {
        // Close the animation and free resources
        self.animation.close();
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _app = AnimationExample::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
