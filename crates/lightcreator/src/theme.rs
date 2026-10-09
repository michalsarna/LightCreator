//! Light, flat look modelled on VectorCraft: neutral grey chrome, white artboard, red-orange accent.
use eframe::egui::{self, Color32, CornerRadius, Stroke};

pub const ACCENT: Color32 = Color32::from_rgb(0xe8, 0x57, 0x3f);
pub const PANEL: Color32 = Color32::from_rgb(0xf4, 0xf4, 0xf5);
pub const PANEL_DARK: Color32 = Color32::from_rgb(0xe9, 0xe9, 0xeb);
pub const BORDER: Color32 = Color32::from_rgb(0xd2, 0xd2, 0xd6);
pub const WORKSPACE: Color32 = Color32::from_rgb(0xdc, 0xdc, 0xe0);
pub const TEXT: Color32 = Color32::from_rgb(0x2b, 0x2b, 0x30);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x80, 0x80, 0x88);

pub fn apply(ctx: &egui::Context) {
    let mut v = egui::Visuals::light();
    v.panel_fill = PANEL;
    v.window_fill = PANEL;
    v.extreme_bg_color = Color32::WHITE;
    v.faint_bg_color = PANEL_DARK;
    v.override_text_color = Some(TEXT);
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;
    let r = CornerRadius::same(4);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = r;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.bg_fill = Color32::from_rgb(0xe4, 0xe4, 0xe8);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0xea, 0xea, 0xee);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0xd8, 0xd8, 0xde);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xdc, 0xdc, 0xe2);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT.gamma_multiply(0.6));
    v.widgets.active.bg_fill = ACCENT;
    v.widgets.active.weak_bg_fill = ACCENT.gamma_multiply(0.8);
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 5.0);
        s.spacing.button_padding = egui::vec2(8.0, 3.0);
        s.spacing.interact_size.y = 22.0;
    });
}
