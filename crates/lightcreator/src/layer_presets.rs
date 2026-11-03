//! The way of burning of a layer (the mode, speed, power, passes, fill, picture, kerf, tabs…) saved under a name and put on any layer
//! later. The list, the window and the file are in `base_ui::layer_presets_ui`; this says what a preset holds. The file is in the
//! shared data folder.
use lc_core::Layer;

/// The file in the shared data folder.
pub const FILE: &str = "lc_layer_presets.json";

/// A preset whose numbers are out of range is left out of the list.
pub fn valid(l: &Layer) -> bool {
    lc_core::profiles::layer_ok(l)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base_ui::layer_presets_ui::{parse, Named};

    #[test]
    fn a_preset_with_a_power_out_of_range_is_left_out_of_a_file() {
        let good = Layer::new(0);
        let mut bad = Layer::new(0);
        bad.power = 500.0;
        let text = serde_json::to_string(&vec![Named { name: "good".to_string(), layer: good }, Named { name: "bad".to_string(), layer: bad }]).unwrap();
        let back = parse::<Layer>(&text, &valid);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].name, "good");
    }
}
