//! Document-level operations: boolean shape operations, offsetting, curve conversion, images and text.
use crate::app::App;
use crate::i18n::{tr, trf};
use eframe::egui;
use lc_core::ops::{self, BoolOp};
use lc_core::{ImageData, Kind, Polyline, Pt, TextData, Xf};
use std::path::PathBuf;

impl App {
    fn status_is(&mut self, s: impl Into<String>) {
        self.status = s.into();
    }

    /// Combine the selected shapes (in selection order) with a boolean operation.
    pub fn bool_op(&mut self, op: BoolOp) {
        let ids: Vec<u64> = self.sel.iter().copied().filter(|id| self.doc.shape(*id).is_some_and(|s| !s.is_image())).collect();
        if ids.len() < 2 {
            self.status_is(tr("Select at least two shapes (not images)."));
            return;
        }
        let polys_of = |app: &App, id: u64| -> Vec<Polyline> { app.doc.shape(id).map(|s| s.polys()).unwrap_or_default() };
        let mut acc = polys_of(self, ids[0]);
        for id in &ids[1..] {
            let next = polys_of(self, *id);
            acc = ops::boolean(&acc, &next, op);
        }
        if acc.is_empty() {
            self.status_is(tr("The result is empty."));
            return;
        }
        let layer = self.doc.shape(ids[0]).map(|s| s.layer).unwrap_or(self.active_layer);
        self.checkpoint();
        self.doc.shapes.retain(|s| !ids.contains(&s.id));
        let id = self.doc.add(layer, Kind::Path(acc), Xf::IDENTITY);
        self.sel = vec![id];
        self.node_sel.clear();
    }

    /// Offset the outlines of the selected shapes by `d` mm (negative shrinks).
    pub fn offset_selected(&mut self, d: f64, keep_original: bool) {
        let ids: Vec<u64> = self.sel.clone();
        let mut made = vec![];
        let mut created: Vec<(usize, Vec<Polyline>)> = vec![];
        for id in &ids {
            let Some(s) = self.doc.shape(*id) else { continue };
            if s.is_image() {
                continue;
            }
            let res = ops::offset(&s.polys(), d);
            if !res.is_empty() {
                created.push((s.layer, res));
            }
        }
        if created.is_empty() {
            self.status_is(tr("The result is empty."));
            return;
        }
        self.checkpoint();
        if !keep_original {
            self.doc.shapes.retain(|s| !ids.contains(&s.id));
        }
        for (layer, polys) in created {
            made.push(self.doc.add(layer, Kind::Path(polys), Xf::IDENTITY));
        }
        self.sel = made;
        self.node_sel.clear();
    }

    pub fn to_curves(&mut self) {
        if self.sel.is_empty() {
            return;
        }
        self.checkpoint();
        for id in self.sel.clone() {
            if let Some(s) = self.doc.shape_mut(id) {
                if !s.is_image() {
                    s.to_bezier();
                }
            }
        }
        self.node_sel.clear();
    }

    pub fn import_image(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "webp"]).pick_file() {
            self.import_image_path(p);
        }
    }

    pub fn import_image_path(&mut self, p: PathBuf) {
        let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let (bw, bh) = (self.doc.device.bed_w, self.doc.device.bed_h);
        let longest = (bw.min(bh) * 0.5).clamp(10.0, 200.0);
        match std::fs::read(&p).map_err(|e| e.to_string()).and_then(|b| ImageData::from_bytes(&b, &name, longest)) {
            Ok(im) => {
                self.checkpoint();
                let (w, h) = (im.w, im.h);
                let id = self.doc.add(self.active_layer, Kind::Image(im), Xf::translate((bw - w) / 2.0, (bh - h) / 2.0));
                self.sel = vec![id];
                self.status_is(trf("Imported image {}", &[&name]));
            }
            Err(e) => self.status_is(trf("Import failed: {}", &[&e])),
        }
    }

    /// Create a text object with its top-left corner at `at`.
    pub fn add_text_at(&mut self, at: Pt) {
        self.checkpoint();
        let t: TextData = self.text_default.clone();
        let id = self.doc.add(self.active_layer, Kind::Text(t), Xf::translate(at.x, at.y));
        self.sel = vec![id];
        self.side_tab = crate::app::SideTab::Properties;
        self.focus_text = true;
    }

    /// Handle files dropped onto the window.
    pub fn handle_drops(&mut self, ctx: &egui::Context) {
        let files: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        for p in files {
            let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            match ext.as_str() {
                "png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp" => self.import_image_path(p),
                "svg" | "lcr" => self.open_path(p),
                _ => {}
            }
        }
    }
}
