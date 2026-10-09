//! Bézier node editing: operations on the selected nodes and the node-tool bar.
use crate::app::App;
use crate::i18n::tr;
use eframe::egui;
use lc_core::{Contour, Kind, Node, Xf};

/// A node address: shape id, contour index, node index.
pub type NodeRef = (u64, usize, usize);

impl App {
    /// Make every selected shape an editable Bézier shape with its transform baked in.
    pub fn nodes_prepare(&mut self) {
        let todo: Vec<u64> = self
            .sel
            .iter()
            .copied()
            .filter(|id| {
                self.doc.shape(*id).is_some_and(|s| {
                    let editable = !matches!(s.kind, Kind::Image { .. });
                    editable && (!matches!(s.kind, Kind::Bezier(_)) || s.xf != Xf::IDENTITY)
                })
            })
            .collect();
        if todo.is_empty() {
            return;
        }
        self.checkpoint();
        for id in todo {
            if let Some(s) = self.doc.shape_mut(id) {
                s.to_bezier();
            }
        }
        self.node_sel.clear();
    }

    pub fn contour_mut(&mut self, id: u64, ci: usize) -> Option<&mut Contour> {
        match &mut self.doc.shape_mut(id)?.kind {
            Kind::Bezier(cs) => cs.get_mut(ci),
            _ => None,
        }
    }

    pub fn node_at(&self, r: NodeRef) -> Option<Node> {
        match &self.doc.shape(r.0)?.kind {
            Kind::Bezier(cs) => cs.get(r.1)?.nodes.get(r.2).copied(),
            _ => None,
        }
    }

    /// Selected nodes grouped per contour, highest index first (safe for removals).
    fn sel_sorted(&self) -> Vec<NodeRef> {
        let mut v = self.node_sel.clone();
        v.sort_by(|a, b| (b.0, b.1, b.2).cmp(&(a.0, a.1, a.2)));
        v.dedup();
        v
    }

    pub fn delete_nodes(&mut self) {
        if self.node_sel.is_empty() {
            return;
        }
        self.checkpoint();
        for (id, ci, ni) in self.sel_sorted() {
            if let Some(c) = self.contour_mut(id, ci) {
                c.remove_node(ni);
            }
        }
        // Drop contours that no longer make sense, and shapes that became empty.
        for s in &mut self.doc.shapes {
            if let Kind::Bezier(cs) = &mut s.kind {
                cs.retain(|c| c.nodes.len() >= 2);
            }
        }
        self.doc.shapes.retain(|s| !matches!(&s.kind, Kind::Bezier(cs) if cs.is_empty()));
        self.sel.retain(|id| self.doc.shape(*id).is_some());
        self.node_sel.clear();
        self.active_seg = None;
        self.touch();
    }

    pub fn nodes_smooth(&mut self, smooth: bool) {
        if self.node_sel.is_empty() {
            return;
        }
        self.checkpoint();
        for (id, ci, ni) in self.node_sel.clone() {
            if let Some(c) = self.contour_mut(id, ci) {
                if ni < c.nodes.len() {
                    if smooth {
                        c.make_smooth(ni);
                    } else {
                        c.make_corner(ni);
                    }
                }
            }
        }
        self.touch();
    }

    /// Turn the segment after each selected node (or the clicked segment) into a line or a curve.
    pub fn segments_set(&mut self, curve: bool) {
        let mut segs: Vec<NodeRef> = self.node_sel.clone();
        if let Some(s) = self.active_seg {
            segs.push(s);
        }
        if segs.is_empty() {
            return;
        }
        self.checkpoint();
        for (id, ci, si) in segs {
            if let Some(c) = self.contour_mut(id, ci) {
                if si < c.seg_count() {
                    if curve {
                        c.segment_to_curve(si);
                    } else {
                        c.segment_to_line(si);
                    }
                }
            }
        }
        self.touch();
    }

    /// Insert a node on the clicked segment, or at the middle of the segment after each selected node.
    pub fn nodes_insert(&mut self, at: Option<(NodeRef, f64)>) {
        self.checkpoint();
        match at {
            Some(((id, ci, si), t)) => {
                if let Some(c) = self.contour_mut(id, ci) {
                    if si < c.seg_count() {
                        let ni = c.insert_node(si, t);
                        self.node_sel = vec![(id, ci, ni)];
                    }
                }
            }
            None => {
                let mut sel = self.sel_sorted();
                sel.retain(|(id, ci, ni)| self.contour_mut(*id, *ci).is_some_and(|c| *ni < c.seg_count()));
                let mut fresh = vec![];
                for (id, ci, si) in sel {
                    if let Some(c) = self.contour_mut(id, ci) {
                        let ni = c.insert_node(si, 0.5);
                        fresh.push((id, ci, ni));
                    }
                }
                self.node_sel = fresh;
            }
        }
        self.active_seg = None;
        self.touch();
    }

    /// Close an open contour, or open a closed one at the selected node.
    pub fn nodes_toggle_closed(&mut self) {
        let Some(&(id, ci, ni)) = self.node_sel.first() else { return };
        self.checkpoint();
        if let Some(c) = self.contour_mut(id, ci) {
            if c.closed {
                // Break at the selected node: it becomes both end points.
                c.nodes.rotate_left(ni);
                let mut end = c.nodes[0];
                end.hout = end.p;
                c.nodes[0].hin = c.nodes[0].p;
                c.nodes.push(end);
                c.closed = false;
            } else if c.nodes.len() >= 3 {
                c.closed = true;
            }
        }
        self.node_sel.clear();
        self.touch();
    }

    pub fn node_bar(&mut self, ui: &mut egui::Ui) {
        let has_nodes = !self.node_sel.is_empty();
        let has_seg = has_nodes || self.active_seg.is_some();
        if ui.add_enabled(has_nodes, egui::Button::new(tr("Corner"))).on_hover_text(tr("Make the selected nodes corners")).clicked() {
            self.nodes_smooth(false);
        }
        if ui.add_enabled(has_nodes, egui::Button::new(tr("Smooth"))).on_hover_text(tr("Make the selected nodes smooth")).clicked() {
            self.nodes_smooth(true);
        }
        if ui.add_enabled(has_seg, egui::Button::new(tr("To line"))).on_hover_text(tr("Make the segment after each selected node straight")).clicked() {
            self.segments_set(false);
        }
        if ui.add_enabled(has_seg, egui::Button::new(tr("To curve"))).on_hover_text(tr("Make the segment after each selected node a curve")).clicked() {
            self.segments_set(true);
        }
        if ui.add_enabled(has_seg, egui::Button::new(tr("Add node"))).on_hover_text(tr("Insert a node (or double-click a segment)")).clicked() {
            let at = self.active_seg.map(|s| (s, 0.5));
            self.nodes_insert(at);
        }
        if ui.add_enabled(has_nodes, egui::Button::new(tr("Delete node"))).clicked() {
            self.delete_nodes();
        }
        if ui.add_enabled(has_nodes, egui::Button::new(tr("Open / close"))).on_hover_text(tr("Close an open path, or open a closed one at the selected node")).clicked() {
            self.nodes_toggle_closed();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui;
    use lc_core::ops::BoolOp;

    fn app() -> App {
        App::build(&egui::Context::default(), None, false)
    }

    fn nodes_of(a: &App, id: u64) -> usize {
        match &a.doc.shape(id).unwrap().kind {
            Kind::Bezier(cs) => cs[0].nodes.len(),
            _ => 0,
        }
    }

    #[test]
    fn delete_insert_and_undo_nodes() {
        let mut a = app();
        let id = a.doc.add(0, Kind::Ellipse { w: 20.0, h: 10.0 }, Xf::translate(5.0, 5.0));
        a.sel = vec![id];
        a.nodes_prepare();
        assert_eq!(nodes_of(&a, id), 4);
        a.node_sel = vec![(id, 0, 0)];
        a.delete_nodes();
        assert_eq!(nodes_of(&a, id), 3);
        a.do_undo();
        assert_eq!(nodes_of(&a, id), 4);
        a.nodes_insert(Some(((id, 0, 1), 0.5)));
        assert_eq!(nodes_of(&a, id), 5);
        // Smooth then corner changes the flag only.
        a.node_sel = vec![(id, 0, 2)];
        a.nodes_smooth(false);
        assert!(!a.node_at((id, 0, 2)).unwrap().smooth);
    }

    #[test]
    fn boolean_ops_replace_selection_with_one_shape() {
        let mut a = app();
        let s1 = a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(0.0, 0.0));
        let s2 = a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(5.0, 0.0));
        a.sel = vec![s1, s2];
        a.bool_op(BoolOp::Union);
        assert_eq!(a.doc.shapes.len(), 1);
        let b = a.doc.shapes[0].bounds().unwrap();
        assert!((b.width() - 15.0).abs() < 1e-6);
        a.do_undo();
        assert_eq!(a.doc.shapes.len(), 2);
    }

    #[test]
    fn offset_creates_a_new_shape() {
        let mut a = app();
        let s = a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(10.0, 10.0));
        a.sel = vec![s];
        a.offset_selected(-1.0, true);
        assert_eq!(a.doc.shapes.len(), 2);
        let b = a.doc.shape(a.sel[0]).unwrap().bounds().unwrap();
        assert!((b.width() - 8.0).abs() < 0.05, "{}", b.width());
    }
}
