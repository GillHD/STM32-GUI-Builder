# 🚀 Pull Request Template

Thank you for contributing to **STM32-GUI-Builder**! 🎉 Please fill out this template to help us review your pull request efficiently. Clear and detailed information ensures a smooth review process.

---

## 📝 Description

Provide a concise summary of the changes introduced in this pull request. Explain **what** was changed, **why** it was necessary, and **how** it improves the project.

- **What**: (e.g., Added support for new device types in `build_settings.yaml`.)
- **Why**: (e.g., To allow more flexible hardware configurations.)
- **How**: (e.g., Updated the YAML schema parser and UI components.)

---

## 🛠 Type of Change

Check all that apply:

- [ ] 🐛 Bug fix (non-breaking change that fixes an issue)
- [ ] ✨ New feature (non-breaking change that adds functionality)
- [ ] ⚠️ Breaking change (fix or feature that would cause existing functionality to change)
- [ ] 📚 Documentation update
- [ ] 🧹 Code refactoring
- [ ] 🔧 Build or CI-related changes
- [ ] 🎨 UI/UX improvements

---

## ✅ Testing

Describe how you tested your changes to ensure they work as expected. Include details about the environment, steps, and any relevant test cases.

- **Environment**: (e.g., Windows 11, Node.js v18, Rust v1.70, STM32CubeIDE v1.12)
- **Steps**:
  1. (e.g., Ran `npm run tauri dev` and verified new UI elements.)
  2. (e.g., Executed builds with updated `build_settings.yaml` and checked `build_config.h` output.)
- **Test Cases**:
  - (e.g., Confirmed new device type range 4-32 is respected.)
  - (e.g., Verified no regressions in existing build functionality.)

If automated tests were added or updated, mention them here:
- (e.g., Added unit tests for YAML schema validation in `tests/schema.rs`.)

---

## 📸 Screenshots (if applicable)

If your changes affect the UI or produce visible output, include screenshots or GIFs to showcase the results. Drag and drop images here or link to them.

---

## 🔗 Related Issues

Link any related GitHub issues or discussions. Use `#` followed by the issue number to auto-link (e.g., `#123`).

- Fixes # (issue number)
- Related to # (issue number)

---

## 📋 Checklist

Please confirm the following before submitting:

- [ ] My code follows the project's [coding style guidelines](link-to-style-guide-if-exists).
- [ ] I have tested my changes thoroughly and they do not introduce new issues.
- [ ] I have updated the documentation where necessary (e.g., `README.md`, inline comments).
- [ ] My changes are compatible with the supported versions of STM32CubeIDE.
- [ ] I have added or updated tests where applicable.
- [ ] My pull request is based on the latest `main` branch and has no merge conflicts.

---

## 💬 Additional Context

Provide any extra information that might help reviewers understand your changes. For example:
- Challenges faced during implementation.
- Trade-offs made in the design.
- Future improvements planned for this feature.

---

## 🙌 Thank You!

We appreciate your time and effort in improving **STM32-GUI-Builder**! 🌟 If you have questions or need assistance during the review process, feel free to reach out in the comments below.
