//! Preferences: the nearest value wins; `options` merges key by key, recursively, and `models`,
//! `workspace`, `context` and `config` merge key by key.
use super::{Ctx, Effective};
use crate::model::Preferences;
use serde_json::{Map, Value};

/// Members whose keys merge one level deep.
const KEYED: &[&str] = &["models", "workspace", "context", "config"];

/// Apply `new` over `acc`, recording each value's source.
pub fn merge(acc: &mut Preferences, new: &Preferences, ctx: &mut Ctx) {
    let mut a = to_map(acc);
    for (key, value) in to_map(new) {
        let field = format!("preferences.{key}");
        if key == "options" || KEYED.contains(&key.as_str()) {
            let slot = a.entry(key.clone()).or_insert_with(|| Value::Object(Map::new()));
            deep(slot, value, &field, key == "options", ctx);
        } else {
            ctx.source(&field);
            a.insert(key, value);
        }
    }
    *acc = serde_json::from_value(Value::Object(a)).expect("merged preferences keep their shape");
}

fn deep(slot: &mut Value, value: Value, field: &str, recurse: bool, ctx: &mut Ctx) {
    match (slot, value) {
        (Value::Object(into), Value::Object(from)) => {
            for (k, v) in from {
                let inner = format!("{field}.{k}");
                let entry = into.entry(k).or_insert(Value::Null);
                if recurse && entry.is_object() && v.is_object() {
                    deep(entry, v, &inner, true, ctx);
                } else {
                    ctx.source(&inner);
                    *entry = v;
                }
            }
        }
        (slot, value) => {
            ctx.source(field);
            *slot = value;
        }
    }
}

/// The agent's runtime's entry in `models` is its model, unless a block after the one that
/// set that entry also sets `model`. `applied` are the preferences that reached it, in order.
pub fn runtime_model(e: &mut Effective, applied: &[&Preferences]) {
    let Some(runtime) = e.preferences.runtime.clone() else { return };
    let last = |f: &dyn Fn(&Preferences) -> bool| applied.iter().rposition(|p| f(p));
    let Some(entry) = last(&|p| p.models.contains_key(&runtime)) else { return };
    if last(&|p| p.model.is_some()).is_some_and(|m| m > entry) {
        return;
    }
    e.preferences.model = e.preferences.models.get(&runtime).cloned();
    if let Some(src) = e.sources.get(&format!("preferences.models.{runtime}")).cloned() {
        e.sources.insert("preferences.model".into(), src);
    }
}

fn to_map(p: &Preferences) -> Map<String, Value> {
    match serde_json::to_value(p) {
        Ok(Value::Object(m)) => m,
        _ => Map::new(),
    }
}
