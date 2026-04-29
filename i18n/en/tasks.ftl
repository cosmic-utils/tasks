tasks = Tasks
trash = Trash
about = About

# Content
add-new-task = Add new task
search-tasks = Search tasks

# Details
title = Title
details = Details
favorite = Favorite
priority = Priority
due-date = Due date
reminder = Reminder
notes = Notes
add-notes = Add notes

# Empty
no-tasks = No tasks
no-tasks-suggestion = Try adding a task with the text field below
no-list-selected = No list selected
no-list-suggestion = Create or select a new list to get started

sub-tasks = Sub-tasks
add-sub-task = Add sub-task

# New List Dialog
create-list = Create a new list

# Rename List Dialog
rename-list = Rename list

# Rename List Dialog
delete-list = The selected list is about to be deleted
delete-list-confirm = Are you sure you want to delete this list?

# Icon Dialog
icon = Set icon
icon-select = Select an icon
icon-select-body = Choose an icon for the list
search-icons = Search icons...

# Date Dialog
select-date = Select a date

# Export Dialog
export = Export
export-save-to-file = Save to file…

# Import (file → File menu entry)
import = Import from markdown

# Dialogs
cancel = Cancel
ok = Ok
copy = Copy
confirm = Confirm
save = Save
list-name = List name

# Context Pages

## About
git-description = Git commit {$hash} on {$date}

## Settings
settings = Settings

### Appearance
appearance = Appearance
theme = Theme
match-desktop = Match desktop
dark = Dark
light = Light

### Privacy
privacy = Privacy
encrypt-notes = Encrypt notes at rest
encrypt-notes-description = Encrypt the notes field of each task using a key stored in the system keyring. Reads always auto-detect, so toggling this on or off does not require a migration. CalDAV sync still pushes plaintext to the server (decryption happens locally before upload).

### Account (CalDAV sync)
account = Account
account-description = Sync your tasks with a CalDAV server such as Nextcloud, Radicale, SOGo or Fastmail. Credentials are stored in the system keyring.
sync-server-url = Server URL
sync-server-url-hint = https://cloud.example.com/remote.php/dav/
sync-server-url-description = The root DAV path of your account.
sync-username = Username
sync-username-hint = user@example.com
sync-username-description = Usually your email or login name.
sync-password = Password
sync-password-hint = Password or app password
sync-password-description = Tip: many providers (Nextcloud, Fastmail, iCloud) require an app-specific password rather than your main account password.
sync-test-connection = Test connection
sync-now = Sync now
sync-sign-out = Sign out
sync-sign-out-confirm-title = Sign out
sync-sign-out-confirm-body = Remove your CalDAV server URL and username from this device, and delete the password from the keyring? Local task lists will not be deleted.
account-status = Status
account-status-not-configured = Not configured
account-status-ready = Signed in as {$username}
account-status-syncing = Syncing…
account-status-error = Error: {$error}
account-last-sync = Last synced
account-last-sync-never = Never
account-last-sync-just-now = just now
account-last-sync-minutes = {$count ->
    [one] {$count} minute ago
   *[other] {$count} minutes ago
}
account-last-sync-hours = {$count ->
    [one] {$count} hour ago
   *[other] {$count} hours ago
}
account-last-sync-days = {$count ->
    [one] {$count} day ago
   *[other] {$count} days ago
}
sync-testing = Testing connection…
sync-test-ok = Connection OK.
sync-test-fail = Connection failed: {$error}
sync-running = Syncing…
sync-done = Sync complete. Lists added: {$lists}, tasks pulled: {$pulled}, pushed: {$pushed}, failed: {$failed}.
sync-fail = Sync failed: {$error}

# Menu

## File
file = File
new-window = New window
new-list = New list
quit = Quit

## Edit
edit = Edit
rename = Rename
delete = Delete

## View
view = View
menu-settings = Settings
menu-about = About Tasks...
hide-completed = Hide completed

## About
repository = Repository
support = Support
website = Website

## Error
cause = Cause
oops-something-wrong = Oops! Something went wrong.
error-title = Tasks - Error

# Sort Menu
sort = Sort
sort-name-asc = Name A-Z
sort-name-desc = Name Z-A
sort-date-asc = Date added (Old to New)
sort-date-desc = Date added (New to Old)
sort-due-asc = Due date (Earliest first)
sort-due-desc = Due date (Latest first)

# Due-date badges
due-today = Today
due-tomorrow = Tomorrow
due-yesterday = Yesterday
