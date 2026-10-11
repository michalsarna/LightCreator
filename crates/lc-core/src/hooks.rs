//! Own G-code that runs at set points of a job (hooks) or by hand (macros), with `{name}` variables.
use crate::doc::*;

/// Variables of a job: a hook may use all of them.
pub const JOB_VARS: &[&str] = &[
    "job_name", "date", "time", "layer_name", "layer_index", "total_layers", "workpiece_name", "workpiece_index", "total_workpieces", "machine_name",
    "max_speed", "work_width", "work_height", "tool_number", "wavelength", "optical_power", "pwm_freq", "pwm_max_freq", "pulse_width", "pulse_min", "pulse_max",
];

/// Variables a macro may use: the ones that do not depend on a job.
pub const MACRO_VARS: &[&str] = &["machine_name", "max_speed", "work_width", "work_height", "tool_number", "wavelength", "optical_power", "pwm_freq", "pwm_max_freq", "pulse_width", "pulse_min", "pulse_max"];

/// The `{name}` words of a text, or an error for a `{` or `}` that does not form one.
pub fn variables(text: &str) -> Result<Vec<String>, String> {
    let mut out = vec![];
    let mut rest = text;
    while let Some(i) = rest.find(['{', '}']) {
        if rest.as_bytes()[i] == b'}' {
            return Err("a } without a {".into());
        }
        let after = &rest[i + 1..];
        let Some(j) = after.find('}') else { return Err("a { without a }".into()) };
        let name = &after[..j];
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
            return Err(format!("{{{}}} is not a variable", name));
        }
        out.push(name.to_string());
        rest = &after[j + 1..];
    }
    Ok(out)
}

/// What is wrong with the variables of `text`, in plain words: an unknown name, or a brace that does not close.
pub fn check_variables(text: &str, allowed: &[&str]) -> Result<(), String> {
    for v in variables(text)? {
        if !allowed.contains(&v.as_str()) {
            return Err(format!("unknown variable {{{v}}}"));
        }
    }
    Ok(())
}

/// `text` with every `{name}` replaced by what `value` says. Names that `value` does not know are left out empty
/// (a text that passed `check_variables` has none).
pub fn substitute(text: &str, value: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        match after.find('}') {
            Some(j) => {
                out.push_str(&value(&after[..j]).unwrap_or_default());
                rest = &after[j + 1..];
            }
            None => {
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// The values of the variables that belong to the machine and its first head.
pub fn device_value(dev: &Device, name: &str, tool: usize) -> Option<String> {
    let (w, h) = dev.work_size();
    let head = dev.head(tool);
    Some(match name {
        "machine_name" => dev.name.clone(),
        "max_speed" => format!("{}", dev.speeds.max_cut_speed),
        "work_width" => format!("{w}"),
        "work_height" => format!("{h}"),
        "tool_number" => format!("{}", head.tool),
        "wavelength" => format!("{}", head.wavelength()),
        "optical_power" => format!("{}", head.power_w()),
        "pwm_freq" => format!("{}", head.pwm_freq_hz),
        "pwm_max_freq" => format!("{}", head.pwm_max_freq_hz),
        "pulse_width" => format!("{}", head.pulse_width_us),
        "pulse_min" => format!("{}", head.pulse_min_us),
        "pulse_max" => format!("{}", head.pulse_max_us),
        _ => return None,
    })
}
