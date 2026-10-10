//! Looking for a newer release on GitHub: once when the program starts (can be switched off in the program
//! options) and on demand from Help > Check for updates…. A newer version opens a small window that leads to
//! the page with the release packages.
use crate::app::{App, APP_VERSION};
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::Duration;

/// Page with the packages of the newest release.
pub const RELEASES_URL: &str = "https://github.com/michalsarna/LightCreator/releases/latest";
const API_URL: &str = "https://api.github.com/repos/michalsarna/LightCreator/releases/latest";

pub enum Outcome {
    Newer(String),
    UpToDate(String),
    Failed(String),
}

#[derive(Default)]
pub struct UpdateState {
    rx: Option<Receiver<Result<String, String>>>,
    /// The check was asked for by hand: tell the result whatever it is.
    manual: bool,
    pub window: Option<Outcome>,
}

/// `v0.0.8` or `0.0.8` as numbers.
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let mut it = s.trim().trim_start_matches(['v', 'V']).split('.');
    let mut next = || it.next().and_then(|p| p.split(['-', '+']).next()).and_then(|p| p.parse::<u64>().ok());
    Some((next()?, next().unwrap_or(0), next().unwrap_or(0)))
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

/// The `tag_name` of the answer of the GitHub releases API.
pub fn latest_tag_from_json(body: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    v.get("tag_name")?.as_str().map(str::to_string)
}

fn fetch_latest() -> Result<String, String> {
    let resp = ureq::get(API_URL)
        .set("User-Agent", "LightCreator")
        .set("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(8))
        .call()
        .map_err(|e| e.to_string())?;
    let body = resp.into_string().map_err(|e| e.to_string())?;
    latest_tag_from_json(&body).ok_or_else(|| "no release found".to_string())
}

impl App {
    /// Ask GitHub for the newest release in the background. `manual` shows the answer even when there is nothing new.
    pub fn start_update_check(&mut self, ctx: &egui::Context, manual: bool) {
        if self.update.rx.is_some() {
            self.update.manual |= manual;
            return;
        }
        let (tx, rx) = channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(fetch_latest());
            ctx.request_repaint();
        });
        self.update.rx = Some(rx);
        self.update.manual = manual;
    }

    /// Pick up the answer and draw the window about it.
    pub fn update_tick(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.update.rx {
            match rx.try_recv() {
                Ok(res) => {
                    let manual = self.update.manual;
                    self.update.rx = None;
                    self.update.manual = false;
                    self.update.window = match res {
                        Ok(tag) if is_newer(&tag, APP_VERSION) => Some(Outcome::Newer(tag)),
                        Ok(tag) if manual => Some(Outcome::UpToDate(tag)),
                        Err(e) if manual => Some(Outcome::Failed(e)),
                        _ => None,
                    };
                }
                Err(TryRecvError::Empty) => ctx.request_repaint_after(Duration::from_millis(300)),
                Err(TryRecvError::Disconnected) => self.update.rx = None,
            }
        }
        let Some(outcome) = &self.update.window else { return };
        let mut open = true;
        let mut go = false;
        let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin { left: 10, right: 10, top: 6, bottom: 8 });
        egui::Window::new("update_window").title_bar(false).frame(frame).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.set_min_width(340.0);
            crate::window_ui::mini_title(ui, &tr("Updates"), &mut open, |_| {});
            match outcome {
                Outcome::Newer(tag) => {
                    ui.label(RichText::new(trf("A newer version is available: {}", &[tag])).strong());
                    ui.label(RichText::new(trf("You have {}.", &[&APP_VERSION])).color(theme::text_dim()));
                    ui.add_space(8.0);
                    let btn = egui::Button::new(RichText::new(tr("Open the download page")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                    go = ui.add_sized([ui.available_width(), 30.0], btn).clicked();
                }
                Outcome::UpToDate(tag) => {
                    ui.label(RichText::new(tr("You have the newest version.")).strong());
                    ui.label(RichText::new(trf("Newest release: {}", &[tag])).color(theme::text_dim()));
                }
                Outcome::Failed(e) => {
                    ui.label(RichText::new(tr("Could not check for updates.")).strong());
                    ui.label(RichText::new(e.as_str()).color(theme::text_dim()));
                }
            }
        });
        if go {
            ctx.open_url(egui::OpenUrl::new_tab(RELEASES_URL));
            open = false;
        }
        if !open {
            self.update.window = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_as_numbers() {
        assert_eq!(parse_version("v0.0.8"), Some((0, 0, 8)));
        assert_eq!(parse_version("1.2"), Some((1, 2, 0)));
        assert_eq!(parse_version("v0.0.10-rc1"), Some((0, 0, 10)));
        assert_eq!(parse_version("nonsense"), None);
        assert!(is_newer("v0.0.9", "v0.0.8"));
        assert!(is_newer("v0.0.10", "v0.0.9"), "10 is more than 9, not less as text");
        assert!(is_newer("v0.1.0", "v0.0.99"));
        assert!(!is_newer("v0.0.8", "v0.0.8"));
        assert!(!is_newer("v0.0.7", "v0.0.8"));
        assert!(!is_newer("garbage", "v0.0.8"));
    }

    #[test]
    fn the_tag_is_read_from_the_github_answer() {
        assert_eq!(latest_tag_from_json(r#"{"id":1,"tag_name":"v0.0.9","name":"x"}"#), Some("v0.0.9".into()));
        assert_eq!(latest_tag_from_json(r#"{"message":"Not Found"}"#), None);
        assert_eq!(latest_tag_from_json("not json"), None);
    }

    #[test]
    fn a_newer_answer_opens_the_window_and_an_old_one_stays_quiet() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        for (tag, manual, shown) in [("v9.9.9", false, true), (APP_VERSION, false, false), (APP_VERSION, true, true)] {
            let (tx, rx) = channel();
            tx.send(Ok(tag.to_string())).unwrap();
            a.update.rx = Some(rx);
            a.update.manual = manual;
            a.update.window = None;
            let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
            a.update_tick(&ctx);
            out.textures_delta.clear();
            assert_eq!(a.update.window.is_some(), shown, "{tag} manual={manual}");
        }
        // An error is only reported when asked for by hand.
        let (tx, rx) = channel();
        tx.send(Err("offline".to_string())).unwrap();
        a.update.rx = Some(rx);
        a.update.manual = false;
        a.update.window = None;
        a.update_tick(&ctx);
        assert!(a.update.window.is_none());
    }
}
