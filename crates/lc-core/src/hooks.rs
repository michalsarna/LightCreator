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

/// How deep `@include` may go.
pub const MAX_INCLUDE_DEPTH: usize = 8;
/// The longest a line may be once it is made, characters (the controller's buffer is 128 bytes).
pub const MAX_LINE: usize = 100;
/// The most lines a macro may come to with its includes.
pub const MAX_MACRO_LINES: usize = 500;

#[derive(Clone, Debug, PartialEq)]
pub enum MacroError {
    /// A macro with this name does not exist (an `@include` of it, or a run of it).
    Unknown(String),
    /// A macro includes itself, directly or through others.
    Cycle(String),
    TooDeep,
    TooLong,
    /// A line that is too long, has a control character or a bad `@include`.
    BadLine(String),
    BadVariable(String),
    Empty,
}

impl std::fmt::Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacroError::Unknown(n) => write!(f, "no macro called \"{n}\""),
            MacroError::Cycle(n) => write!(f, "\"{n}\" includes itself"),
            MacroError::TooDeep => write!(f, "includes go deeper than {MAX_INCLUDE_DEPTH} levels"),
            MacroError::TooLong => write!(f, "more than {MAX_MACRO_LINES} lines"),
            MacroError::BadLine(l) => write!(f, "bad line \"{l}\""),
            MacroError::BadVariable(v) => write!(f, "{v}"),
            MacroError::Empty => write!(f, "nothing to run"),
        }
    }
}

/// A line of a macro without its comments (`;` to the end, and `( ... )` anywhere), trimmed.
pub fn strip_comments(line: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in line.chars() {
        match c {
            ';' if depth == 0 => break,
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

fn find<'a>(dev: &'a Device, name: &str) -> Option<&'a Macro> {
    dev.macros.iter().find(|m| m.name.trim().eq_ignore_ascii_case(name.trim()))
}

fn expand_into(dev: &Device, m: &Macro, stack: &mut Vec<String>, out: &mut Vec<String>) -> Result<(), MacroError> {
    let key = m.name.trim().to_lowercase();
    if stack.contains(&key) {
        return Err(MacroError::Cycle(m.name.trim().to_string()));
    }
    if stack.len() >= MAX_INCLUDE_DEPTH {
        return Err(MacroError::TooDeep);
    }
    stack.push(key);
    if !snippet_ok(&m.code) {
        return Err(MacroError::BadLine(m.name.clone()));
    }
    for raw in m.code.lines() {
        // `@include(name)` keeps its brackets; only the `;` comment is cut before it is read.
        let head = raw.split(';').next().unwrap_or("").trim();
        if let Some(rest) = head.strip_prefix("@include") {
            let name = rest.trim().strip_prefix('(').and_then(|r| r.strip_suffix(')')).ok_or_else(|| MacroError::BadLine(head.to_string()))?;
            let inner = find(dev, name).ok_or_else(|| MacroError::Unknown(name.trim().to_string()))?;
            expand_into(dev, inner, stack, out)?;
            continue;
        }
        let line = strip_comments(raw);
        if line.is_empty() {
            continue;
        }
        check_variables(&line, MACRO_VARS).map_err(MacroError::BadVariable)?;
        let line = substitute(&line, &|v| device_value(dev, v, 0));
        if line.chars().count() > MAX_LINE || line.chars().any(|c| c.is_control()) {
            return Err(MacroError::BadLine(line.chars().take(40).collect()));
        }
        out.push(line);
        if out.len() > MAX_MACRO_LINES {
            return Err(MacroError::TooLong);
        }
    }
    stack.pop();
    Ok(())
}

/// The lines to send for the macro called `name`: comments and blank lines gone, `@include`s and variables filled in.
pub fn expand(dev: &Device, name: &str) -> Result<Vec<String>, MacroError> {
    let m = find(dev, name).ok_or_else(|| MacroError::Unknown(name.to_string()))?;
    let mut out = vec![];
    expand_into(dev, m, &mut vec![], &mut out)?;
    if out.is_empty() {
        return Err(MacroError::Empty);
    }
    Ok(out)
}

/// The first thing wrong with the macros of a device: a name that is empty or used twice, or a macro that cannot be
/// expanded. A macro with nothing in it is not an error.
pub fn validate_macros(dev: &Device) -> Option<(String, String)> {
    for (i, m) in dev.macros.iter().enumerate() {
        let name = m.name.trim();
        if name.is_empty() || name.chars().count() > 40 || name.chars().any(|c| c.is_control()) || name.contains(['(', ')']) {
            return Some((m.name.clone(), "the name is empty, too long or has a bracket".into()));
        }
        if dev.macros[..i].iter().any(|o| o.name.trim().eq_ignore_ascii_case(name)) {
            return Some((m.name.clone(), "the name is used twice".into()));
        }
        match expand(dev, name) {
            Ok(_) | Err(MacroError::Empty) => {}
            Err(e) => return Some((m.name.clone(), e.to_string())),
        }
    }
    None
}
