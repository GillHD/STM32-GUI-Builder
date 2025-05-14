# STM32-GUI-Builder — Usage Guide

This guide explains how to use STM32-GUI-Builder for automated project builds and what you need to prepare in your STM32CubeIDE project and environment.

---

## 1. Preparing Your STM32CubeIDE Project

Before using STM32-GUI-Builder, make sure your STM32CubeIDE project is properly set up:

- **All source files, include paths, and build settings must be configured and tested in STM32CubeIDE.**
- **Build Configurations**:  
  In STM32CubeIDE, set up all required build configurations (e.g., Debug, Release, or custom configs) via `Project > Build Configurations > Manage...`.  
  The configuration name you select in the GUI must exist in your `.cproject` file.
- **Workspace**:  
  The workspace directory you specify in STM32-GUI-Builder must match the one used in STM32CubeIDE.

---

## 2. Editing Your Header Files

To allow STM32-GUI-Builder to manage your build defines, you must wrap your editable defines in your header file (e.g., `main.h`) as follows:

```c
#ifndef BUILD_CONFIG_H_ // Block #1

// defines edited by STM32-GUI-Builder

#endif
```

- All defines that will be enabled/disabled by the builder must be placed inside this block.
- The application will overwrite this file for each build combination.

---

## 3. Closing STM32CubeIDE Before Using the Builder

> **Important:**  
> Before running STM32-GUI-Builder, **close STM32CubeIDE**.  
> If STM32CubeIDE is running and the workspace is open, the builder will fail to start the build process due to workspace locking.

---

## 4. Using STM32-GUI-Builder

### Step-by-step Instructions

1. **Launch the Application**
   - Start STM32-GUI-Builder using the instructions from the main README.

2. **Configure Project Settings**
   - In the GUI, set:
     - **Workspace Path**: Path to your STM32CubeIDE workspace.
     - **STM32CubeIDE Executable Path**: Full path to `stm32cubeide.exe` (or the corresponding binary on your OS).
     - **Project Path**: Path to your STM32CubeIDE project folder.
     - **Build Directory**: Output directory for build artifacts.

3. **Define Build Parameters**
   - Use the GUI to select device types, modes, languages, and other options.
   - The available parameters are loaded from `build_settings.yaml` (or the backend default schema).

4. **Start the Build**
   - Click the build button in the GUI.
   - The application will:
     - Generate a `build_config.h` file for each combination of parameters.
     - Launch STM32CubeIDE in headless mode for each configuration.
     - Collect logs and build results in real time.

5. **Review Results**
   - Monitor build progress and logs in the GUI.
   - Output binaries and logs will be placed in the specified build directory.

6. **Customizing Build Parameters**
   - To add or change build parameters, edit `build_settings.yaml` in the project root.
   - See the example schema in the README.

---

## 5. Notes and Recommendations

- **Do not open STM32CubeIDE while using the builder.**  
  The builder requires exclusive access to the workspace.
- **All project settings, source files, and build scripts must be pre-configured in STM32CubeIDE.**  
  The builder does not modify project structure or settings except for the generated header file.
- **If you encounter build errors related to workspace locking, close STM32CubeIDE and try again.**
- **You can use the builder to automate builds for multiple configurations and parameter sets without manual intervention.**

---

## 6. Troubleshooting

- **Build fails with workspace lock error:**  
  Close STM32CubeIDE and retry.
- **Defines are not updated:**  
  Ensure all editable defines are inside the `#ifndef BUILD_CONFIG_H_ ... #endif` block in your header.
- **Build configuration not found:**  
  Make sure the configuration exists in STM32CubeIDE and is listed in `.cproject`.

---

For more details, see the [README.md](./README.md).
