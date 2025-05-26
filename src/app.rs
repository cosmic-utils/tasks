use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
    env, process,
};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use cosmic::{
    app::{self, Core},
    cosmic_config::{self, Update},
    cosmic_theme::{self, ThemeMode},
    iced::{
        keyboard::{Event as KeyEvent, Modifiers},
        Event, Subscription,
    },
    widget::{
        self,
        calendar::CalendarModel,
        menu::{key_bind::KeyBind, Action as _},
        segmented_button::{Entity, EntityMut, SingleSelect},
    },
    Application, ApplicationExt, Element,
};

use crate::{
    actions::{Action, ApplicationAction, NavMenuAction, TasksAction},
    app::{config::CONFIG_VERSION, key_bind::key_binds},
    content::{self, Content},
    context::ContextPage,
    core::{models::List, storage::LocalStorage},
    details::{self, Details},
    dialog::{DialogAction, DialogPage},
    fl,
};

pub mod config;
pub mod icons;
mod key_bind;
pub mod localize;
pub mod markdown;
pub mod menu;
pub mod settings;
pub mod style;

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
    fn settings(&self) -> Element<Message> {
        widget::scrollable(widget::settings::section().title(fl!("appearance")).add(
            widget::settings::item::item(
                fl!("theme"),
                widget::dropdown(
                    &self.app_themes,
                    Some(self.config.app_theme.into()),
                    |theme| Message::Application(ApplicationAction::AppTheme(theme)),
                ),
            ),
        ))
        .into()
    }

    fn create_nav_item(&mut self, list: &List) -> EntityMut<SingleSelect> {
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
            }
        }
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
                            let data = if let Some(entity) = entity {
                                self.nav_model.data_mut::<List>(entity)
                            } else {
                                self.nav_model.active_data_mut::<List>()
                            };
                            if let Some(list) = data {
                                list.name.clone_from(&name.clone());
                                let list = list.clone();
                                self.nav_model
                                    .text_set(self.nav_model.active(), name.clone());
                                if let Err(err) = self.storage.update_list(&list) {
                                    tracing::error!("Error updating list: {err}");
                                }
                                tasks.push(self.update(Message::Content(
                                    content::Message::SetList(Some(list)),
                                )));
                            }
                        }
                        DialogPage::Delete(entity) => {
                            tasks
                                .push(self.update(Message::Tasks(TasksAction::DeleteList(entity))));
                        }
                        DialogPage::Icon(entity, name, _) => {
                            let data = if let Some(entity) = entity {
                                self.nav_model.data::<List>(entity)
                            } else {
                                self.nav_model.active_data::<List>()
                            };
                            if let Some(list) = data {
                                let entity = self.nav_model.active();
                                self.nav_model.text_set(entity, list.name.clone());
                                self.nav_model
                                    .icon_set(entity, crate::app::icons::get_icon(&name, 16));
                            }
                            if let Some(list) = self.nav_model.active_data_mut::<List>() {
                                list.icon = Some(name);
                                let list = list.clone();
                                if let Err(err) = self.storage.update_list(&list) {
                                    tracing::error!("Error updating list: {err}");
                                }
                                tasks.push(self.update(Message::Content(
                                    content::Message::SetList(Some(list)),
                                )));
                            }
                        }
                        DialogPage::Calendar(date) => {
                            self.details
                                .update(details::Message::SetDueDate(date.selected));
                        }
                        DialogPage::Export(content) => {
                            let mut clipboard = ClipboardContext::new().unwrap();
                            clipboard.set_contents(content).unwrap();
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
                                tasks.push(self.update(Message::Application(
                                    ApplicationAction::Dialog(DialogAction::Open(
                                        DialogPage::Export(exported_markdown),
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
                for list in lists {
                    self.create_nav_item(&list);
                }
                let Some(entity) = self.nav_model.iter().next() else {
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
    type Flags = crate::settings::app::Flags;
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
            .icon(Self::APP_ID)
            .version("0.2.0")
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
        };

        let mut tasks = vec![app.update(Message::Tasks(TasksAction::FetchLists))];

        if let Some(id) = app.core.main_window_id() {
            tasks.push(app.set_window_title(fl!("tasks"), id));
        }

        app.core.nav_bar_toggle_condensed();

        (app, app::Task::batch(tasks))
    }

    fn context_drawer(&self) -> Option<app::context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => app::context_drawer::about(
                &self.about,
                Message::Open,
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

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(&self.key_binds, &self.config)]
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

    fn view(&self) -> Element<Self::Message> {
        self.content.view().map(Message::Content)
    }

    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = self.dialog_pages.front()?;
        let dialog = dialog_page.view(&self.dialog_text_input);
        Some(dialog.into())
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

        Subscription::batch(subscriptions)
    }
}
