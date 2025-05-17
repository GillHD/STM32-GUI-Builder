# 🚀 Contributing to STM32-GUI-Builder

Thank you for your interest in contributing to **STM32-GUI-Builder**! 🎉 We welcome contributions from the community to make this tool even better for embedded systems developers working with STM32 microcontrollers. This guide outlines how to get started, the types of contributions we accept, and the process for submitting changes.

---

## 🌟 Getting Started

Before you begin, please take a moment to familiarize yourself with the project:

- **Read the README**: Check out the [README.md](README.md) to understand the project’s purpose, setup instructions, and usage.
- **Explore Issues**: Visit our [GitHub Issues page](https://github.com/your-username/STM32-GUI-Builder/issues) to find tasks labeled `good first issue` or `help wanted` for beginner-friendly contributions.
- **Join the Discussion**: Engage with the community by commenting on issues or pull requests to share ideas or clarify requirements.

---

## 🛠 How to Contribute

We accept various types of contributions, including code, documentation, bug reports, and feature suggestions. Here’s how you can contribute:

### 1. **Reporting Bugs** 🐛
- Check if the bug is already reported on the [Issues page](https://github.com/your-username/STM32-GUI-Builder/issues).
- Open a new issue with:
  - A clear title and description.
  - Steps to reproduce the bug.
  - Expected and actual behavior.
  - Environment details (e.g., OS, Node.js version, STM32CubeIDE version).
- Use the `bug` label when creating the issue.

### 2. **Suggesting Features** 💡
- Share your ideas for new features or improvements by opening an issue.
- Describe the feature, its use case, and how it benefits the project.
- Use the `enhancement` label for feature requests.

### 3. **Submitting Code Changes** 💻
- Follow the steps below to set up the project, make changes, and submit a pull request.
- Contributions can include bug fixes, new features, UI improvements, or performance optimizations.

### 4. **Improving Documentation** 📚
- Enhance the [README.md](README.md), code comments, or other documentation.
- Fix typos, clarify instructions, or add examples to help users and developers.

---

## ⚙️ Setting Up the Development Environment

To contribute code or documentation, set up the project locally:

1. **Fork and Clone the Repository**:
   ```bash
   git clone https://github.com/your-username/STM32-GUI-Builder.git
   cd STM32-GUI-Builder
   ```

2. **Install Dependencies**:
   - Install Node.js (v16 or higher) and Rust (v1.65 or higher).
   - Run:
     ```bash
     npm install
     cargo install tauri-cli
     ```
   - Ensure STM32CubeIDE is installed and accessible.

3. **Run the Development Server**:
   ```bash
   npm run tauri dev
   ```
   This starts the app with hot-reloading for testing changes.

4. **Verify Setup**:
   - Confirm the GUI loads and builds execute correctly using the default `build_settings.yaml`.
   - Check the console and Tauri logs for errors.

---

## 📝 Coding Guidelines

To ensure consistency and quality, please adhere to the following guidelines:

- **Code Style**:
  - **Frontend (Vue/TypeScript)**: Follow the [Vue Style Guide](https://vuejs.org/style-guide/) and use Prettier for formatting.
  - **Backend (Rust)**: Adhere to Rust’s formatting with `cargo fmt` and clippy for linting (`cargo clippy`).
  - Use meaningful variable names and add comments for complex logic.
- **File Structure**:
  - Place frontend code in `src/` (Vue components, TypeScript).
  - Place backend code in `src-tauri/` (Rust, Tauri integration).
  - Update `build_settings.yaml` or `defaults.rs` for new build parameters.
- **Commits**:
  - Write clear, concise commit messages (e.g., `Add device type validation to build_settings.yaml`).
  - Use the present tense (e.g., “Fix bug” instead of “Fixed bug”).
  - Reference related issues (e.g., `Fixes #123`).
- **Tests**:
  - Add unit tests for new backend logic in `src-tauri/tests/`.
  - Ensure existing tests pass with `cargo test`.
  - Manually test UI changes in the Tauri app.

---

## 🚀 Submitting a Pull Request

Follow these steps to submit your changes:

1. **Create a Branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```
   Use descriptive names (e.g., `fix/yaml-parser-bug`, `feature/add-language-support`).

2. **Make Changes**:
   - Implement your feature or bug fix.
   - Update documentation if necessary (e.g., `README.md` or `build_settings.yaml` examples).
   - Ensure `package-lock.json` is updated and committed if dependencies change.

3. **Test Your Changes**:
   - Run the app locally (`npm run tauri dev`) and verify functionality.
   - Check for regressions in existing features (e.g., build execution, UI responsiveness).
   - Run `cargo test` for backend tests.

4. **Commit and Push**:
   ```bash
   git add .
   git commit -m "Add your descriptive commit message"
   git push origin feature/your-feature-name
   ```

5. **Open a Pull Request**:
   - Go to the [GitHub repository](https://github.com/your-username/STM32-GUI-Builder/pulls) and create a pull request from your branch.
   - Fill out the [PULL_REQUEST_TEMPLATE.md](PULL_REQUEST_TEMPLATE.md) with details about your changes.
   - Link related issues (e.g., `Fixes #123`).
   - Request a review from maintainers.

6. **Address Feedback**:
   - Respond to reviewer comments and make requested changes.
   - Push updates to the same branch to keep the pull request current.

---

## ✅ Pull Request Review Process

- **Review Timeline**: We aim to review pull requests within **7 days**. Larger changes may take longer.
- **Approval**: Pull requests require at least one maintainer’s approval. We check for code quality, functionality, and adherence to guidelines.
- **Merging**: Once approved, the pull request will be merged into the `main` branch. We may squash commits for a cleaner history.
- **Rejections**: If a pull request is not accepted, we’ll provide clear feedback on why and suggest next steps.

---

## 🤝 Community Guidelines

We strive to maintain a welcoming and inclusive community. Please:
- Be respectful and constructive in discussions.
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md) (if available, or replace with your preferred guidelines).
- Ask questions if you’re unsure—there are no silly questions!

---

## 🙌 Thank You!

Your contributions help make **STM32-GUI-Builder** a better tool for the STM32 development community. Whether it’s a bug fix, new feature, or documentation improvement, we appreciate your time and effort! 🌟

If you have questions, feel free to reach out via [GitHub Issues](https://github.com/your-username/STM32-GUI-Builder/issues) or comment on your pull request.

Happy coding! 💻
