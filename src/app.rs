pub mod actions;
pub mod context;
pub mod dialog;
pub mod error;
mod flags;
pub mod markdown;
pub mod menu;

pub use flags::*;
use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
    env, process,
};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use cosmic::{
    Application, ApplicationExt, Element,
    app::{self, Core},
    cosmic_config::{self, Update},
    cosmic_theme::{self, ThemeMode},
    iced::{
        Event, Length, Subscription,
        keyboard::{Event as KeyEvent, Modifiers},
    },
    widget::{
        self,
        calendar::CalendarModel,
        menu::{Action as _, key_bind::KeyBind},
        segmented_button::{Entity, EntityMut, SingleSelect},
    },
};

use crate::{
    app::{
        actions::{Action, ApplicationAction, NavMenuAction, TasksAction},
        context::ContextPage,
        dialog::{DialogAction, DialogPage},
    },
    core::{
        config::{self, CONFIG_VERSION},
        icons,
        key_bind::key_binds,
    },
    fl,
    pages::{
        content::{self, Content},
        details::{self, Details},
    },
    storage::{LocalStorage, models::List},
};

pub struct Tasks {
    core: Core,
    about: widget::about::About,
    nav_model: widget::segmented_button::SingleSelectModel,
    storage: LocalStorage,
    content: Content,
    details: Details,
    config_handler: Option<cosmic_config::Config>,
    config: config::TasksConfig,
    app_themes: Vec<String>,
    context_page: ContextPage,
    key_binds: HashMap<KeyBind, Action>,
    modifiers: Modifiers,
    dialog_pages: VecDeque<DialogPage>,
    dialog_text_input: widget::Id,
    sync_status: String,
    sync_in_progress: bool,
    sync_password: String,
    sync_last_at: Option<chrono::DateTime<chrono::Utc>>,
    sync_last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Content(content::Message),
    Details(details::Message),
    Tasks(TasksAction),
    Application(ApplicationAction),
    Open(String),
}

impl Tasks {
    fn settings(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let appearance =
            widget::settings::section()
                .title(fl!("appearance"))
                .add(widget::settings::item::item(
                    fl!("theme"),
                    widget::dropdown(
                        &self.app_themes,
                        Some(self.config.app_theme.into()),
                        |theme| Message::Application(ApplicationAction::AppTheme(theme)),
                    ),
                ));

        let privacy = widget::settings::section()
            .title(fl!("privacy"))
            .add(widget::settings::item::builder(fl!("encrypt-notes")).description(fl!("encrypt-notes-description")).control(
                widget::toggler(self.config.encrypt_notes)
                    .on_toggle(|on| Message::Application(ApplicationAction::ToggleEncryptNotes(on))),
            ));

        let creds = self.sync_credentials();
        let configured = crate::sync::engine::is_configured(&creds);

        // --- status row -------------------------------------------------
        let (status_icon, status_text, status_class) = if self.sync_in_progress {
            (
                "process-working-symbolic",
                fl!("account-status-syncing"),
                cosmic::style::Text::Default,
            )
        } else if let Some(err) = &self.sync_last_error {
            (
                "dialog-error-symbolic",
                fl!("account-status-error", error = err.as_str()),
                cosmic::style::Text::Color(cosmic::iced::Color::from_rgb(0.86, 0.30, 0.30)),
            )
        } else if configured {
            (
                "emblem-default-symbolic",
                fl!(
                    "account-status-ready",
                    username = self.config.sync_username.as_str()
                ),
                cosmic::style::Text::Accent,
            )
        } else {
            (
                "dialog-information-symbolic",
                fl!("account-status-not-configured"),
                cosmic::style::Text::Default,
            )
        };
        let status_row = widget::row::with_children(vec![
            icons::get_icon(status_icon, 16).into(),
            widget::text(status_text).class(status_class).into(),
        ])
        .align_y(cosmic::iced::Alignment::Center)
        .spacing(spacing.space_xs);

        let last_sync_row = widget::settings::item::item(
            fl!("account-last-sync"),
            widget::text(format_relative_time(self.sync_last_at)),
        );

        // --- credential inputs -----------------------------------------
        let url_input =
            widget::text_input(fl!("sync-server-url-hint"), &self.config.sync_server_url)
                .on_input(|s| Message::Application(ApplicationAction::SetSyncServerUrl(s)));
        let user_input = widget::text_input(fl!("sync-username-hint"), &self.config.sync_username)
            .on_input(|s| Message::Application(ApplicationAction::SetSyncUsername(s)));
        let pass_input =
            widget::secure_input(fl!("sync-password-hint"), &self.sync_password, None, true)
                .on_input(|s| Message::Application(ApplicationAction::SetSyncPassword(s)));

        let url_field = widget::column::with_children(vec![
            widget::text::body(fl!("sync-server-url")).into(),
            url_input.into(),
            widget::text::caption(fl!("sync-server-url-description")).into(),
        ])
        .spacing(spacing.space_xxxs);
        let user_field = widget::column::with_children(vec![
            widget::text::body(fl!("sync-username")).into(),
            user_input.into(),
            widget::text::caption(fl!("sync-username-description")).into(),
        ])
        .spacing(spacing.space_xxxs);
        let pass_field = widget::column::with_children(vec![
            widget::text::body(fl!("sync-password")).into(),
            pass_input.into(),
            widget::text::caption(fl!("sync-password-description")).into(),
        ])
        .spacing(spacing.space_xxxs);

        // --- buttons ---------------------------------------------------
        let test_button = widget::button::standard(fl!("sync-test-connection")).on_press_maybe(
            (!self.sync_in_progress && configured)
                .then_some(Message::Application(ApplicationAction::TestSyncConnection)),
        );
        let sync_button = widget::button::suggested(fl!("sync-now")).on_press_maybe(
            (!self.sync_in_progress && configured)
                .then_some(Message::Application(ApplicationAction::SyncNow)),
        );
        let mut button_children: Vec<Element<'_, Message>> = vec![
            test_button.into(),
            widget::horizontal_space()
                .width(cosmic::iced::Length::Fixed(8.0))
                .into(),
            sync_button.into(),
        ];
        if configured {
            button_children.push(widget::horizontal_space().width(Length::Fill).into());
            button_children.push(
                widget::button::destructive(fl!("sync-sign-out"))
                    .on_press(Message::Application(ApplicationAction::SignOut))
                    .into(),
            );
        }
        let buttons =
            widget::row::with_children(button_children).align_y(cosmic::iced::Alignment::Center);

        // --- assemble --------------------------------------------------
        let mut account_section = widget::settings::section()
            .title(fl!("account"))
            .add(
                widget::column::with_children(vec![
                    widget::text::caption(fl!("account-description")).into(),
                    status_row.into(),
                ])
                .spacing(spacing.space_xs)
                .padding([
                    spacing.space_xs,
                    spacing.space_none,
                    spacing.space_s,
                    spacing.space_none,
                ]),
            )
            .add(url_field)
            .add(user_field)
            .add(pass_field)
            .add(last_sync_row)
            .add(buttons);

        if !self.sync_status.is_empty() {
            account_section = account_section.add(widget::text::caption(self.sync_status.clone()));
        }

        widget::scrollable(
            widget::column::with_children(vec![
                appearance.into(),
                privacy.into(),
                account_section.into(),
            ])
            .spacing(spacing.space_m),
        )
        .into()
    }

    fn create_nav_item(&mut self, list: &List) -> EntityMut<'_, SingleSelect> {
        let icon =
            crate::app::icons::get_icon(list.icon.as_deref().unwrap_or("view-list-symbolic"), 16);
        self.nav_model
            .insert()
            .text(list.name.clone())
            .icon(icon)
            .data(list.clone())
    }

    fn update_content(
        &mut self,
        tasks: &mut Vec<cosmic::Task<cosmic::Action<Message>>>,
        message: content::Message,
    ) {
        let content_tasks = self.content.update(message);
        for content_task in content_tasks {
            match content_task {
                content::Output::Focus(id) => {
                    tasks.push(self.update(Message::Application(ApplicationAction::Focus(id))))
                }
                content::Output::OpenTaskDetails(task) => {
                    let entity = self.details.priority_model.entity_at(task.priority as u16);
                    if let Some(entity) = entity {
                        self.details.priority_model.activate(entity);
                    }
                    self.details.task = task.clone();
                    self.details.text_editor_content =
                        widget::text_editor::Content::with_text(&task.notes);

                    tasks.push(self.update(Message::Application(
                        ApplicationAction::ToggleContextPage(ContextPage::TaskDetails),
                    )));
                }
                content::Output::ToggleHideCompleted(list) => {
                    if let Some(data) = self.nav_model.active_data_mut::<List>() {
                        data.hide_completed = list.hide_completed;
                        if let Err(err) = self.storage.update_list(&list) {
                            tracing::error!("Error updating list: {err}");
                        }
                    }
                }
                content::Output::Mutated => {
                    self.maybe_trigger_sync(tasks);
                }
            }
        }
    }

    fn update_details(
        &mut self,
        tasks: &mut Vec<cosmic::Task<cosmic::Action<Message>>>,
        message: details::Message,
    ) {
        let details_tasks = self.details.update(message);
        for details_task in details_tasks {
            match details_task {
                details::Output::OpenCalendarDialog => {
                    tasks.push(self.update(Message::Application(ApplicationAction::Dialog(
                        DialogAction::Open(DialogPage::Calendar(CalendarModel::now())),
                    ))));
                }
                details::Output::RefreshTask(task) => {
                    tasks.push(self.update(Message::Content(content::Message::RefreshTask(
                        task.clone(),
                    ))));
                }
                details::Output::Mutated => {
                    self.maybe_trigger_sync(tasks);
                }
            }
        }
    }

    fn sync_credentials(&self) -> crate::sync::engine::SyncCredentials {
        crate::sync::engine::SyncCredentials {
            server_url: self.config.sync_server_url.clone(),
            username: self.config.sync_username.clone(),
            password: self.sync_password.clone(),
        }
    }

    fn maybe_trigger_sync(&mut self, tasks: &mut Vec<cosmic::Task<cosmic::Action<Message>>>) {
        if self.sync_in_progress {
            return;
        }
        if !crate::sync::engine::is_configured(&self.sync_credentials()) {
            return;
        }
        tasks.push(self.update(Message::Application(ApplicationAction::SyncNow)));
    }

    fn update_dialog(
        &mut self,
        tasks: &mut Vec<cosmic::Task<cosmic::Action<Message>>>,
        dialog_action: DialogAction,
    ) {
        match dialog_action {
            DialogAction::Open(page) => {
                match page {
                    DialogPage::Rename(entity, _) => {
                        let data = if let Some(entity) = entity {
                            self.nav_model.data::<List>(entity)
                        } else {
                            self.nav_model.active_data::<List>()
                        };
                        if let Some(list) = data {
                            self.dialog_pages
                                .push_back(DialogPage::Rename(entity, list.name.clone()));
                        }
                    }
                    page => self.dialog_pages.push_back(page),
                }
                tasks.push(self.update(Message::Application(ApplicationAction::Focus(
                    self.dialog_text_input.clone(),
                ))));
            }
            DialogAction::Update(dialog_page) => {
                self.dialog_pages[0] = dialog_page;
            }
            DialogAction::Close => {
                self.dialog_pages.pop_front();
            }
            DialogAction::Complete => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::New(name) => {
                            let list = List::new(&name);
                            match self.storage.create_list(&list) {
                                Ok(list) => {
                                    tasks.push(
                                        self.update(Message::Tasks(TasksAction::AddList(list))),
                                    );
                                }
                                Err(err) => {
                                    tracing::error!("Error creating list: {err}");
                                }
                            }
                        }
                        DialogPage::Rename(entity, name) => {
                            let target = entity.unwrap_or_else(|| self.nav_model.active());
                            if let Some(list) = self.nav_model.data_mut::<List>(target) {
                                list.name.clone_from(&name);
                                let list = list.clone();
                                self.nav_model.text_set(target, name.clone());
                                if let Err(err) = self.storage.update_list(&list) {
                                    tracing::error!("Error updating list: {err}");
                                }
                                if target == self.nav_model.active() {
                                    tasks.push(self.update(Message::Content(
                                        content::Message::SetList(Some(list)),
                                    )));
                                }
                            }
                        }
                        DialogPage::Delete(entity) => {
                            tasks
                                .push(self.update(Message::Tasks(TasksAction::DeleteList(entity))));
                        }
                        DialogPage::Icon(entity, name, _) => {
                            let target = entity.unwrap_or_else(|| self.nav_model.active());
                            if let Some(list) = self.nav_model.data_mut::<List>(target) {
                                list.icon = Some(name.clone());
                                let list = list.clone();
                                self.nav_model
                                    .icon_set(target, crate::app::icons::get_icon(&name, 16));
                                if let Err(err) = self.storage.update_list(&list) {
                                    tracing::error!("Error updating list: {err}");
                                }
                                if target == self.nav_model.active() {
                                    tasks.push(self.update(Message::Content(
                                        content::Message::SetList(Some(list)),
                                    )));
                                }
                            }
                        }
                        DialogPage::Calendar(date) => {
                            // Route through update_details so the resulting
                            // RefreshTask/Mutated outputs are dispatched —
                            // calling self.details.update directly would drop
                            // them and leave the task list stale on screen.
                            tasks.push(self.update(Message::Details(
                                details::Message::SetDueDate(date.selected),
                            )));
                        }
                        DialogPage::Export(content, _filename) => {
                            let Ok(mut clipboard) = ClipboardContext::new() else {
                                tracing::error!("Clipboard is not available");
                                return;
                            };
                            if let Err(error) = clipboard.set_contents(content) {
                                tracing::error!("Error setting clipboard contents: {error}");
                            }
                        }
                    }
                }
            }
            DialogAction::None => (),
        }
    }

    fn update_app(
        &mut self,
        tasks: &mut Vec<cosmic::Task<cosmic::Action<Message>>>,
        application_action: ApplicationAction,
    ) {
        match application_action {
            ApplicationAction::WindowClose => {
                if let Some(window_id) = self.core.main_window_id() {
                    tasks.push(cosmic::iced::window::close(window_id));
                }
            }
            ApplicationAction::WindowNew => match env::current_exe() {
                Ok(exe) => match process::Command::new(&exe).spawn() {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("failed to execute {exe:?}: {err}");
                    }
                },
                Err(err) => {
                    eprintln!("failed to get current executable path: {err}");
                }
            },
            ApplicationAction::AppTheme(theme) => {
                if let Some(handler) = &self.config_handler {
                    if let Err(err) = self.config.set_app_theme(handler, theme.into()) {
                        tracing::error!("{err}")
                    }
                }
            }
            ApplicationAction::ToggleHideCompleted(value) => {
                if let Some(handler) = &self.config_handler {
                    if let Err(err) = self.config.set_hide_completed(handler, value) {
                        tracing::error!("{err}")
                    }
                    tasks.push(self.update(Message::Content(content::Message::SetConfig(
                        self.config.clone(),
                    ))));
                }
            }
            ApplicationAction::SystemThemeModeChange => {
                tasks.push(cosmic::command::set_theme(self.config.app_theme.theme()));
            }
            ApplicationAction::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.clone().into_iter() {
                    if key_bind.matches(modifiers, &key) {
                        tasks.push(self.update(action.message()));
                    }
                }
            }
            ApplicationAction::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            ApplicationAction::NavMenuAction(nav_menu_action) => match nav_menu_action {
                NavMenuAction::Rename(entity) => {
                    tasks.push(self.update(Message::Application(ApplicationAction::Dialog(
                        DialogAction::Open(DialogPage::Rename(Some(entity), String::new())),
                    ))));
                }
                NavMenuAction::SetIcon(entity) => {
                    tasks.push(self.update(Message::Application(ApplicationAction::Dialog(
                        DialogAction::Open(DialogPage::Icon(
                            Some(entity),
                            String::new(),
                            String::new(),
                        )),
                    ))));
                }
                NavMenuAction::Export(entity) => {
                    if let Some(list) = self.nav_model.data::<List>(entity) {
                        match self.storage.tasks(list) {
                            Ok(data) => {
                                let exported_markdown = LocalStorage::export_list(list, &data);
                                let default_filename = default_export_filename(&list.name);
                                tasks.push(self.update(Message::Application(
                                    ApplicationAction::Dialog(DialogAction::Open(
                                        DialogPage::Export(exported_markdown, default_filename),
                                    )),
                                )));
                            }
                            Err(err) => {
                                tracing::error!("Error fetching tasks: {err}");
                            }
                        }
                    }
                }
                NavMenuAction::Delete(entity) => {
                    tasks.push(self.update(Message::Application(ApplicationAction::Dialog(
                        DialogAction::Open(DialogPage::Delete(Some(entity))),
                    ))));
                }
                NavMenuAction::SyncNow => {
                    tasks.push(self.update(Message::Application(ApplicationAction::SyncNow)));
                }
            },
            ApplicationAction::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
                tasks.push(
                    self.update(Message::Content(content::Message::ContextMenuOpen(
                        self.core.window.show_context,
                    ))),
                );
            }
            ApplicationAction::ToggleContextDrawer => {
                self.core.window.show_context = !self.core.window.show_context;
                tasks.push(
                    self.update(Message::Content(content::Message::ContextMenuOpen(
                        self.core.window.show_context,
                    ))),
                );
            }
            ApplicationAction::Dialog(dialog_action) => self.update_dialog(tasks, dialog_action),
            ApplicationAction::Focus(id) => tasks.push(widget::text_input::focus(id)),
            ApplicationAction::SortByNameAsc => {
                tasks.push(self.update(Message::Content(content::Message::SetSort(
                    content::SortType::NameAsc,
                ))));
            }
            ApplicationAction::SortByNameDesc => {
                tasks.push(self.update(Message::Content(content::Message::SetSort(
                    content::SortType::NameDesc,
                ))));
            }
            ApplicationAction::SortByDateAsc => {
                tasks.push(self.update(Message::Content(content::Message::SetSort(
                    content::SortType::DateAsc,
                ))));
            }
            ApplicationAction::SortByDateDesc => {
                tasks.push(self.update(Message::Content(content::Message::SetSort(
                    content::SortType::DateDesc,
                ))));
            }
            ApplicationAction::SortByDueAsc => {
                tasks.push(self.update(Message::Content(content::Message::SetSort(
                    content::SortType::DueAsc,
                ))));
            }
            ApplicationAction::SortByDueDesc => {
                tasks.push(self.update(Message::Content(content::Message::SetSort(
                    content::SortType::DueDesc,
                ))));
            }
            ApplicationAction::ImportFromFile => {
                tasks.push(cosmic::Task::perform(
                    pick_and_read_markdown(),
                    |result| {
                        cosmic::Action::App(Message::Application(
                            ApplicationAction::ImportFromFileResult(result),
                        ))
                    },
                ));
            }
            ApplicationAction::ImportFromFileResult(result) => match result {
                Ok((filename, text)) => {
                    let parsed = crate::app::markdown::parse_import(&text);
                    let fallback = std::path::Path::new(&filename)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Imported")
                        .to_string();
                    match self.storage.import_list(parsed, &fallback) {
                        Ok(list) => {
                            tasks.push(self.update(Message::Tasks(TasksAction::AddList(list))));
                        }
                        Err(err) => tracing::error!("import failed: {err}"),
                    }
                }
                Err(e) if e == "cancelled" => {}
                Err(e) => tracing::error!("import picker: {e}"),
            },
            ApplicationAction::SaveExportToFile => {
                let Some(DialogPage::Export(content, default_filename)) =
                    self.dialog_pages.front().cloned()
                else {
                    return;
                };
                tasks.push(cosmic::Task::perform(
                    pick_and_save_markdown(content, default_filename),
                    |result| {
                        cosmic::Action::App(Message::Application(
                            ApplicationAction::SaveExportToFileResult(result),
                        ))
                    },
                ));
            }
            ApplicationAction::SaveExportToFileResult(result) => match result {
                Ok(_) => {
                    self.dialog_pages.pop_front();
                }
                Err(e) if e == "cancelled" => {}
                Err(e) => tracing::error!("export save: {e}"),
            },
            ApplicationAction::ToggleEncryptNotes(value) => {
                if let Some(handler) = &self.config_handler {
                    if let Err(err) = self.config.set_encrypt_notes(handler, value) {
                        tracing::error!("{err}");
                    }
                }
                self.storage.set_encrypt_notes(value);
                if value {
                    // Force the keyring entry to materialize now so the user
                    // gets an immediate prompt-to-unlock if needed, rather
                    // than the first time they edit a note's text.
                    if let Err(err) = crate::storage::notes_crypto::encrypt("warmup") {
                        tracing::warn!("notes encryption warmup failed: {err}");
                    }
                }
            }
            ApplicationAction::SetSyncServerUrl(value) => {
                if let Some(handler) = &self.config_handler {
                    if let Err(err) = self.config.set_sync_server_url(handler, value) {
                        tracing::error!("{err}");
                    }
                }
            }
            ApplicationAction::SetSyncUsername(value) => {
                let old = self.config.sync_username.clone();
                if let Some(handler) = &self.config_handler {
                    if let Err(err) = self.config.set_sync_username(handler, value.clone()) {
                        tracing::error!("{err}");
                    }
                }
                if old != value {
                    // Prefer an existing keyring entry for the new username
                    // (e.g. user is switching back to a previously configured
                    // account); otherwise migrate the in-memory password
                    // under the new key.
                    if let Some(stored) = crate::sync::secret::load(&value) {
                        self.sync_password = stored;
                    } else if !self.sync_password.is_empty() && !value.is_empty() {
                        if let Err(e) = crate::sync::secret::store(&value, &self.sync_password) {
                            tracing::warn!("keyring store under new username: {e}");
                        }
                    } else if value.is_empty() {
                        self.sync_password.clear();
                    }
                    if !old.is_empty() && old != value {
                        crate::sync::secret::delete(&old);
                    }
                }
            }
            ApplicationAction::SetSyncPassword(value) => {
                self.sync_password = value;
                let username = self.config.sync_username.clone();
                if username.is_empty() {
                    // No username yet — keep in memory; will be persisted once
                    // the username is set.
                } else if self.sync_password.is_empty() {
                    crate::sync::secret::delete(&username);
                } else if let Err(e) = crate::sync::secret::store(&username, &self.sync_password) {
                    tracing::warn!("keyring store: {e}");
                }
            }
            ApplicationAction::TestSyncConnection => {
                self.sync_in_progress = true;
                self.sync_status = fl!("sync-testing");
                let creds = self.sync_credentials();
                tasks.push(cosmic::Task::perform(
                    async move {
                        crate::sync::engine::test_connection(&creds)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |result| {
                        cosmic::Action::App(Message::Application(
                            ApplicationAction::TestSyncConnectionResult(result),
                        ))
                    },
                ));
            }
            ApplicationAction::TestSyncConnectionResult(result) => {
                self.sync_in_progress = false;
                self.sync_status = match result {
                    Ok(()) => fl!("sync-test-ok"),
                    Err(e) => fl!("sync-test-fail", error = e),
                };
            }
            ApplicationAction::SyncNow => {
                self.sync_in_progress = true;
                self.sync_status = fl!("sync-running");
                let creds = self.sync_credentials();
                let storage = self.storage.clone();
                tasks.push(cosmic::Task::perform(
                    async move {
                        crate::sync::engine::sync(&storage, &creds)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |result| {
                        cosmic::Action::App(Message::Application(ApplicationAction::SyncResult(
                            result,
                        )))
                    },
                ));
            }
            ApplicationAction::SyncTick => {
                self.maybe_trigger_sync(tasks);
            }
            ApplicationAction::SyncResult(result) => {
                self.sync_in_progress = false;
                match result {
                    Ok(report) => {
                        self.sync_last_at = Some(chrono::Utc::now());
                        self.sync_last_error = None;
                        self.sync_status = fl!(
                            "sync-done",
                            lists = report.lists_pulled,
                            pulled = report.tasks_pulled,
                            pushed = report.tasks_pushed,
                            failed = report.tasks_failed
                        );
                        tasks.push(self.update(Message::Tasks(TasksAction::FetchLists)));
                        // FetchLists won't reload the active list's tasks if the
                        // selected list id is unchanged; force a re-read so newly
                        // pulled VTODOs become visible without a manual reselect.
                        tasks.push(self.update(Message::Content(content::Message::ReloadTasks)));
                    }
                    Err(e) => {
                        self.sync_last_error = Some(e.clone());
                        self.sync_status = fl!("sync-fail", error = e);
                    }
                }
            }
            ApplicationAction::SignOut => {
                let username = self.config.sync_username.clone();
                if !username.is_empty() {
                    crate::sync::secret::delete(&username);
                }
                self.sync_password.clear();
                if let Some(handler) = &self.config_handler {
                    if let Err(err) = self.config.set_sync_server_url(handler, String::new()) {
                        tracing::error!("{err}");
                    }
                    if let Err(err) = self.config.set_sync_username(handler, String::new()) {
                        tracing::error!("{err}");
                    }
                }
                self.sync_status.clear();
                self.sync_last_at = None;
                self.sync_last_error = None;
            }
        }
    }

    fn update_tasks(
        &mut self,
        tasks: &mut Vec<cosmic::Task<cosmic::Action<Message>>>,
        tasks_action: TasksAction,
    ) {
        match tasks_action {
            TasksAction::FetchLists => match self.storage.lists() {
                Ok(lists) => {
                    tasks.push(self.update(Message::Tasks(TasksAction::PopulateLists(lists))));
                }
                Err(err) => {
                    tracing::error!("Error fetching lists: {err}");
                }
            },
            TasksAction::PopulateLists(lists) => {
                let previously_active = self.nav_model.active_data::<List>().map(|l| l.id.clone());
                self.nav_model.clear();
                for list in lists {
                    self.create_nav_item(&list);
                }
                let restore = previously_active.and_then(|id| {
                    self.nav_model.iter().find(|e| {
                        self.nav_model
                            .data::<List>(*e)
                            .map(|l| l.id == id)
                            .unwrap_or(false)
                    })
                });
                let target = restore.or_else(|| self.nav_model.iter().next());
                let Some(entity) = target else {
                    return;
                };
                self.nav_model.activate(entity);
                let task = self.on_nav_select(entity);
                tasks.push(task);
            }
            TasksAction::AddList(list) => {
                self.create_nav_item(&list);
                let Some(entity) = self.nav_model.iter().last() else {
                    return;
                };
                let task = self.on_nav_select(entity);
                tasks.push(task);
            }
            TasksAction::DeleteList(entity) => {
                let data = if let Some(entity) = entity {
                    self.nav_model.data::<List>(entity)
                } else {
                    self.nav_model.active_data::<List>()
                };
                if let Some(list) = data {
                    if let Err(err) = self.storage.delete_list(list) {
                        tracing::error!("Error deleting list: {err}");
                    }

                    tasks.push(self.update(Message::Content(content::Message::SetList(None))));
                }
                self.nav_model.remove(self.nav_model.active());
            }
        }
    }
}

impl Application for Tasks {
    type Executor = cosmic::executor::Default;
    type Flags = crate::app::Flags;
    type Message = Message;
    const APP_ID: &'static str = "dev.edfloreshz.Tasks";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, app::Task<Self::Message>) {
        let nav_model = widget::segmented_button::ModelBuilder::default().build();

        let about = widget::about::About::default()
            .name(fl!("tasks"))
            .icon(widget::icon::from_name(Self::APP_ID))
            .version(env!("CARGO_PKG_VERSION"))
            .author("Eduardo Flores")
            .license("GPL-3.0-only")
            .links([
                (fl!("repository"), "https://github.com/cosmic-utils/tasks"),
                (
                    fl!("support"),
                    "https://github.com/cosmic-utils/tasks/issues",
                ),
                (fl!("website"), "https://tasks.edfloreshz.dev"),
            ])
            .developers([("Eduardo Flores", "edfloreshz@proton.me")]);

        let mut app = Tasks {
            core,
            about,
            storage: flags.storage.clone(),
            nav_model,
            content: Content::new(flags.storage.clone()),
            details: Details::new(flags.storage),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            context_page: ContextPage::Settings,
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
            sync_status: String::new(),
            sync_in_progress: false,
            sync_password: String::new(),
            sync_last_at: None,
            sync_last_error: None,
        };

        // Load CalDAV password from the system keyring (Secret Service / cosmic-keyring).
        let username = app.config.sync_username.clone();
        if let Some(pw) = crate::sync::secret::load(&username) {
            app.sync_password = pw;
        }

        // Propagate the persisted encryption preference into the storage
        // layer. The flag governs writes; reads always auto-detect, so this
        // is safe whether or not the keyring entry already exists.
        app.storage.set_encrypt_notes(app.config.encrypt_notes);

        let mut tasks = vec![app.update(Message::Tasks(TasksAction::FetchLists))];

        if let Some(id) = app.core.main_window_id() {
            tasks.push(app.set_window_title(fl!("tasks"), id));
        }

        app.core.nav_bar_toggle_condensed();

        (app, app::Task::batch(tasks))
    }

    fn context_drawer(&self) -> Option<app::context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => app::context_drawer::about(
                &self.about,
                |url| Message::Open(url.to_string()),
                Message::Application(ApplicationAction::ToggleContextDrawer),
            )
            .title(self.context_page.title()),
            ContextPage::Settings => app::context_drawer::context_drawer(
                self.settings(),
                Message::Application(ApplicationAction::ToggleContextDrawer),
            )
            .title(self.context_page.title()),
            ContextPage::TaskDetails => app::context_drawer::context_drawer(
                self.details.view().map(Message::Details),
                Message::Application(ApplicationAction::ToggleContextDrawer),
            )
            .title(self.context_page.title()),
        })
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        let dialog_page = self.dialog_pages.front()?;
        let dialog = dialog_page.view(&self.dialog_text_input);
        Some(dialog.into())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![menu::menu_bar(&self.key_binds, &self.config)]
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        let creds = self.sync_credentials();
        if !crate::sync::engine::is_configured(&creds) {
            return vec![];
        }
        let icon = if self.sync_in_progress {
            "process-working-symbolic"
        } else {
            "emblem-synchronizing-symbolic"
        };
        let mut button = widget::button::icon(icons::get_handle(icon, 18));
        if !self.sync_in_progress {
            button = button.on_press(Message::Application(ApplicationAction::SyncNow));
        }
        vec![
            widget::tooltip(
                button,
                widget::text(fl!("sync-now")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        ]
    }

    fn nav_context_menu(
        &self,
        id: widget::nav_bar::Id,
    ) -> Option<Vec<widget::menu::Tree<cosmic::Action<Self::Message>>>> {
        Some(cosmic::widget::menu::items(
            &HashMap::new(),
            vec![
                cosmic::widget::menu::Item::Button(
                    fl!("rename"),
                    Some(icons::get_handle("edit-symbolic", 14)),
                    NavMenuAction::Rename(id),
                ),
                cosmic::widget::menu::Item::Button(
                    fl!("icon"),
                    Some(icons::get_handle("face-smile-big-symbolic", 14)),
                    NavMenuAction::SetIcon(id),
                ),
                cosmic::widget::menu::Item::Button(
                    fl!("export"),
                    Some(icons::get_handle("share-symbolic", 18)),
                    NavMenuAction::Export(id),
                ),
                cosmic::widget::menu::Item::Button(
                    fl!("sync-now"),
                    Some(icons::get_handle("emblem-synchronizing-symbolic", 14)),
                    NavMenuAction::SyncNow,
                ),
                cosmic::widget::menu::Item::Button(
                    fl!("delete"),
                    Some(icons::get_handle("user-trash-full-symbolic", 14)),
                    NavMenuAction::Delete(id),
                ),
            ],
        ))
    }

    fn nav_model(&self) -> Option<&widget::segmented_button::SingleSelectModel> {
        Some(&self.nav_model)
    }

    fn on_escape(&mut self) -> app::Task<Self::Message> {
        if self.dialog_pages.pop_front().is_some() {
            return app::Task::none();
        }

        self.core.window.show_context = false;

        app::Task::none()
    }

    fn on_nav_select(&mut self, entity: Entity) -> app::Task<Self::Message> {
        let mut tasks = vec![];
        self.nav_model.activate(entity);
        let location_opt = self.nav_model.data::<List>(entity);

        if let Some(list) = location_opt {
            let message = Message::Content(content::Message::SetList(Some(list.clone())));
            let window_title = format!("{} - {}", list.name, fl!("tasks"));
            if let Some(window_id) = self.core.main_window_id() {
                tasks.push(self.set_window_title(window_title, window_id));
            }
            return self.update(message);
        }

        app::Task::batch(tasks)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let mut subscriptions = vec![
            cosmic::iced::event::listen_with(|event, _status, _window_id| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => {
                    Some(Message::Application(ApplicationAction::Key(modifiers, key)))
                }
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => Some(
                    Message::Application(ApplicationAction::Modifiers(modifiers)),
                ),
                _ => None,
            }),
            cosmic_config::config_subscription(
                TypeId::of::<ConfigSubscription>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|update: Update<ThemeMode>| {
                if !update.errors.is_empty() {
                    tracing::info!(
                        "errors loading config {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::Application(ApplicationAction::SystemThemeModeChange)
            }),
            cosmic_config::config_subscription::<_, cosmic_theme::ThemeMode>(
                TypeId::of::<ThemeSubscription>(),
                cosmic_theme::THEME_MODE_ID.into(),
                cosmic_theme::ThemeMode::version(),
            )
            .map(|update: Update<ThemeMode>| {
                if !update.errors.is_empty() {
                    tracing::info!(
                        "errors loading theme mode {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::Application(ApplicationAction::SystemThemeModeChange)
            }),
        ];

        subscriptions.push(self.content.subscription().map(Message::Content));

        subscriptions.push(
            cosmic::iced::time::every(std::time::Duration::from_secs(60))
                .map(|_| Message::Application(ApplicationAction::SyncTick)),
        );

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Self::Message> {
        let mut tasks = vec![];
        match message {
            Message::Open(url) => {
                if let Err(err) = open::that_detached(url) {
                    tracing::error!("{err}")
                }
            }
            Message::Content(message) => {
                self.update_content(&mut tasks, message);
            }
            Message::Details(message) => {
                self.update_details(&mut tasks, message);
            }
            Message::Tasks(tasks_action) => {
                self.update_tasks(&mut tasks, tasks_action);
            }
            Message::Application(application_action) => {
                self.update_app(&mut tasks, application_action);
            }
        }

        app::Task::batch(tasks)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.content.view().map(Message::Content)
    }
}

/// Slugified default filename to seed the portal Save dialog with.
fn default_export_filename(list_name: &str) -> String {
    let slug: String = list_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "tasks.md".to_string()
    } else {
        format!("{slug}.md")
    }
}

/// Open the portal file chooser and return `(filename, contents)` for the
/// picked markdown file. Uses `rfd`'s xdg-portal backend so it works
/// inside a Flatpak sandbox without needing `--filesystem=home`. The
/// `"cancelled"` sentinel signals user cancellation, which the caller
/// treats as a no-op.
async fn pick_and_read_markdown() -> Result<(String, String), String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title(&fl!("import"))
        .add_filter("Markdown / text", &["md", "markdown", "txt"])
        .pick_file()
        .await
        .ok_or_else(|| "cancelled".to_string())?;
    let bytes = handle.read().await;
    let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    Ok((handle.file_name(), text))
}

/// Open the portal Save dialog and write `content` to the chosen path.
async fn pick_and_save_markdown(
    content: String,
    default_filename: String,
) -> Result<std::path::PathBuf, String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title(&fl!("export-save-to-file"))
        .set_file_name(&default_filename)
        .add_filter("Markdown", &["md"])
        .save_file()
        .await
        .ok_or_else(|| "cancelled".to_string())?;
    handle
        .write(content.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    Ok(handle.path().to_path_buf())
}

/// Render `at` as a coarse human-friendly relative timestamp ("just now",
/// "5 minutes ago"). Used for the last-sync row in settings.
fn format_relative_time(at: Option<chrono::DateTime<chrono::Utc>>) -> String {
    let Some(at) = at else {
        return fl!("account-last-sync-never");
    };
    let secs = (chrono::Utc::now() - at).num_seconds().max(0);
    let n = |s: i64| (s.max(0)) as i32;
    if secs < 60 {
        fl!("account-last-sync-just-now")
    } else if secs < 60 * 60 {
        fl!("account-last-sync-minutes", count = n(secs / 60))
    } else if secs < 60 * 60 * 24 {
        fl!("account-last-sync-hours", count = n(secs / 3600))
    } else {
        fl!("account-last-sync-days", count = n(secs / 86400))
    }
}
