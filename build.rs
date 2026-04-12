use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let dest = out_dir.join("toggleable_iter_impls.rs");

    fs::write(dest, generate_impls()).expect("write generated toggle impls");
    println!("cargo:rerun-if-changed=build.rs");
}

fn generate_impls() -> String {
    let mut out = String::new();

    for total in 1..=8 {
        for prefix in 1..=total {
            emit_tuple_impl(&mut out, total, prefix);
            out.push('\n');
        }
    }

    out
}

fn emit_tuple_impl(out: &mut String, total: usize, prefix: usize) {
    let item_params = generic_names("I", total);
    let toggle_params = generic_names("T", prefix);
    let item_tuple = tuple_type(&item_params);
    let toggle_tuple = tuple_type(&toggle_params);
    let states_tuple = tuple_type(&vec!["bool".to_string(); prefix]);

    out.push_str("impl<");
    push_generic_params(out, &toggle_params, "crate::ToggleableComponent");
    if !item_params.is_empty() {
        out.push_str(", ");
    }
    push_bounded_item_params(out, &item_params[..prefix], &toggle_params);
    if total > prefix {
        out.push_str(", ");
        push_generic_names(out, &item_params[prefix..]);
    }
    out.push_str("> ToggleItemMatch<");
    out.push_str(&toggle_tuple);
    out.push_str("> for ");
    out.push_str(&item_tuple);
    out.push_str(" {\n");

    out.push_str("    #[inline]\n");
    out.push_str("    fn matches_toggle_states(&self, states: &");
    out.push_str(&states_tuple);
    out.push_str(") -> bool {\n");

    out.push_str("        ");
    for index in 0..prefix {
        if index > 0 {
            out.push_str(" &&\n        ");
        }
        out.push_str("self.");
        out.push_str(&index.to_string());
        out.push_str(".toggle_ref().is_active() == states.");
        out.push_str(&index.to_string());
    }
    out.push_str("\n    }\n");
    out.push_str("}\n");
}

fn generic_names(prefix: &str, count: usize) -> Vec<String> {
    (1..=count)
        .map(|index| format!("{prefix}{index}"))
        .collect()
}

fn tuple_type(parts: &[String]) -> String {
    match parts.len() {
        0 => "()".to_string(),
        1 => format!("({},)", parts[0]),
        _ => format!("({})", parts.join(", ")),
    }
}

fn push_generic_params(out: &mut String, names: &[String], bound: &str) {
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(name);
        if !bound.is_empty() {
            out.push_str(": ");
            out.push_str(bound);
        }
    }
}

fn push_bounded_item_params(out: &mut String, item_names: &[String], toggle_names: &[String]) {
    for (index, name) in item_names.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(name);
        out.push_str(": core::ops::Deref<Target = ");
        out.push_str(&toggle_names[index]);
        out.push('>');
    }
}

fn push_generic_names(out: &mut String, names: &[String]) {
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(name);
    }
}
