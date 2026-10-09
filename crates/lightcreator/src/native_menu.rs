//! Native macOS menu bar built with `muda` (the same crate VectorCraft uses). On Windows and Linux the
//! in-window egui menu bar is used instead, so no GTK dependency is needed.
use crate::i18n::{lang, tr};
use crate::menu::{menus, Act, Entry};
use eframe::egui;
use muda::{accelerator::Accelerator, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use std::sync::Mutex;

static QUEUE: Mutex<Vec<String>> = Mutex::new(Vec::new());

enum Handle {
    Plain(MenuItem),
    Check(CheckMenuItem),
}

pub struct NativeMenu {
    _menu: Menu,
    items: Vec<(Act, Handle)>,
    built_for: crate::i18n::Lang,
}

fn id(a: Act) -> String {
    format!("{a:?}")
}

fn is_check(a: Act) -> bool {
    matches!(a, Act::ToggleGrid | Act::ToggleSnap | Act::TogglePreview | Act::SetLang(_) | Act::SetScheme(_))
}

/// Accelerators the system menu owns. Editing shortcuts (copy, paste, delete...) stay with egui so text
/// fields keep working; the egui shortcut handler skips the ones listed here.
pub fn owns_shortcut(a: Act) -> bool {
    matches!(a, Act::New | Act::Open | Act::Save | Act::SaveAs | Act::ImportSvg)
}

fn append(sub: &Submenu, entries: &[Entry], items: &mut Vec<(Act, Handle)>) {
    for e in entries {
        match e {
            Entry::Sep => {
                let _ = sub.append(&PredefinedMenuItem::separator());
            }
            Entry::Sub(title, children) => {
                let s = Submenu::new(tr(title), true);
                append(&s, children, items);
                let _ = sub.append(&s);
            }
            Entry::Item(act, label, accel) => {
                let accel: Option<Accelerator> = if owns_shortcut(*act) { accel.and_then(|(a, _)| a.parse().ok()) } else { None };
                if is_check(*act) {
                    let it = CheckMenuItem::with_id(id(*act), tr(label), true, false, accel);
                    let _ = sub.append(&it);
                    items.push((*act, Handle::Check(it)));
                } else {
                    let it = MenuItem::with_id(id(*act), tr(label), true, accel);
                    let _ = sub.append(&it);
                    items.push((*act, Handle::Plain(it)));
                }
            }
        }
    }
}

impl NativeMenu {
    pub fn install(ctx: &egui::Context) -> NativeMenu {
        let ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            if let Ok(mut q) = QUEUE.lock() {
                q.push(e.id.0.clone());
            }
            ctx.request_repaint();
        }));
        Self::build()
    }

    fn build() -> NativeMenu {
        let menu = Menu::new();
        let mut items = Vec::new();
        // Application menu (the one named after the app, left of File).
        let app = Submenu::new("LightCreator", true);
        let about = MenuItem::with_id(id(Act::About), tr("About LightCreator"), true, None);
        let _ = app.append(&about);
        items.push((Act::About, Handle::Plain(about)));
        let _ = app.append(&PredefinedMenuItem::separator());
        let _ = app.append(&PredefinedMenuItem::services(None));
        let _ = app.append(&PredefinedMenuItem::separator());
        let _ = app.append(&PredefinedMenuItem::hide(None));
        let _ = app.append(&PredefinedMenuItem::hide_others(None));
        let _ = app.append(&PredefinedMenuItem::show_all(None));
        let _ = app.append(&PredefinedMenuItem::separator());
        let _ = app.append(&PredefinedMenuItem::quit(None));
        let _ = menu.append(&app);
        for (title, entries) in menus() {
            let sub = Submenu::new(tr(title), true);
            append(&sub, &entries, &mut items);
            let _ = menu.append(&sub);
        }
        let window = Submenu::new("Window", true);
        let _ = window.append(&PredefinedMenuItem::minimize(None));
        let _ = window.append(&PredefinedMenuItem::maximize(None));
        let _ = menu.append(&window);
        menu.init_for_nsapp();
        window.set_as_windows_menu_for_nsapp();
        NativeMenu { _menu: menu, items, built_for: lang() }
    }

    /// Actions the user picked in the menu since the last call.
    pub fn take_actions(&self) -> Vec<Act> {
        let ids: Vec<String> = QUEUE.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
        ids.iter().filter_map(|i| self.items.iter().find(|(a, _)| &id(*a) == i).map(|(a, _)| *a)).collect()
    }

    /// Rebuild after a language change and mirror check marks / enabled state from the app.
    pub fn sync(&mut self, checked: impl Fn(Act) -> Option<bool>, enabled: impl Fn(Act) -> bool) {
        if self.built_for != lang() {
            *self = Self::build();
        }
        for (act, h) in &self.items {
            match h {
                Handle::Check(c) => {
                    if let Some(v) = checked(*act) {
                        if c.is_checked() != v {
                            c.set_checked(v);
                        }
                    }
                }
                Handle::Plain(p) => {
                    let en = enabled(*act);
                    if p.is_enabled() != en {
                        p.set_enabled(en);
                    }
                }
            }
        }
    }
}

/// Show the LightCreator icon in the Dock (running outside an .app bundle gives no icon otherwise).
pub fn set_dock_icon(png: &[u8]) {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;
    let Some(mtm) = MainThreadMarker::new() else { return };
    let data = NSData::with_bytes(png);
    if let Some(img) = NSImage::initWithData(NSImage::alloc(), &data) {
        // SAFETY: called on the main thread with a valid NSImage.
        unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&img)) };
    }
}
