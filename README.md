<div align="center">
  <br>
  <img src="res/icons/hicolor/scalable/apps/com.github.digit1024.ms-todo-app.svg" width="150" />
  <h1>MS TODO App</h1>

  <p>A Microsoft TODO management application for the COSMIC™ desktop</p>
  
  ![baner.png](https://raw.githubusercontent.com/digit1024/msToDO/main/res/screenshots/baner.png)

  ![window-light.png](https://raw.githubusercontent.com/digit1024/msToDO/main/res/screenshots/window-light.png#gh-light-mode-only)
  ![window-dark.png](https://raw.githubusercontent.com/digit1024/msToDO/main/res/screenshots/window-dark.png#gh-dark-mode-only)

  <a href='https://flathub.org/apps/com.github.digit1024.ms-todo-app'>
    <img width='200' alt='Get it on Flathub' src='https://flathub.org/api/badge?locale=en'/>
  </a>
</div>

# About

This is a fork of the original [Tasks](https://github.com/cosmic-utils/tasks) application by Eduardo Flores, rebranded and focused on Microsoft TODO integration for the COSMIC desktop environment.

## Original Developer
- **Eduardo Flores** - Original creator of the Tasks application
- **Repository**: https://github.com/cosmic-utils/tasks
- **Contact**: edfloreshz@proton.me

## Fork Maintainer
- **digit1024** - Current maintainer of MS TODO App
- **Repository**: https://github.com/digit1024/msToDO
- **Support**: https://buymeacoffee.com/digit1024

# Installation

```bash
git clone https://github.com/digit1024/msToDO.git
cd msToDO
sudo just install
```

# Build

```bash
git clone https://github.com/digit1024/msToDO.git
cd msToDO
cargo build
```

# CLI Usage

MS TODO App includes a comprehensive command-line interface for automation and scripting:

```bash
# GUI mode (no arguments)
./ms-todo-app

# CLI mode (with commands)
./ms-todo-app lists --include-virtual
./ms-todo-app tasks <list-id> --today
./ms-todo-app create --list <list-id> --title "New task" --due-date "2025-10-15"
./ms-todo-app update <task-id> --status finished
./ms-todo-app delete <task-id> --yes
```

**Features:**
- 📋 List management and task queries
- ✅ Create, update, and delete tasks
- 🎯 Filter by date, status, and keywords
- 📊 JSON output for scripting (`-o json`)
- 🔄 POSIX exit codes for automation
- 🤖 Perfect for cron jobs and shell scripts

See [CLI_USAGE.md](CLI_USAGE.md) for detailed documentation and scripting examples.

# Flatpak

To build the cargo sources for the Flatpak manifest:

```bash
python3 ./flatpak/flatpak-cargo-generator.py ./Cargo.lock -o ./flatpak/cargo-sources.json
appstreamcli validate --pedantic --explain res/com.github.digit1024.ms-todo-app.metainfo.xml
```

## Dependencies

- [libcosmic](https://github.com/pop-os/libcosmic?tab=readme-ov-file#building)

# Copyright and licensing

Copyright 2024 © Eduardo Flores (Original Tasks app)
Copyright 2025 © digit1024 (MS TODO App fork)

MS TODO App is released under the terms of the [GPL-3.0](https://github.com/digit1024/msToDO/blob/main/LICENSE)

## Support the Project

If you find this application useful, consider supporting the development:

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/digit1024)
