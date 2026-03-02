use super::overrides::{
    resolve_prj_config_home, resolve_project_root, wendao_config_file_override,
};
use serde_yaml::{Mapping, Value};
use std::path::Path;

fn read_yaml_file(path: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str::<Value>(&content).ok()
}

fn deep_merge(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Mapping(base_map), Value::Mapping(overlay_map)) => {
            for (key, value) in overlay_map {
                if let Some(existing) = base_map.get_mut(&key) {
                    deep_merge(existing, value);
                } else {
                    base_map.insert(key, value);
                }
            }
        }
        (base_value, overlay_value) => {
            *base_value = overlay_value;
        }
    }
}

pub(in crate::link_graph::runtime_config) fn merged_wendao_settings() -> Value {
    let root = resolve_project_root();
    let system_path = root.join("packages/conf/wendao.yaml");
    let user_path = wendao_config_file_override().unwrap_or_else(|| {
        resolve_prj_config_home(&root).join("xiuxian-artisan-workshop/wendao.yaml")
    });

    let mut merged = Value::Mapping(Mapping::new());
    if let Some(system) = read_yaml_file(&system_path) {
        deep_merge(&mut merged, system);
    }
    if let Some(user) = read_yaml_file(&user_path) {
        deep_merge(&mut merged, user);
    }
    merged
}
