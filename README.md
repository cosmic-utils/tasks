<div align="center">
  <br>
  <img src="res/icons/hicolor/scalable/apps/dev.edfloreshz.Tasks.svg" width="150" />
  <h1>Tasks</h1>

  <p>A simple task management application for the COSMIC™ desktop</p>

  ![window-light.png](https://raw.githubusercontent.com/edfloreshz/tasks/main/res/screenshots/window-light.png#gh-light-mode-only)
  ![window-dark.png](https://raw.githubusercontent.com/edfloreshz/tasks/main/res/screenshots/window-dark.png#gh-dark-mode-only)

  <a href='https://flathub.org/apps/dev.edfloreshz.Tasks'>
    <img width='200' alt='Get it on Flathub' src='https://flathub.org/api/badge?locale=en'/>
  </a>
</div>

# CalDAV sync

This fork adds two-way CalDAV sync so your task lists can stay in sync with
servers like Nextcloud, Radicale, SOGo, and Fastmail.

Configure it from **Settings → Sync (CalDAV)**:

- **Server URL** — the root DAV path, e.g. `https://cloud.example.com/remote.php/dav/`
- **Username** — your account name
- **Password** — an app password (recommended for Nextcloud / Fastmail). Stored
  in the system keyring (Secret Service / cosmic-keyring), never on disk.

Then hit **Test connection** and **Sync now**. Once configured:

- Edits push automatically a moment after they happen.
- A periodic sync runs in the background every 60 seconds.
- A sync icon appears in the header bar and as a "Sync now" entry in the
  **View** menu and per-list right-click menu.

Remote calendars that support `VTODO` are auto-discovered; one local list is
created for each. Conflicts use `LAST-MODIFIED` to pick a winner. Deletes are
not yet propagated — see [CHANGELOG.md](CHANGELOG.md).

### Flatpak permissions

The Flatpak manifest already requests `--share=network` and the secret service.
If you build a sandboxed copy yourself, make sure those are present, otherwise
the keyring write or the HTTPS request will silently fail.

# Installation
```
git clone https://github.com/edfloreshz/tasks.git
cd tasks
sudo just install
```

# Build
```
git clone https://github.com/edfloreshz/tasks.git
cd tasks
cargo build
```

# Flatpak
To build the cargo sources for the Flatpak manifest:

```
python3 ./flatpak/flatpak-cargo-generator.py ./Cargo.lock -o ./flatpak/cargo-sources.json
appstreamcli validate --pedantic --explain res/dev.edfloreshz.Tasks.metainfo.xml
```

## Dependencies
- [libcosmic](https://github.com/pop-os/libcosmic?tab=readme-ov-file#building)

# Copyright and licensing

Copyright 2024 © Eduardo Flores

Tasks is released under the terms of the [GPL-3.0](https://github.com/edfloreshz/tasks/blob/main/LICENSE)
