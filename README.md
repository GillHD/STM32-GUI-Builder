# STM32-GUI-Builder 🚀

**STM32-GUI-Builder** is a cross-platform desktop application designed to streamline the configuration and building of STM32 microcontroller projects using STM32CubeIDE. 🌟 With an intuitive graphical user interface (GUI), it simplifies project setup, build configuration, and execution, making it an essential tool for embedded systems developers working with diverse STM32 hardware and software configurations.

---

## ✨ Key Features

- 🛠 **Project Configuration**: Easily set project paths, workspace directories, and STM32CubeIDE executable locations.
- ⚙️ **Dynamic Build Settings**: Define build parameters (e.g., device types, operating modes, languages, and optional features) via a YAML schema, supporting range inputs, dropdowns, and checkbox groups.
- 🏗 **Automated Build Process**: Run headless builds with STM32CubeIDE, automatically generating `build_config.h` files for each configuration combination.
- 📊 **Real-Time Build Monitoring**: Track build status, logs, and stdout/stderr output through a responsive GUI.
- 🌍 **Cross-Platform**: Built with Tauri for lightweight, native performance on Windows, macOS, and Linux.
- 🔧 **Extensible Schema**: Customize build settings in `build_settings.yaml` to meet specific project needs.

---

## 📸 Screenshots

![image](https://github.com/user-attachments/assets/e68d7f42-d053-474c-b005-6d46ec71dcfe)


---

## 🛠 Tech Stack

- **Frontend**: Vue 3 (TypeScript, Composition API), Tailwind CSS for modern, responsive UI. 🎨
- **Backend**: Rust with Tauri for secure, high-performance native integration. ⚡
- **Build Integration**: Interfaces with STM32CubeIDE for headless builds, leveraging YAML schemas for dynamic configuration.
- **Dependencies**:
  - Frontend: `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `vue`
  - Backend: `serde`, `tokio`, `tauri`

---

## 📋 Prerequisites

Before setting up the project, ensure the following are installed:

- **Node.js**: v16 or higher
- **Rust**: v1.65 or higher
- **STM32CubeIDE**: Installed and accessible (required for build execution)
- **npm or yarn**: For managing frontend dependencies

---

## ⚙️ Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/STM32-GUI-Builder.git
   cd STM32-GUI-Builder
   ```

2. Install frontend dependencies:
   ```bash
   npm install
   ```

3. Ensure Rust and Tauri CLI are installed:
   ```bash
   cargo install tauri-cli
   ```

---

## 🚀 Running the Application

- Start the development server:
  ```bash
  npm run tauri dev
  ```
  This launches the app in development mode with hot-reloading.

- Build a release version:
  ```bash
  npm run tauri build
  ```
  This generates native binaries in `src-tauri/target/release`.

---

## 📖 Usage

1. **Configure Project Settings** ⚙️: In the GUI, set the workspace path, STM32CubeIDE executable path, project path, and build directory.
2. **Define Build Parameters** 📝: Use the build settings interface to specify device types (range), modes (dropdown), languages, and additional options (checkboxes). Settings are loaded from `build_settings.yaml` or default to the backend schema.
3. **Execute Builds** 🏗: Trigger builds via the GUI. The app generates `build_config.h` files and runs STM32CubeIDE in headless mode. Monitor progress with real-time logs and status updates.
4. **Customize Settings** 🔧: Edit `build_settings.yaml` in the project root to define custom build parameters (e.g., new device types or options).

---

## 🗂 Build Settings Schema

Build settings are defined in `build_settings.yaml` (or the default schema in `defaults.rs`). Below is an example schema:

```yaml
version: "1.0"

build_settings:
  # Range input example
  - id: device_type
    label: "Device Type"
    value: "type"
    define: DEVICE_TYPE
    description: "Device type number (4-32). Each number represents a specific hardware variant."
    field_type: range
    format: number
    validation:
      min: 4
      max: 32

  # Dropdown (select) example
  - id: device_mode
    label: "Device Mode"
    value: "mode"
    description: "Operating mode that determines device behavior and available features."
    field_type: select
    format: string
    options:
      - label: "GPIO_EN"
        value: "GPIO"
        define: "DEVICE_MODE_GPIO"
        description: "GPIO-enabled mode."
      - label: "adc_ext"
        value: "ADC_EXT"
        define: "DEVICE_MODE_ADC_EXT"
        description: "ADC external mode."

  # Checkbox group example
  - id: languages
    label: "Languages"
    value: "lang"
    description: "Supported interface languages. At least one language must be selected."
    field_type: checkbox_group
    format: string[]
    min_selected: 1
    options:
      - label: "English"
        value: "en"
        define: "LANG_EN"
        description: "English language support."
      - label: "Armenian"
        value: "ar"
        define: "LANG_AR"
        description: "Armenian language support."
      - label: "Kazakh"
        value: "kz"
        define: "LANG_KZ"
        description: "Kazakh language support."
```

---

## 🤝 Contributing

Contributions are welcome! To contribute:

1. Fork the repository.
2. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature
   ```
3. Commit your changes:
   ```bash
   git commit -m "Add your feature"
   ```
4. Push to the branch:
   ```bash
   git push origin feature/your-feature
   ```
5. Open a pull request.

Please adhere to the coding style, ensure tests pass, and include relevant documentation.

---

## 🛠 Troubleshooting

- **Settings not displaying** 🚫: Check console logs in browser DevTools and Tauri terminal. Verify `build_settings.yaml` exists and matches the schema.
- **Build failures** ⚠️: Ensure STM32CubeIDE is correctly installed and accessible. Review logs in the GUI or `build_log.txt`.
- **Dependency issues** 🧩: Run `npm install` and `cargo build` to resolve missing dependencies.

For additional help, open an issue with relevant logs and details.

---

## 📜 License

This project is licensed under the [MIT License](LICENSE). 📄

---

## 🙌 Acknowledgments

- Built with [Tauri](https://tauri.app/) and [Vue.js](https://vuejs.org/). 💻
- Inspired by the needs of STM32 embedded developers. 🌟
