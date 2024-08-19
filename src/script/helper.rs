#[cfg(target_os = "macos")]
use std::{error::Error, ffi::c_void};

#[cfg(target_os = "macos")]
use {
    log::trace,
    objc2::msg_send,
    objc2_app_kit::{NSEvent, NSEventModifierFlags, NSSystemDefined},
    objc2_foundation::NSPoint,
};

use enigo::Key;

pub fn map_str_to_key(s: &str) -> Key {
    match s {
        "Alt" => Key::Alt,
        "Backspace" => Key::Backspace,
        // "Begin" => Key::Begin,
        // "Break" => Key::Break,
        // "Cancel" => Key::Cancel,
        "CapsLock" => Key::CapsLock,
        // "Clear" => Key::Clear,
        "Control" => Key::Control,
        "Delete" => Key::Delete,
        "DownArrow" => Key::DownArrow,
        "End" => Key::End,
        "Escape" => Key::Escape,
        // "Execute" => Key::Execute,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        "F13" => Key::F13,
        "F14" => Key::F14,
        "F15" => Key::F15,
        "F16" => Key::F16,
        "F17" => Key::F17,
        "F18" => Key::F18,
        "F19" => Key::F19,
        "F20" => Key::F20,
        // "F21" => Key::F21,
        // "F22" => Key::F22,
        // "F23" => Key::F23,
        // "F24" => Key::F24,
        // "F25" => Key::F25,
        // "F26" => Key::F26,
        // "F27" => Key::F27,
        // "F28" => Key::F28,
        // "F29" => Key::F29,
        // "F30" => Key::F30,
        // "F31" => Key::F31,
        // "F32" => Key::F32,
        // "F33" => Key::F33,
        // "F34" => Key::F34,
        // "F35" => Key::F35,
        // "Find" => Key::Find,
        // "Hangul" => Key::Hangul,
        // "Hanja" => Key::Hanja,
        "Help" => Key::Help,
        "Home" => Key::Home,
        // "Insert" => Key::Insert,
        // "Kanji" => Key::Kanji,
        "LControl" => Key::LControl,
        "LeftArrow" => Key::LeftArrow,
        // "Linefeed" => Key::Linefeed,
        // "LMenu" => Key::LMenu,
        "LShift" => Key::LShift,
        "Meta" => Key::Meta,
        // "ModeChange" => Key::ModeChange,
        // "Numlock" => Key::Numlock,
        "Option" => Key::Option,
        "PageDown" => Key::PageDown,
        "PageUp" => Key::PageUp,
        // "Pause" => Key::Pause,
        // "Print" => Key::Print,
        "RControl" => Key::RControl,
        // "Redo" => Key::Redo,
        "Return" => Key::Return,
        "RightArrow" => Key::RightArrow,
        "RShift" => Key::RShift,
        // "ScrollLock" => Key::ScrollLock,
        // "Select" => Key::Select,
        // "ScriptSwitch" => Key::ScriptSwitch,
        "Shift" => Key::Shift,
        // "ShiftLock" => Key::ShiftLock,
        "Space" => Key::Space,
        // "SysReq" => Key::SysReq,
        "Tab" => Key::Tab,
        // "Undo" => Key::Undo,
        "UpArrow" => Key::UpArrow,
        _ => Key::Layout(s.as_bytes()[0] as char),
    }
}

#[link(name = "CoreGraphics", kind = "framework")]
#[cfg(target_os = "macos")]
extern "C" {
    fn CGEventPost(tap: u32, event: *mut c_void);
}

#[cfg(target_os = "macos")]
pub fn hid_post_aux_key(key: u32, down: bool) -> Result<(), Box<dyn Error>> {
    unsafe {
        let event = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
            NSSystemDefined,
            NSPoint { x: 0., y: 0. },
            NSEventModifierFlags::empty(),
            0.0,
            0,
            None,
            8,
            ((key << 16) | ((if down { 0xa } else { 0xb }) << 8)) as isize,
            -1
            ).unwrap();

        let cgevent: *mut c_void = msg_send![&event, CGEvent];

        trace!("Sending CGEvent: {:?}", event);
        CGEventPost(0, cgevent);
    }

    Ok(())
}
