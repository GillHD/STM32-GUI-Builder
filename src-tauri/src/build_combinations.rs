use serde_json::Value;
use crate::config::BuildSettingsConfig;

pub fn generate_build_combinations(
    settings_config: &BuildSettingsConfig,
    settings: &serde_json::Map<String, Value>
) -> Vec<Vec<(String, String)>> {
    let settings_values = settings_config.build_settings.iter().map(|setting| {
        let values = match setting.field_type.as_str() {
            "range" => {
                // Get range string and parse it into numbers
                if let Some(value) = settings.get(&setting.id) {
                    if let Some(str_val) = value.as_str() {
                        // Use parse_range_string to get numbers
                        if let Some(validation) = &setting.validation {
                            match crate::config::parse_range_string(str_val, validation.min, validation.max) {
                                Ok(numbers) => numbers.into_iter().map(|n| n.to_string()).collect(),
                                Err(_) => Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            },
            "select" => settings.get(&setting.id)
                .and_then(|v| v.as_str().map(|s| vec![s.to_string()]))
                .unwrap_or_default(),
            "checkbox_group" => settings.get(&setting.id)
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
                .unwrap_or_default(),
            _ => vec![],
        };
        (setting, values)
    }).collect::<Vec<_>>();

    // Create combinations for build
    let mut build_combinations = vec![vec![]];
    for (setting, values) in &settings_values {
        let mut new_combinations = vec![];
        // If parameter is optional and array is empty — use [None] for Cartesian product
        let is_optional = setting.min_selected.unwrap_or(0) == 0;
        let values_for_comb = if values.is_empty() && is_optional {
            vec![None]
        } else {
            values.iter().map(|v| Some(v.clone())).collect()
        };
        for value_opt in values_for_comb {
            for combo in &build_combinations {
                let mut new_combo = combo.clone();
                if let Some(ref value) = value_opt {
                    new_combo.push((setting.id.clone(), value.clone()));
                }
                new_combinations.push(new_combo);
            }
        }
        build_combinations = new_combinations;
    }

    build_combinations
}