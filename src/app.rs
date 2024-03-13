use std::{env, process};
use std::any::TypeId;
use std::collections::{HashMap, VecDeque};

use cosmic::{
    app, Application, ApplicationExt, Command, cosmic_config, cosmic_theme, Element, executor,
    theme, widget,
};
use cosmic::app::{Core, message, Message as CosmicMessage};
use cosmic::iced::{
    Alignment, event, Event, keyboard::Event as KeyEvent, Length, Subscription, window,
};
use cosmic::iced::keyboard::{Key, Modifiers};
use cosmic::widget::{scrollable, segmented_button};
use cosmic::widget::segmented_button::{Entity, EntityMut, SingleSelect};
use done_core::models::list::List;
use done_core::service::Service;

use crate::{content, details, fl, menu, todo};
use crate::config::{AppTheme, CONFIG_VERSION};
use crate::content::Content;
use crate::details::Details;
use crate::key_bind::{key_binds, KeyBind};

pub struct App {
    core: Core,
    nav_model: segmented_button::SingleSelectModel,
    content: Content,
    details: Details,
    config_handler: Option<cosmic_config::Config>,
    config: crate::config::Config,
    app_themes: Vec<String>,
    context_page: ContextPage,
    key_binds: HashMap<KeyBind, Action>,
    modifiers: Modifiers,
    dialog_pages: VecDeque<DialogPage>,
    dialog_text_input: widget::Id,
}

#[derive(Debug, Clone)]
pub enum Message {
    Content(content::Message),
    Details(details::Message),
    ToggleContextPage(ContextPage),
    LaunchUrl(String),
    PopulateLists(Vec<List>),
    WindowClose,
    WindowNew,
    DialogCancel,
    DialogComplete,
    DialogUpdate(DialogPage),
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    AppTheme(AppTheme),
    SystemThemeModeChange(cosmic_theme::ThemeMode),
    OpenNewListDialog,
    OpenRenameListDialog,
    OpenDeleteListDialog,
    OpenIconDialog,
    AddList(List),
    DeleteList,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
    TaskDetails,
    Settings,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::Settings => fl!("settings"),
            Self::TaskDetails => fl!("details"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    New(String),
    Icon(String),
    Rename { to: String },
    Delete,
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: crate::config::Config,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    About,
    ItemDown,
    ItemUp,
    Settings,
    WindowClose,
    WindowNew,
    NewList,
    DeleteList,
    RenameList,
    Icon,
}

impl Action {
    pub fn message(self, _entity_opt: Option<Entity>) -> Message {
        match self {
            Action::About => Message::ToggleContextPage(ContextPage::About),
            Action::ItemDown => Message::Content(content::Message::ItemDown),
            Action::ItemUp => Message::Content(content::Message::ItemUp),
            Action::Settings => Message::ToggleContextPage(ContextPage::Settings),
            Action::WindowClose => Message::WindowClose,
            Action::WindowNew => Message::WindowNew,
            Action::NewList => Message::OpenNewListDialog,
            Action::Icon => Message::OpenIconDialog,
            Action::RenameList => Message::OpenRenameListDialog,
            Action::DeleteList => Message::OpenDeleteListDialog,
        }
    }
}

impl App {
    fn update_config(&mut self) -> Command<CosmicMessage<Message>> {
        app::command::set_theme(self.config.app_theme.theme())
    }

    fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
        let repository = "https://github.com/edfloreshz/cosmic-todo";
        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");
        widget::column::with_children(vec![
            widget::svg(widget::svg::Handle::from_memory(
                &include_bytes!(
                    "../res/icons/hicolor/128x128/apps/com.system76.CosmicTodo.svg"
                )[..],
            ))
                .into(),
            widget::text::title3(fl!("cosmic-todo")).into(),
            widget::button::link(repository)
                .on_press(Message::LaunchUrl(repository.to_string()))
                .padding(0)
                .into(),
            widget::button::link(fl!("git-description", hash = short_hash.as_str(), date = date))
                .on_press(Message::LaunchUrl(format!("{}/commits/{}", repository, hash)))
                .padding(0)
                .into(),
        ])
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .width(Length::Fill)
            .into()
    }

    fn settings(&self) -> Element<Message> {
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };
        widget::settings::view_column(vec![widget::settings::view_section(fl!("appearance"))
            .add(
                widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                    &self.app_themes,
                    Some(app_theme_selected),
                    move |index| {
                        Message::AppTheme(match index {
                            1 => AppTheme::Dark,
                            2 => AppTheme::Light,
                            _ => AppTheme::System,
                        })
                    },
                )),
            )
            .into()])
            .into()
    }

    fn create_nav_item(&mut self, list: List) -> EntityMut<SingleSelect> {
        self.nav_model
            .insert()
            .text(list.name.clone())
            .icon(widget::icon::icon(
                widget::icon::from_name(list.clone().icon.unwrap())
                    .size(16)
                    .handle(),
            ))
            .data(list.clone())
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = "com.system76.CosmicTodo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, flags: Self::Flags) -> (Self, Command<CosmicMessage<Self::Message>>) {
        core.nav_bar_toggle_condensed();
        let nav_model = segmented_button::ModelBuilder::default();

        let app = App {
            core,
            nav_model: nav_model.build(),
            content: Content::new(),
            details: Details::new(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            context_page: ContextPage::Settings,
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
        };

        let commands = vec![Command::perform(
            todo::fetch_lists(),
            |result| match result {
                Ok(data) => message::app(Message::PopulateLists(data)),
                Err(_) => message::none(),
            },
        )];

        (app, Command::batch(commands))
    }

    fn context_drawer(&self) -> Option<Element<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::Settings => self.settings(),
            ContextPage::TaskDetails => self.details.view().map(Message::Details),
        })
    }

    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = match self.dialog_pages.front() {
            Some(some) => some,
            None => return None,
        };

        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let dialog = match dialog_page {
            DialogPage::New(name) => widget::dialog(fl!("create-list"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(self.dialog_text_input.clone())
                            .on_input(move |name| Message::DialogUpdate(DialogPage::New(name)))
                            .into(),
                    ])
                        .spacing(space_xxs),
                ),
            DialogPage::Rename { to: name } => widget::dialog(fl!("rename-list"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(self.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogUpdate(DialogPage::Rename { to: name })
                            })
                            .into(),
                    ])
                        .spacing(space_xxs),
                ),
            DialogPage::Delete => widget::dialog(fl!("delete-list"))
                .body(fl!("delete-list-confirm"))
                .primary_action(
                    widget::button::suggested(fl!("ok"))
                        .on_press_maybe(Some(Message::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                ),
            DialogPage::Icon(icon) => {
                let icons = vec!["address-book-new-symbolic", "application-exit-symbolic", "application-menu-symbolic", "appointment-new-symbolic", "call-start-symbolic", "call-stop-symbolic", "close-menu-symbolic", "contact-new-symbolic", "date-symbolic", "document-new-symbolic", "document-open-recent-symbolic", "document-open-symbolic", "document-page-setup-symbolic", "document-print-preview-symbolic", "document-print-symbolic", "document-properties-symbolic", "document-revert-symbolic", "document-save-as-symbolic", "document-save-symbolic", "document-send-symbolic", "edit-clear-symbolic-rtl", "edit-clear-symbolic", "edit-copy-symbolic", "edit-cut-symbolic", "edit-delete-symbolic", "edit-find-replace-symbolic", "edit-find-symbolic", "edit-paste-symbolic", "edit-redo-symbolic", "edit-select-all-symbolic", "edit-symbolic", "edit-undo-symbolic", "focus-windows-symbolic", "folder-new-symbolic", "format-indent-less-symbolic", "format-indent-more-symbolic", "format-justify-center-symbolic", "format-justify-fill-symbolic", "format-justify-left-symbolic", "format-justify-right-symbolic", "format-text-bold-symbolic", "format-text-direction-ltr-symbolic", "format-text-direction-symbolic-rtl", "format-text-italic-symbolic", "format-text-strikethrough-symbolic", "format-text-underline-symbolic", "go-bottom-symbolic", "go-down-symbolic", "go-first-symbolic", "go-home-symbolic", "go-jump-symbolic", "go-last-symbolic", "go-next-symbolic", "go-previous-symbolic", "go-top-symbolic", "go-up-symbolic", "grip-lines-symbolic", "help-about-symbolic", "help-contents-symbolic", "help-faq-symbolic", "image-red-eye-symbolic", "insert-image-symbolic", "insert-link-symbolic", "insert-object-symbolic", "insert-text-symbolic", "list-add-symbolic", "list-remove-symbolic", "mail-forward-symbolic", "mail-mark-important-symbolic", "mail-mark-junk-symbolic", "mail-mark-notjunk-symbolic", "mail-mark-read-symbolic", "mail-mark-unread-symbolic", "mail-message-new-symbolic", "mail-reply-all-symbolic", "mail-send-receive-symbolic", "mail-send-symbolic", "media-eject-symbolic", "media-playback-pause-symbolic", "media-playback-start-symbolic-rtl", "media-playback-start-symbolic", "media-playback-stop-symbolic", "media-record-symbolic", "media-seek-backward-symbolic", "media-seek-forward-symbolic", "media-skip-backward-symbolic", "media-skip-forward-symbolic", "notification-alert-symbolic", "object-flip-horizontal-symbolic", "object-flip-vertical-symbolic", "object-rotate-left-symbolic", "object-rotate-right-symbolic", "object-select-symbolic", "open-menu-symbolic", "pan-down-symbolic", "pan-end-symbolic", "pan-start-symbolic", "pan-up-symbolic", "process-stop-symbolic", "system-lock-screen-symbolic", "system-log-out-symbolic", "system-reboot-symbolic", "system-run-symbolic", "system-search-symbolic", "system-shutdown-symbolic", "system-suspend-symbolic", "tools-check-spelling-symbolic", "view-fullscreen-symbolic", "view-more-horizontal-symbolic", "view-more-symbolic", "view-refresh-symbolic", "view-restore-symbolic", "view-sort-ascending-symbolic", "view-sort-descending-symbolic", "window-close-symbolic", "window-maximize-symbolic", "window-minimize-symbolic", "window-new-symbolic", "window-restore-symbolic", "window-stack-symbolic", "window-swap-symbolic", "zoom-fit-best-symbolic", "zoom-in-symbolic", "zoom-original-symbolic", "zoom-out-symbolic", "accessories-calculator-symbolic", "accessories-character-map-symbolic", "accessories-clock-symbolic", "accessories-dictionary-symbolic", "accessories-screenshot-symbolic", "accessories-text-editor-symbolic", "addressbook-symbolic", "application-default-symbolic", "application-font-symbolic", "chat-symbolic", "cheese-symbolic", "disks-symbolic", "disk-usage-analyzer-symbolic", "document-viewer-symbolic", "firmware-manager-symbolic", "help-browser-symbolic", "help-contents-symbolic", "help-faq-symbolic", "mail-archive-symbolic", "multimedia-audio-player-symbolic", "multimedia-equalizer-symbolic", "multimedia-photo-manager-symbolic", "multimedia-photo-viewer-symbolic", "multimedia-video-player-symbolic", "multimedia-volume-control-symbolic", "office-calendar-symbolic", "password-manager-symbolic", "preferences-appearance-symbolic", "preferences-desktop-accessibility-symbolic", "preferences-desktop-locale-symbolic", "preferences-desktop-screensaver-symbolic", "preferences-desktop-wallpaper-symbolic", "preferences-dock-symbolic", "preferences-workspaces-symbolic", "setroubleshoot_icon-symbolic", "sparkleshare-symbolic", "system-file-manager-symbolic", "system-software-install-symbolic", "system-users-symbolic", "user-info-symbolic", "utilities-system-monitor-symbolic", "utilities-terminal-symbolic", "utilities-tweak-tool-symbolic", "weather-symbolic", "web-browser-symbolic", "applications-accessibility-symbolic", "applications-engineering-symbolic", "applications-games-symbolic", "applications-graphics-symbolic", "applications-multimedia-symbolic", "applications-science-symbolic", "applications-system-symbolic", "applications-utilities-symbolic", "applications-video-symbolic", "preferences-desktop-online-accounts-symbolic", "preferences-other-symbolic", "preferences-system-symbolic", "slideshow-symbolic", "system-help-symbolic", "ac-adapter-symbolic", "airplane-mode-symbolic", "audio-card-symbolic", "audio-card-usb-symbolic", "audio-headphones-symbolic", "audio-headset-symbolic", "audio-input-microphone-symbolic", "audio-speakers-symbolic", "audio-speaker-symbolic", "audio-speaker-testing-symbolic", "audio-subwoofer-symbolic", "audio-subwoofer-testing-symbolic", "battery-symbolic", "bluetooth-symbolic", "camera-photo-symbolic", "camera-video-symbolic", "camera-web-symbolic", "computer-symbolic", "display-projector-symbolic", "display-symbolic", "drive-harddisk-ieee1394-symbolic", "drive-harddisk-solidstate-symbolic", "drive-harddisk-system-symbolic", "drive-harddisk-usb-symbolic", "drive-multidisk-symbolic", "drive-optical-symbolic", "drive-removable-media-symbolic", "input-dialpad-symbolic", "input-gaming-symbolic", "input-keyboard-symbolic", "input-mouse-symbolic", "input-tablet-symbolic", "input-touchpad-symbolic", "laptop-symbolic", "media-flash-symbolic", "media-floppy-symbolic", "media-removable-symbolic", "media-tape-symbolic", "media-zip-symbolic", "multimedia-player-symbolic", "network-wired-symbolic", "network-wireless-symbolic", "payment-card-symbolic", "phone-symbolic", "printer-network-symbolic", "printer-symbolic", "scanner-symbolic", "sim-card-symbolic", "smartphone-symbolic", "speaker-symbolic", "tablet-symbolic", "tv-symbolic", "uninterruptible-power-supply-symbolic", "video-display-symbolic", "emblem-default-symbolic", "emblem-documents-symbolic", "emblem-favorite-symbolic", "emblem-important-symbolic", "emblem-music-symbolic", "emblem-ok-symbolic", "emblem-photos-symbolic", "emblem-shared-symbolic", "emblem-synchronizing-symbolic", "emblem-system-symbolic", "emblem-videos-symbolic", "application-certificate-symbolic", "application-rss+xml-symbolic", "application-x-addon-symbolic", "application-x-executable-symbolic", "application-x-firmware-symbolic", "audio-x-generic-symbolic", "image-x-generic-symbolic", "inode-directory-symbolic", "media-optical-symbolic", "package-x-generic-symbolic", "playlist-symbolic", "text-html-symbolic", "text-x-generic-symbolic", "video-x-generic-symbolic", "x-office-address-book-symbolic", "x-office-calendar-symbolic", "x-office-document-symbolic", "x-office-drawing-symbolic", "x-office-presentation-symbolic", "x-office-spreadsheet-symbolic", "applications-audio-symbolic", "applications-office-symbolic", "folder-documents-symbolic", "folder-download-symbolic", "folder-music-symbolic", "folder-pictures-symbolic", "folder-publicshare-symbolic", "folder-recent-symbolic", "folder-remote-symbolic", "folder-saved-search-symbolic", "folder-symbolic", "folder-templates-symbolic", "folder-videos-symbolic", "mail-archive-symbolic", "mail-folder-inbox-symbolic", "mail-folder-outbox-symbolic", "network-server-symbolic", "network-workgroup-symbolic", "start-here-symbolic", "user-bookmarks-symbolic", "user-desktop-symbolic", "user-home-symbolic", "user-trash-symbolic", "appointment-missed-symbolic", "appointment-soon-symbolic", "audio-volume-high-symbolic", "audio-volume-low-symbolic", "audio-volume-medium-symbolic", "audio-volume-muted-symbolic", "audio-volume-overamplified-symbolic", "changes-allow-symbolic", "changes-prevent-symbolic", "checkbox-checked-symbolic", "checkbox-mixed-symbolic", "dialog-error-symbolic", "dialog-information-symbolic", "dialog-password-symbolic", "dialog-question-symbolic", "dialog-warning-symbolic", "folder-drag-accept-symbolic", "folder-open-symbolic", "folder-visiting-symbolic", "image-loading-symbolic", "keyboard-brightness-symbolic", "mail-attachment-symbolic", "mail-read-symbolic", "mail-replied-symbolic", "mail-unread-symbolic", "media-playlist-repeat-symbolic-rtl", "media-playlist-repeat-symbolic", "media-playlist-shuffle-symbolic-rtl", "media-playlist-shuffle-symbolic", "microphone-sensitivity-high-symbolic", "microphone-sensitivity-low-symbolic", "microphone-sensitivity-medium-symbolic", "microphone-sensitivity-muted-symbolic", "network-disconnected-symbolic", "network-error-symbolic", "network-idle-symbolic", "network-receive-symbolic", "network-transmit-receive-symbolic", "network-transmit-symbolic", "network-wired-acquiring-symbolic", "network-wired-disconnected-symbolic", "network-wired-error-symbolic", "network-wired-no-route-symbolic", "network-wired-symbolic", "network-wireless-acquiring-symbolic", "network-wireless-connected-symbolic", "network-wireless-disconnected-symbolic", "network-wireless-encrypted-symbolic", "network-wireless-error-symbolic", "network-wireless-hotspot-symbolic", "network-wireless-no-route-symbolic", "network-wireless-signal-excellent-symbolic", "network-wireless-signal-good-symbolic", "network-wireless-signal-none-symbolic", "network-wireless-signal-ok-symbolic", "network-wireless-signal-weak-symbolic", "printer-error-symbolic", "printer-printing-symbolic", "radio-checked-symbolic", "security-high-symbolic", "security-low-symbolic", "security-medium-symbolic", "software-update-available-symbolic", "software-update-urgent-symbolic", "task-due-symbolic", "task-past-due-symbolic", "user-available-symbolic", "user-away-symbolic", "user-idle-symbolic", "user-offline-symbolic", "user-trash-full-symbolic", "weather-clear-night-symbolic", "weather-clear-symbolic", "weather-few-clouds-night-symbolic", "weather-few-clouds-symbolic", "weather-fog-symbolic", "weather-overcast-symbolic", "weather-severe-alert-symbolic", "weather-showers-scattered-symbolic", "weather-showers-symbolic", "weather-snow-symbolic", "weather-storm-symbolic"];
                let icon_buttons: Vec<Element<_>> = icons.into_iter().map(|icon_name| {
                    widget::button::icon(widget::icon::from_name(icon_name).size(32).handle())
                        .on_press(Message::DialogUpdate(DialogPage::Icon(icon_name.to_string())))
                        .into()
                }).collect();
                let mut dialog = widget::dialog(fl!("icon-select"))
                    .body(fl!("icon-select-body"))
                    .primary_action(
                        widget::button::suggested(fl!("ok"))
                            .on_press_maybe(Some(Message::DialogComplete)),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(widget::container(scrollable(widget::flex_row(icon_buttons))).height(Length::Fixed(200.0)));

                if !icon.is_empty() {
                    dialog = dialog.icon(widget::icon::from_name(icon.as_str()).size(32).icon());
                }

                dialog
            }
        };

        Some(dialog.into())
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(&self.key_binds)]
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
        Some(&self.nav_model)
    }

    fn on_escape(&mut self) -> Command<CosmicMessage<Self::Message>> {
        if self.dialog_pages.pop_front().is_some() {
            return Command::none();
        }

        self.core.window.show_context = false;

        Command::none()
    }

    fn on_nav_select(&mut self, entity: Entity) -> Command<CosmicMessage<Self::Message>> {
        self.nav_model.activate(entity);
        let location_opt = self.nav_model.data::<List>(entity);

        if let Some(list) = location_opt {
            let message = Message::Content(content::Message::List(Some(list.clone())));
            return self.update(message);
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let mut subscriptions = vec![
            event::listen_with(|event, status| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => match status {
                    event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            cosmic_config::config_subscription(
                TypeId::of::<ConfigSubscription>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
                .map(|update| {
                    if !update.errors.is_empty() {
                        log::info!(
                        "errors loading config {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                    }
                    Message::SystemThemeModeChange(update.config)
                }),
            cosmic_config::config_subscription::<_, cosmic_theme::ThemeMode>(
                TypeId::of::<ThemeSubscription>(),
                cosmic_theme::THEME_MODE_ID.into(),
                cosmic_theme::ThemeMode::version(),
            )
                .map(|update| {
                    if !update.errors.is_empty() {
                        log::info!(
                        "errors loading theme mode {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                    }
                    Message::SystemThemeModeChange(update.config)
                }),
        ];

        subscriptions.push(self.content.subscription().map(Message::Content));

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Command<CosmicMessage<Self::Message>> {
        // Helper for updating config values efficiently
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!(
                                    "failed to save config {:?}: {}",
                                    stringify!($name),
                                    err
                                );
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        log::warn!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name)
                        );
                    }
                }
            };
        }

        let mut commands = vec![];

        match message {
            Message::Content(message) => {
                let content_commands = self.content.update(message);
                for content_command in content_commands {
                    match content_command {
                        content::Command::GetTasks(list_id) => {
                            commands.push(Command::perform(todo::fetch_tasks(list_id), |result| {
                                match result {
                                    Ok(data) => message::app(Message::Content(
                                        content::Message::SetItems(data),
                                    )),
                                    Err(_) => message::none(),
                                }
                            }));
                        }
                        content::Command::DisplayTask(task) => {
                            let entity =
                                self.details.priority_model.entity_at(task.priority as u16);
                            if let Some(entity) = entity {
                                self.details.priority_model.activate(entity);
                            }

                            self.details.task = Some(task.clone());
                            commands.push(
                                self.update(Message::ToggleContextPage(ContextPage::TaskDetails)),
                            );
                        }
                        content::Command::UpdateTask(task) => {
                            self.details.task = Some(task.clone());
                            let command =
                                Command::perform(todo::update_task(task), |result| match result {
                                    Ok(_) => message::none(),
                                    Err(_) => message::none(),
                                });
                            commands.push(command);
                        }
                        content::Command::Delete(id) => {
                            if let Some(list) = self.nav_model.data::<List>(self.nav_model.active())
                            {
                                let command = Command::perform(
                                    todo::delete_task(list.id.clone(), id.clone()),
                                    |result| match result {
                                        Ok(_) => message::none(),
                                        Err(_) => message::none(),
                                    },
                                );
                                commands.push(command);
                            }
                        }
                        content::Command::CreateTask(task) => {
                            let command =
                                Command::perform(todo::create_task(task), |result| match result {
                                    Ok(_) => message::none(),
                                    Err(_) => message::none(),
                                });
                            commands.push(command);
                        }
                    }
                }
            }
            Message::Details(message) => {
                let details_commands = self.details.update(message);
                for details_command in details_commands {
                    match details_command {
                        details::Command::Update(task) => {
                            commands.push(Command::perform(todo::update_task(task), |result| {
                                match result {
                                    Ok(_) => message::none(),
                                    Err(_) => message::none(),
                                }
                            }));
                        }
                        details::Command::Rename(id, title) => {
                            commands.push(self.update(Message::Content(content::Message::Rename(
                                id.clone(),
                                title.clone(),
                            ))));
                        }
                        details::Command::Favorite(id, favorite) => {
                            commands.push(self.update(Message::Content(
                                content::Message::Favorite(id.clone(), favorite),
                            )));
                        }
                        details::Command::PriorityActivate(id, priority) => {
                            commands.push(self.update(Message::Content(
                                content::Message::SetPriority(id.clone(), priority),
                            )));
                        }
                    }
                }
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page.clone();
                    self.core.window.show_context = true;
                }
                self.set_context_title(context_page.clone().title());
            }
            Message::WindowClose => {
                return window::close(window::Id::MAIN);
            }
            Message::WindowNew => match env::current_exe() {
                Ok(exe) => match process::Command::new(&exe).spawn() {
                    Ok(_child) => {}
                    Err(err) => {
                        eprintln!("failed to execute {:?}: {}", exe, err);
                    }
                },
                Err(err) => {
                    eprintln!("failed to get current executable path: {}", err);
                }
            },
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    log::warn!("failed to open {:?}: {}", url, err);
                }
            },
            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }
            Message::SystemThemeModeChange(_) => {
                return self.update_config();
            }
            Message::PopulateLists(lists) => {
                for list in lists {
                    self.create_nav_item(list);
                }
                let Some(entity) = self.nav_model.iter().next() else {
                    return Command::none();
                };
                self.nav_model.activate(entity);
                let command = self.on_nav_select(entity);
                commands.push(command);
            }
            Message::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message(None));
                    }
                }
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::AddList(list) => {
                self.create_nav_item(list);
                let Some(entity) = self.nav_model.iter().last() else {
                    return Command::none();
                };
                let command = self.on_nav_select(entity);
                commands.push(command);
            }
            Message::DeleteList => {
                if let Some(list) = self.nav_model.data::<List>(self.nav_model.active()) {
                    let command =
                        Command::perform(
                            todo::delete_list(list.id.clone()),
                            |result| match result {
                                Ok(_) => message::none(),
                                Err(_) => message::none(),
                            },
                        );

                    commands.push(self.update(Message::Content(content::Message::List(None))));

                    commands.push(command);
                }
                self.nav_model.remove(self.nav_model.active());
            }
            Message::OpenNewListDialog => {
                self.dialog_pages.push_back(DialogPage::New(String::new()));
                return widget::text_input::focus(self.dialog_text_input.clone());
            }
            Message::OpenRenameListDialog => {
                if let Some(list) = self.nav_model.data::<List>(self.nav_model.active()) {
                    self.dialog_pages.push_back(DialogPage::Rename {
                        to: list.name.clone(),
                    });
                    return widget::text_input::focus(self.dialog_text_input.clone());
                }
            }
            Message::OpenDeleteListDialog => {
                if self
                    .nav_model
                    .data::<List>(self.nav_model.active())
                    .is_some()
                {
                    self.dialog_pages.push_back(DialogPage::Delete);
                }
            }
            Message::OpenIconDialog => {
                if self
                    .nav_model
                    .data::<List>(self.nav_model.active())
                    .is_some()
                {
                    self.dialog_pages.push_back(DialogPage::Icon(String::new()));
                }
            }
            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }
            Message::DialogComplete => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::New(name) => {
                            let list = List::new(&name, Service::Computer);
                            commands.push(Command::perform(todo::create_list(list), |result| {
                                match result {
                                    Ok(list) => message::app(Message::AddList(list)),
                                    Err(_) => message::none(),
                                }
                            }));
                        }
                        DialogPage::Rename { to: name } => {
                            let entity = self.nav_model.active();
                            self.nav_model.text_set(entity, name.clone());
                            if let Some(list) = self.nav_model.active_data_mut::<List>() {
                                list.name = name.clone();
                                let command =
                                    Command::perform(todo::update_list(list.clone()), |_| {
                                        message::none()
                                    });
                                commands.push(command);
                            }
                        }
                        DialogPage::Delete => {
                            commands.push(self.update(Message::DeleteList));
                        }
                        DialogPage::Icon(icon) => {
                            let entity = self.nav_model.active();
                            self.nav_model.icon_set(entity, widget::icon::from_name(icon.clone()).size(16).icon());
                            if let Some(list) = self.nav_model.active_data_mut::<List>() {
                                list.icon = Some(icon);
                                let command =
                                    Command::perform(todo::update_list(list.clone()), |_| {
                                        message::none()
                                    });
                                commands.push(command);
                            }
                        }
                    }
                }
            }
            Message::DialogUpdate(dialog_page) => {
                //TODO: panicless way to do this?
                self.dialog_pages[0] = dialog_page;
            }
        }

        if !commands.is_empty() {
            return Command::batch(commands);
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let content_view = self.content.view().map(Message::Content);

        widget::column::with_children(vec![content_view]).into()
    }
}
