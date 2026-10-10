//! Colour schemes: flat chrome modelled on VectorCraft with a red-orange accent.
//! Four schemes (Dark, Light Dark, Medium Light, Light) selectable at runtime.
use eframe::egui::{self, Color32, CornerRadius, Stroke};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Scheme {
    Dark,
    LightDark,
    MediumLight,
    Light,
}

impl Scheme {
    pub const ALL: [Scheme; 4] = [Scheme::Dark, Scheme::LightDark, Scheme::MediumLight, Scheme::Light];

    /// English label (translated through `i18n::tr`).
    pub fn label(self) -> &'static str {
        match self {
            Scheme::Dark => "Dark",
            Scheme::LightDark => "Light Dark",
            Scheme::MediumLight => "Medium Light",
            Scheme::Light => "Light",
        }
    }
    pub fn id(self) -> &'static str {
        match self {
            Scheme::Dark => "dark",
            Scheme::LightDark => "light-dark",
            Scheme::MediumLight => "medium-light",
            Scheme::Light => "light",
        }
    }
    pub fn from_id(s: &str) -> Option<Scheme> {
        Scheme::ALL.into_iter().find(|x| x.id() == s)
    }
    pub fn is_dark(self) -> bool {
        matches!(self, Scheme::Dark | Scheme::LightDark)
    }
    fn index(self) -> u8 {
        Scheme::ALL.iter().position(|x| *x == self).unwrap_or(3) as u8
    }
}

struct Palette {
    panel: Color32,
    panel_dark: Color32,
    border: Color32,
    workspace: Color32,
    text: Color32,
    text_dim: Color32,
    widget: Color32,
    widget_weak: Color32,
    widget_hover: Color32,
    widget_hover_weak: Color32,
    input: Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

const DARK: Palette = Palette {
    panel: rgb(0x26, 0x27, 0x2b),
    panel_dark: rgb(0x1e, 0x1f, 0x22),
    border: rgb(0x3a, 0x3b, 0x42),
    workspace: rgb(0x16, 0x17, 0x19),
    text: rgb(0xe6, 0xe6, 0xea),
    text_dim: rgb(0x93, 0x93, 0x9e),
    widget: rgb(0x34, 0x35, 0x3b),
    widget_weak: rgb(0x2f, 0x30, 0x35),
    widget_hover: rgb(0x44, 0x45, 0x4d),
    widget_hover_weak: rgb(0x3d, 0x3e, 0x45),
    input: rgb(0x1a, 0x1b, 0x1e),
};
const LIGHT_DARK: Palette = Palette {
    panel: rgb(0x3c, 0x3d, 0x43),
    panel_dark: rgb(0x33, 0x34, 0x39),
    border: rgb(0x52, 0x53, 0x5b),
    workspace: rgb(0x2b, 0x2c, 0x30),
    text: rgb(0xec, 0xec, 0xf0),
    text_dim: rgb(0xa6, 0xa6, 0xb0),
    widget: rgb(0x4c, 0x4d, 0x55),
    widget_weak: rgb(0x46, 0x47, 0x4e),
    widget_hover: rgb(0x5a, 0x5b, 0x64),
    widget_hover_weak: rgb(0x54, 0x55, 0x5e),
    input: rgb(0x2f, 0x30, 0x34),
};
const MEDIUM_LIGHT: Palette = Palette {
    panel: rgb(0xdd, 0xdd, 0xe1),
    panel_dark: rgb(0xcf, 0xcf, 0xd4),
    border: rgb(0xb4, 0xb4, 0xbb),
    workspace: rgb(0xb8, 0xb8, 0xbf),
    text: rgb(0x22, 0x22, 0x27),
    text_dim: rgb(0x5c, 0x5c, 0x66),
    widget: rgb(0xcb, 0xcb, 0xd1),
    widget_weak: rgb(0xd2, 0xd2, 0xd8),
    widget_hover: rgb(0xbd, 0xbd, 0xc5),
    widget_hover_weak: rgb(0xc4, 0xc4, 0xcc),
    input: rgb(0xf0, 0xf0, 0xf2),
};
const LIGHT: Palette = Palette {
    panel: rgb(0xf4, 0xf4, 0xf5),
    panel_dark: rgb(0xe9, 0xe9, 0xeb),
    border: rgb(0xd2, 0xd2, 0xd6),
    workspace: rgb(0xdc, 0xdc, 0xe0),
    text: rgb(0x2b, 0x2b, 0x30),
    text_dim: rgb(0x80, 0x80, 0x88),
    widget: rgb(0xe4, 0xe4, 0xe8),
    widget_weak: rgb(0xea, 0xea, 0xee),
    widget_hover: rgb(0xd8, 0xd8, 0xde),
    widget_hover_weak: rgb(0xdc, 0xdc, 0xe2),
    input: Color32::WHITE,
};

static CURRENT: AtomicU8 = AtomicU8::new(3);

fn pal() -> &'static Palette {
    match CURRENT.load(Ordering::Relaxed) {
        0 => &DARK,
        1 => &LIGHT_DARK,
        2 => &MEDIUM_LIGHT,
        _ => &LIGHT,
    }
}

pub fn accent() -> Color32 {
    rgb(0xe8, 0x57, 0x3f)
}
pub fn panel() -> Color32 {
    pal().panel
}
pub fn panel_dark() -> Color32 {
    pal().panel_dark
}
pub fn border() -> Color32 {
    pal().border
}
pub fn workspace() -> Color32 {
    pal().workspace
}
pub fn text() -> Color32 {
    pal().text
}
/// Background of the console output: near black in the dark schemes, a warm paper tone in the light ones.
pub fn console_bg() -> Color32 {
    if CURRENT.load(Ordering::Relaxed) <= 1 {
        rgb(0x0f, 0x13, 0x1b)
    } else {
        rgb(0xfb, 0xf7, 0xe9)
    }
}
pub fn text_dim() -> Color32 {
    pal().text_dim
}

pub fn apply(ctx: &egui::Context, scheme: Scheme) {
    CURRENT.store(scheme.index(), Ordering::Relaxed);
    let p = pal();
    let accent = accent();
    let mut v = if scheme.is_dark() { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = p.panel;
    v.window_fill = p.panel;
    v.extreme_bg_color = p.input;
    v.faint_bg_color = p.panel_dark;
    v.override_text_color = Some(p.text);
    v.window_stroke = Stroke::new(1.0, p.border);
    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    v.selection.bg_fill = accent.gamma_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0, accent);
    v.hyperlink_color = accent;
    let r = CornerRadius::same(4);
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = r;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    v.widgets.inactive.bg_fill = p.widget;
    v.widgets.inactive.weak_bg_fill = p.widget_weak;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border);
    v.widgets.hovered.bg_fill = p.widget_hover;
    v.widgets.hovered.weak_bg_fill = p.widget_hover_weak;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, accent.gamma_multiply(0.6));
    v.widgets.active.bg_fill = accent;
    v.widgets.active.weak_bg_fill = accent.gamma_multiply(0.8);
    v.widgets.active.bg_stroke = Stroke::new(1.0, accent);
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 5.0);
        s.spacing.button_padding = egui::vec2(8.0, 3.0);
        s.spacing.interact_size.y = 22.0;
        // Wide enough that menu entries are not cut short with an ellipsis.
        s.spacing.menu_width = 280.0;
    });
}
