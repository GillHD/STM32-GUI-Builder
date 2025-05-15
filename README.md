# STM32-GUI-Builder 🚀

**STM32-GUI-Builder** is a cross-platform desktop application that streamlines the configuration and building of STM32 microcontroller projects using STM32CubeIDE.  
With an intuitive graphical interface, it simplifies project setup, build configuration, and execution — making it an essential tool for embedded systems developers.

---

## ✨ Features

- 🛠 **Project Configuration**: Easily set project paths, workspace directories, and STM32CubeIDE executable locations.
- ⚙️ **Dynamic Build Settings**: Define build parameters (device types, modes, languages, and more) via a YAML schema, supporting ranges, dropdowns, and checkbox groups.
- 🏗 **Automated Build Process**: Run headless builds with STM32CubeIDE, automatically generating `build_config.h` for each configuration.
- 📊 **Real-Time Build Monitoring**: Track build status and logs through a responsive GUI.
- 🌍 **Cross-Platform**: Native performance on Windows, macOS, and Linux (powered by Tauri).
- 🔧 **Extensible Schema**: Customize build settings in `build_settings.yaml` to fit your project needs.

---

## 📸 Screenshots

![STM32-GUI-Builder Screenshot](https://github.com/user-attachments/assets/e68d7f42-d053-474c-b005-6d46ec71dcfe)

---

## 🛠 Tech Stack

- **Frontend**: Vue 3 (TypeScript, Composition API), Tailwind CSS
- **Backend**: Rust + Tauri
- **Build Integration**: STM32CubeIDE (headless), YAML schemas
- **Dependencies**:
  - Frontend: `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `vue`
  - Backend: `serde`, `tokio`, `tauri`, `anyhow`, `quick-xml`, and more

---

## 📋 Prerequisites

- **[Node.js](https://nodejs.org/en/)**: v22 or higher
- **[Rust](https://www.rust-lang.org/tools/install)**: v1.65 or higher
- **[C++ Build Tools](https://visualstudio.microsoft.com/ru/visual-cpp-build-tools/)**: or Visual Studio with C++ Build Tools for build Rust on Windows 
- **Tauri CLI**:  
  ```bash
  cargo install tauri-cli
  ```
- **[STM32CubeIDE](https://www.st.com/en/development-tools/stm32cubeide.html)**: Installed and accessible in your system
- **npm** or **yarn**: For frontend dependencies usally installed with node.js

---

## ⚡ Quick Start

1. **Clone the repository:**
   ```bash
   git clone https://github.com/GillHD/STM32-GUI-Builder.git
   cd STM32-GUI-Builder
   ```

2. **Install frontend dependencies:**
   ```bash
   npm install
   # or
   yarn install
   ```

3. **Run the application in development mode:**
   ```bash
   npm run tauri dev
   # or
   yarn tauri dev
   ```
   > The app will open with hot-reloading enabled.

4. **Build a release version:**
   ```bash
   npm run tauri build
   # or
   yarn tauri build
   ```
   > Native binaries will be generated in `src-tauri/target/release`.

---

## 📖 Usage

1. **Configure Project Settings**:  
   In the GUI, set the workspace path, STM32CubeIDE executable path, project path, and build directory.
2. **Define Build Parameters**:  
   Use the build settings interface to specify device types (range), modes (dropdown), languages, and additional options (checkboxes). Settings are loaded from `build_settings.yaml` or the backend schema.
3. **Execute Builds**:  
   Trigger builds via the GUI. The app generates `build_config.h` files and runs STM32CubeIDE in headless mode. Monitor progress with real-time logs and status updates.
4. **Customize Settings**:  
   Edit `build_settings.yaml` in the project root to define custom build parameters (e.g., new device types or options).

For more details, see the [USAGE.md](./USAGE.md) and [HOW_IT_WORKS.md](./HOW_IT_WORKS.md).

---

## 🗂 Example Build Settings Schema

```yaml
version: "1.0"

build_settings:
  - id: device_type
    label: "Device Type"
    value: "type"
    define: DEVICE_TYPE
    description: "Device type number (4-32)."
    field_type: range
    format: number
    validation:
      min: 4
      max: 32

  - id: device_mode
    label: "Device Mode"
    value: "mode"
    description: "Operating mode."
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

  - id: languages
    label: "Languages"
    value: "lang"
    description: "Supported interface languages."
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

Contributions are welcome!  
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

Please follow the coding style, ensure tests pass, and include relevant documentation.

---

## 🛠 Troubleshooting

- **Settings not displaying**:  
  Check browser DevTools and Tauri terminal logs. Ensure `build_settings.yaml` exists and matches the schema.
- **Build failures**:  
  Ensure STM32CubeIDE is installed and accessible. Review logs in the GUI or `build_log.txt`.
- **Dependency issues**:  
  Run `npm install` and `cargo build` to resolve missing dependencies.

For additional help, open an issue with relevant logs and details.

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).

---

## 🙌 Acknowledgments

- Built with [Tauri](https://tauri.app/) and [Vue.js](https://vuejs.org/)
- Inspired by the needs of STM32 embedded developers
