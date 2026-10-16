//! Tests of node editing (the operations live in base-ui).
#[cfg(test)]
use base_ui::editor::EditorShapeOps;
use crate::app::App;
#[cfg(test)]
use base_ui::clipart_ui::ClipGallery;
#[cfg(test)]
use base_ui::editor::EditorHost;
#[cfg(test)]
use lc_core::{Kind, Xf};

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
        a.ed.sel = vec![id];
        a.nodes_prepare();
        assert_eq!(nodes_of(&a, id), 4);
        a.ed.node_sel = vec![(id, 0, 0)];
        a.delete_nodes();
        assert_eq!(nodes_of(&a, id), 3);
        a.do_undo();
        assert_eq!(nodes_of(&a, id), 4);
        a.nodes_insert(Some(((id, 0, 1), 0.5)));
        assert_eq!(nodes_of(&a, id), 5);
        // Smooth then corner changes the flag only.
        a.ed.node_sel = vec![(id, 0, 2)];
        a.nodes_smooth(false);
        assert!(!a.node_at((id, 0, 2)).unwrap().smooth);
    }

    #[test]
    fn boolean_ops_replace_selection_with_one_shape() {
        let mut a = app();
        let s1 = a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(0.0, 0.0));
        let s2 = a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(5.0, 0.0));
        a.ed.sel = vec![s1, s2];
        a.bool_op(BoolOp::Union);
        assert_eq!(a.doc.scene.shapes.len(), 1);
        let b = a.doc.scene.shapes[0].bounds().unwrap();
        assert!((b.width() - 15.0).abs() < 1e-6);
        a.do_undo();
        assert_eq!(a.doc.scene.shapes.len(), 2);
    }

    #[test]
    fn offset_creates_a_new_shape() {
        let mut a = app();
        let s = a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(10.0, 10.0));
        a.ed.sel = vec![s];
        a.offset_selected(-1.0, true);
        assert_eq!(a.doc.scene.shapes.len(), 2);
        let b = a.doc.shape(a.ed.sel[0]).unwrap().bounds().unwrap();
        assert!((b.width() - 8.0).abs() < 0.05, "{}", b.width());
    }

    #[test]
    fn gallery_pictures_can_be_edited_node_by_node() {
        let mut a = app();
        let svg = crate::clipart_ui::BUILTIN_CLIPART.iter().find(|c| c.1 == "cat").map(|c| c.2).expect("cat clipart");
        a.place_clip(svg, "cat");
        let ids = a.ed.sel.clone();
        assert!(!ids.is_empty());
        let before = a.doc.bounds_of(&ids).unwrap();
        // Entering the node tool bakes the placement scale into the nodes without moving anything.
        a.ed.tool = crate::app::Tool::Node;
        a.nodes_prepare();
        for id in &ids {
            let s = a.doc.shape(*id).unwrap();
            assert!(matches!(s.kind, Kind::Bezier(_)) && s.xf == Xf::IDENTITY);
        }
        let after = a.doc.bounds_of(&ids).unwrap();
        assert!((before.width() - after.width()).abs() < 1e-6 && (before.min.x - after.min.x).abs() < 1e-6);
        // Move a node, delete a node, undo.
        let id = ids[0];
        let n0 = a.node_at((id, 0, 0)).unwrap();
        a.contour_mut(id, 0).unwrap().nodes[0].translate(3.0, 0.0);
        assert!((a.node_at((id, 0, 0)).unwrap().p.x - n0.p.x - 3.0).abs() < 1e-9);
        let count = |a: &App| match &a.doc.shape(id).unwrap().kind {
            Kind::Bezier(cs) => cs[0].nodes.len(),
            _ => 0,
        };
        let n = count(&a);
        assert!(n > 3);
        a.ed.node_sel = vec![(id, 0, 1)];
        a.delete_nodes();
        assert_eq!(count(&a), n - 1);
        a.do_undo();
        assert_eq!(count(&a), n);
        // The picture's parts still form one group.
        assert_eq!(a.doc.group_of(id).len(), ids.len());
    }

    #[test]
    fn edit_nodes_command_switches_tool_and_text_is_never_converted_silently() {
        let mut a = app();
        a.start_node_edit();
        assert_eq!(a.ed.tool, crate::app::Tool::Select, "needs a selection first");
        let t = a.doc.add(0, Kind::Text(lc_core::TextData::default()), Xf::IDENTITY);
        a.ed.sel = vec![t];
        a.start_node_edit();
        assert_eq!(a.ed.tool, crate::app::Tool::Node);
        a.nodes_prepare();
        assert!(matches!(a.doc.shape(t).unwrap().kind, Kind::Text(_)), "text stays text");
        assert!(a.status.contains("Convert text to curves"));
        // After converting it, nodes are editable.
        a.to_curves();
        a.nodes_prepare();
        assert!(matches!(a.doc.shape(t).unwrap().kind, Kind::Bezier(_)));
    }
}
