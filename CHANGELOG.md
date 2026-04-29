# Changelog

All notable changes to this fork are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] — 2026-04-29

This release introduces CalDAV sync and a number of related quality-of-life
improvements.

### Added

- **CalDAV sync.** Two-way sync against any RFC-4791 server (Nextcloud,
  Radicale, SOGo, Fastmail, …). Calendars that advertise `VTODO` support are
  auto-discovered; a local list is created for each.
- **Account credentials in the keyring.** The CalDAV password is stored via
  the system Secret Service (cosmic-keyring), never in `cosmic-config` on
  disk.
- **Push on edit + 60 s background sync.** Local edits trigger a push
  shortly after they happen; a tick subscription also runs a full sync
  every minute.
- **Sync triggers everywhere.**
    - A sync icon in the header bar that disables itself while a sync is
      in progress.
    - "Sync now" entry under the **View** menu.
    - "Sync now" in the per-list right-click menu in the sidebar.
- **Account settings panel.** The old "Sync (CalDAV)" section is now a
  proper "Account" panel: status row with icon and message ("Signed in
  as user@example.com" / "Not configured" / "Syncing…" / error), helper
  text under each input, a "Last synced" relative timestamp ("just now",
  "5 minutes ago"), and a destructive "Sign out" button that wipes the
  saved URL/username and removes the keyring entry.
- **Due-date badge on each task row.** Renders "Today", "Tomorrow",
  "Yesterday", an upcoming weekday, or `YYYY-MM-DD` otherwise — using the
  user's local timezone.
- **Sort by due date.** New "Due date (Earliest first)" / "(Latest first)"
  entries in the Sort menu. Completed tasks always sink to the bottom
  regardless of the chosen sort.
- **`List::remote_url`.** Lists carry an explicit `Option<String>` field
  binding them to a CalDAV resource. Replaces the previous practice of
  stuffing `caldav:URL` into the description; legacy lists are migrated
  automatically on first sync.
- **`CHANGELOG.md`** and a CalDAV section in `README.md`.

### Changed

- **Date picker now stores local-midnight, not UTC-midnight.** Picking
  "April 28" stays "April 28" regardless of viewer timezone, both in the
  UI and on the wire.
- **All-day DUE is emitted as `VALUE=DATE`.** RFC-correct encoding for an
  all-day VTODO; the previous `DUE:…T000000Z` form caused other clients to
  shift the displayed day.
- **Date format on the details pane** changed from `MM-DD-YYYY` to ISO
  `YYYY-MM-DD`.
- **`DTSTAMP` is always emitted** on outgoing VTODOs — some servers refuse
  components without one.
- **iCalendar parsing is far more permissive.** Uses
  `icalendar::Todo::get_due()` so all RFC 5545 forms (`DATE`, `DATE-TIME`
  UTC / floating / TZID) are accepted; falls back to a textual parse for
  ISO-8601 with separators and offsets.
- **Bumped to libcosmic 1.0** flatpak permissions: `--share=network` and
  `--talk-name=org.freedesktop.secrets`.
- **About dialog** now reads the version from `CARGO_PKG_VERSION`.

### Fixed

- **Pulled VTODOs now appear immediately in the active list.** Previously
  `Message::SetList` short-circuited when the list id was unchanged, so
  tasks pulled from CalDAV would land on disk but stay invisible until the
  user reselected the list. A new `Message::ReloadTasks` is dispatched
  after every successful sync.
- **Date dialog now persists the picked date.** The Calendar dialog's
  `Complete` handler used to call `details.update(...)` directly,
  discarding the resulting `RefreshTask`/`Mutated` outputs. It now routes
  through the regular update path so the in-memory task copy and the
  sidebar refresh, and a sync is triggered.
- **`invalid SecondaryMap key used` panic.** Rewriting the slotmap on
  `SetTasks` (which `ReloadTasks` triggers) could leave message handlers
  dereferencing stale `DefaultKey`s. All `task_input_ids[..]` /
  `sub_task_input_ids[..]` accesses now use `.get()` and bail out when the
  key is gone.
- **Rename / Set-Icon dialogs** now correctly target the entity passed in
  from the nav context menu — previously they always wrote back to the
  active list.
- **Calendar URLs without a trailing slash** had `Url::join("uid.ics")`
  silently replace the last path segment. Trailing slashes are now
  enforced when discovering calendars.
- **Stale CalDAV XML parser branch.** Removed a `b"collection"` branch
  whose `&& false` placeholder made it dead code.
- **Removed `unsafe impl Send for List`.** It was unnecessary (`PathBuf` is
  already `Send`) and unsound to assert manually.

### Removed

- **`sync_password` from `TasksConfig`.** Passwords live only in the
  keyring now; the legacy plaintext-password migration block in `init` is
  gone.
- **`sqlx` dependency.** Nothing in the codebase used it; the only
  reference was a dead `Error::Sqlx` variant. Removing it shaves a
  significant chunk off the build graph.
- **`caldav:` description marker.** Superseded by `List::remote_url`; the
  marker is still read once for migration and then stripped.

### Tests

- New unit tests cover legacy-marker parsing, `remote_url` precedence,
  marker stripping, ISO-8601 date parsing (UTC, offset, extended), garbage
  rejection, and the all-day round-trip emitting `VALUE=DATE`.

[0.3.0]: https://github.com/edfloreshz/tasks/compare/v0.2.0...v0.3.0
