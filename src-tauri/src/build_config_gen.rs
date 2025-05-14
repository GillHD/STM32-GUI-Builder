use crate::config::BuildSettingsConfig;

pub fn generate_build_config_h(
    settings_config: &BuildSettingsConfig,
    combination: &[(String, String)]
) -> Result<String, String> {
    let mut build_config_content = String::new();
    build_config_content.push_str("#ifndef BUILD_CONFIG_H_\n#define BUILD_CONFIG_H_\n\n");

    for setting in &settings_config.build_settings {
        let id = &setting.id;
        let value_opt = combination.iter().find(|(s_id, _)| s_id == id).map(|(_, v)| v.clone());

        match setting.field_type.as_str() {
            "range" => {
                if let Some(value) = value_opt {
                    if let Some(validation) = &setting.validation {
                        if let Ok(numbers) = crate::config::parse_range_string(&value, validation.min, validation.max) {
                            if let Some(last_num) = numbers.last() {
                                if let Some(define) = &setting.define {
                                    build_config_content.push_str(&format!(
                                        "#ifndef {}\n#define {} {}\n#endif\n",
                                        define, define, last_num
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            "select" | "checkbox_group" => {
                if let Some(options) = &setting.options {
                    for opt in options {
                        let is_selected = if let Some(v) = &value_opt {
                            v == &opt.value
                        } else {
                            false
                        };

                        if let Some(define) = &opt.define {
                            if is_selected {
                                build_config_content.push_str(&format!("#define {}\n", define));
                            } else {
                                build_config_content.push_str(&format!("#undef {}\n", define));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    build_config_content.push_str("#undef DEBUG_SET\n");
    build_config_content.push_str("\n#endif // BUILD_CONFIG_H_\n");

    Ok(build_config_content)
}