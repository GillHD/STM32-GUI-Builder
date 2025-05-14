pub const DEFAULT_BUILD_SETTINGS: &str = r#"# build_settings.yaml
version: "1.0"

# build_settings sample file
build_settings:
  # range sample
  - id: device_type       # Any unique identifier for the setting
    label: "Device Type"  # User-friendly label for the setting naming on the UI
    value: "type"         # The value that will be used in the naming of the output file
    define: DEVICE_TYPE   # define for the setting in С/C++ code
    description: "Device type number (4-32). Each number represents a specific hardware variant." # Description of the setting
    field_type: range     # Type of field in the UI (e.g., range, select, checkbox_group)
    format: number        # Format of the value (e.g., number, string)
    validation:           # Validation rules for the setting 
      min: 4              # Minimum value for the range
      max: 32             # Maximum value for the range

  # select sample
  - id: device_mode       # Unique identifier for the setting
    label: "Device Mode"  # User-friendly label for the setting naming on the UI
    value: "mode"         # The value that will be used in the naming of the output file
    description: "Operating mode that determines device behavior and available features"          # Description of the setting
    field_type: select    # Type of field in the UI (e.g., range, select, checkbox_group)
    format: string        # Format of the value (e.g., number, string)
    options:              # Options for the select field
      - label: "GPIO_EN"  # User-friendly label for the option
        value: "GPIO"     # Value that will be used in the naming of the output file
        define: "DEVICE_MODE_GPIO"    # define for the option in С/C++ code
        description: "Any text"       # Description of the option
      
      - label: "adc_ext"  # User-friendly label for the option
        value: "ADC_EXT"  # Value that will be used in the naming of the output file
        define: "DEVICE_MODE_ADC_EXT" # define for the option in С/C++ code
        description: "Any text"       # Description of the option

  # checkbox_group sample
  - id: languages       # Unique identifier for the setting
    label: "Languages"  # User-friendly label for the setting naming on the UI
    value: "lang"       # The value that will be used in the naming of the output file
    description: "Supported interface languages. At least one language must be selected." # Description of the setting
    field_type: checkbox_group  # Type of field in the UI (e.g., range, select, checkbox_group)
    format: string[]    # Format of the value (e.g., number, string)
    min_selected: 1     # Minimum number of options that must be selected 
    options:            # Options for the checkbox_group field
      - label: "English"  # User-friendly label for the option
        value: "en"       # Value that will be used in the naming of the output file
        define: "LANG_EN" # define for the option in С/C++ code
        description: "English language support"                         # Description of the option   
      
      - label: "Armenian"  # User-friendly label for the option
        value: "ar"       # Value that will be used in the naming of the output file
        define: "LANG_AR" # define for the option in С/C++ code
        description: "Armenian language support"                         # Description of the option
      
      - label: "Kazakh"   # User-friendly label for the option
        value: "kz"       # Value that will be used in the naming of the output file
        define: "LANG_KZ" # define for the option in С/C++ code
        description: "Kazakh language support"                          # Description of the option
"#;         