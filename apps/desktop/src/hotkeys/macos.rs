//! Carbon's default registration is non-exclusive. Use the documented exclusive
//! option so another app's binding is a conflict rather than a second recipient.
//! global-hotkey supplies the main-thread Carbon event handler; we own the refs.
use super::Backend;
use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use std::{collections::HashMap, ffi::c_void, ptr};

#[repr(C)]
struct HotKeyId {
    signature: u32,
    id: u32,
}
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn GetApplicationEventTarget() -> *mut c_void;
    fn RegisterEventHotKey(
        code: u32,
        modifiers: u32,
        id: HotKeyId,
        target: *mut c_void,
        options: u32,
        output: *mut *mut c_void,
    ) -> i32;
    fn UnregisterEventHotKey(reference: *mut c_void) -> i32;
}
pub(super) struct Native {
    _events: GlobalHotKeyManager,
    references: HashMap<u32, *mut c_void>,
}
impl Native {
    pub(super) fn new() -> Result<Self, String> {
        if objc2::MainThreadMarker::new().is_none() {
            return Err("Hotkeys benötigen den Hauptthread.".into());
        }
        Ok(Self {
            _events: GlobalHotKeyManager::new().map_err(|e| e.to_string())?,
            references: HashMap::new(),
        })
    }
}
impl Backend for Native {
    fn register(&mut self, key: HotKey) -> Result<(), String> {
        if self.references.contains_key(&key.id()) {
            return Err("Bereits registriert.".into());
        }
        let code =
            virtual_key(key.key).ok_or_else(|| "Taste unter macOS nicht verfügbar.".to_string())?;
        let mut modifiers = 0;
        for (modifier, mask) in [
            (Modifiers::SHIFT, 512),
            (Modifiers::CONTROL, 4096),
            (Modifiers::ALT, 2048),
            (Modifiers::SUPER, 256),
        ] {
            if key.mods.contains(modifier) {
                modifiers |= mask;
            }
        }
        let mut reference = ptr::null_mut();
        // SAFETY: called on the main thread, ABI matches CarbonEvents.h. The ref
        // belongs to this backend. Option 1 is kEventHotKeyExclusive (10.5+).
        let result = unsafe {
            RegisterEventHotKey(
                code,
                modifiers,
                HotKeyId {
                    signature: u32::from_be_bytes(*b"beni"),
                    id: key.id(),
                },
                GetApplicationEventTarget(),
                1,
                &mut reference,
            )
        };
        if result != 0 || reference.is_null() {
            return Err(format!("Carbon-Registrierung fehlgeschlagen: {result}"));
        }
        self.references.insert(key.id(), reference);
        Ok(())
    }
    fn unregister(&mut self, key: HotKey) -> Result<(), String> {
        let Some(reference) = self.references.get(&key.id()).copied() else {
            return Ok(());
        };
        // SAFETY: only live refs returned by our successful registration are used.
        let result = unsafe { UnregisterEventHotKey(reference) };
        if result != 0 {
            return Err(format!("Carbon-Freigabe fehlgeschlagen: {result}"));
        }
        self.references.remove(&key.id());
        Ok(())
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        for reference in self.references.drain().map(|(_, reference)| reference) {
            // SAFETY: final attempt for still-owned refs, before the event handler drops.
            let _ = unsafe { UnregisterEventHotKey(reference) };
        }
    }
}

// Physical US key positions, from Apple's Events.h kVK_ANSI_* / kVK_* values.
fn virtual_key(key: Code) -> Option<u32> {
    use Code::*;
    Some(match key {
        KeyA => 0x00,
        KeyS => 0x01,
        KeyD => 0x02,
        KeyF => 0x03,
        KeyH => 0x04,
        KeyG => 0x05,
        KeyZ => 0x06,
        KeyX => 0x07,
        KeyC => 0x08,
        KeyV => 0x09,
        KeyB => 0x0b,
        KeyQ => 0x0c,
        KeyW => 0x0d,
        KeyE => 0x0e,
        KeyR => 0x0f,
        KeyY => 0x10,
        KeyT => 0x11,
        Digit1 => 0x12,
        Digit2 => 0x13,
        Digit3 => 0x14,
        Digit4 => 0x15,
        Digit6 => 0x16,
        Digit5 => 0x17,
        Equal => 0x18,
        Digit9 => 0x19,
        Digit7 => 0x1a,
        Minus => 0x1b,
        Digit8 => 0x1c,
        Digit0 => 0x1d,
        BracketRight => 0x1e,
        KeyO => 0x1f,
        KeyU => 0x20,
        BracketLeft => 0x21,
        KeyI => 0x22,
        KeyP => 0x23,
        Enter => 0x24,
        KeyL => 0x25,
        KeyJ => 0x26,
        Quote => 0x27,
        KeyK => 0x28,
        Semicolon => 0x29,
        Backslash => 0x2a,
        Comma => 0x2b,
        Slash => 0x2c,
        KeyN => 0x2d,
        KeyM => 0x2e,
        Period => 0x2f,
        Tab => 0x30,
        Space => 0x31,
        Backquote => 0x32,
        Backspace => 0x33,
        Escape => 0x35,
        F17 => 0x40,
        F18 => 0x4f,
        F19 => 0x50,
        F20 => 0x5a,
        F5 => 0x60,
        F6 => 0x61,
        F7 => 0x62,
        F3 => 0x63,
        F8 => 0x64,
        F9 => 0x65,
        F11 => 0x67,
        F13 => 0x69,
        F16 => 0x6a,
        F14 => 0x6b,
        F10 => 0x6d,
        F12 => 0x6f,
        F15 => 0x71,
        Insert => 0x72,
        Home => 0x73,
        PageUp => 0x74,
        Delete => 0x75,
        F4 => 0x76,
        End => 0x77,
        F2 => 0x78,
        PageDown => 0x79,
        F1 => 0x7a,
        ArrowLeft => 0x7b,
        ArrowRight => 0x7c,
        ArrowDown => 0x7d,
        ArrowUp => 0x7e,
        _ => return None,
    })
}
