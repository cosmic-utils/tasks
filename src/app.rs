use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::{env, process};

use chrono::{Local, NaiveDate};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use cosmic::app::{message, Core, Message as CosmicMessage};
use cosmic::cosmic_config::Update;
use cosmic::cosmic_theme::ThemeMode;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::keyboard::{Key, Modifiers};
use cosmic::iced::{
    event, keyboard::Event as KeyEvent, window, Alignment, Event, Length, Subscription,
};
use cosmic::widget::menu::action::MenuAction;
use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::segmented_button::{Entity, EntityMut, SingleSelect};
use cosmic::widget::{horizontal_space, scrollable, segmented_button};
use cosmic::{
    app, cosmic_config, cosmic_theme, executor, theme, widget, Application, ApplicationExt, Element,
};
use tasks_core::models::List;
use tasks_core::models::Task;
use tasks_core::service::{Provider, TaskService};

use crate::app::config::{AppTheme, CONFIG_VERSION};
use crate::app::key_bind::key_binds;
use crate::content::Content;
use crate::details::Details;
use crate::{content, details, fl, todo};

pub mod config;
pub mod icon_cache;
mod key_bind;
pub mod localize;
pub mod markdown;
pub mod menu;
pub mod settings;

pub struct Tasks {
    core: Core,
    service: TaskService,
    nav_model: segmented_button::SingleSelectModel,
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
    Tasks(TasksAction),
    Content(content::Message),
    Details(details::Message),
    Dialog(DialogAction),
    ToggleContextPage(ContextPage),
    Application(ApplicationAction),
}

#[derive(Debug, Clone)]
pub enum DialogAction {
    Open(DialogPage),
    Update(DialogPage),
    Close,
    Complete,
}

#[derive(Debug, Clone)]
pub enum TasksAction {
    PopulateLists(Vec<List>),
    Export(Vec<Task>),
    AddList(List),
    DeleteList,
    FetchLists,
}

#[derive(Debug, Clone)]
pub enum ApplicationAction {
    LaunchUrl(String),
    WindowClose,
    WindowNew,
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    AppTheme(usize),
    SystemThemeModeChange,
    Focus(widget::Id),
    NavMenuAction(NavMenuAction),
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
    Rename(String),
    Delete,
    Calendar(NaiveDate),
    Export(String),
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: config::TasksConfig,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    About,
    Settings,
    WindowClose,
    WindowNew,
    NewList,
    DeleteList,
    RenameList,
    Icon,
}

impl MenuAction for Action {
    type Message = Message;
    fn message(&self) -> Self::Message {
        match self {
            Action::About => Message::ToggleContextPage(ContextPage::About),
            Action::Settings => Message::ToggleContextPage(ContextPage::Settings),
            Action::WindowClose => Message::Application(ApplicationAction::WindowClose),
            Action::WindowNew => Message::Application(ApplicationAction::WindowNew),
            Action::NewList => Message::Dialog(DialogAction::Open(DialogPage::New(String::new()))),
            Action::Icon => Message::Dialog(DialogAction::Open(DialogPage::Icon(String::new()))),
            Action::RenameList => {
                Message::Dialog(DialogAction::Open(DialogPage::Rename(String::new())))
            }
            Action::DeleteList => Message::Dialog(DialogAction::Open(DialogPage::Delete)),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NavMenuAction {
    Rename(segmented_button::Entity),
    SetIcon(segmented_button::Entity),
    Delete(segmented_button::Entity),
}

impl MenuAction for NavMenuAction {
    type Message = cosmic::app::Message<Message>;

    fn message(&self) -> Self::Message {
        cosmic::app::Message::App(Message::Application(ApplicationAction::NavMenuAction(
            *self,
        )))
    }
}

impl Tasks {
    fn update_config(&mut self) -> app::Task<Message> {
        app::command::set_theme(self.config.app_theme.theme())
    }

    fn about(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;
        let repository = "https://github.com/cosmic-utils/tasks";
        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");
        widget::column::with_children(vec![
            widget::svg(widget::svg::Handle::from_memory(
                &include_bytes!("../res/icons/hicolor/scalable/apps/dev.edfloreshz.Tasks.svg")[..],
            ))
            .into(),
            widget::text::title3(fl!("tasks")).into(),
            widget::button::link(repository)
                .on_press(Message::Application(ApplicationAction::LaunchUrl(
                    repository.to_string(),
                )))
                .padding(spacing.space_none)
                .into(),
            widget::button::link(fl!(
                "git-description",
                hash = short_hash.as_str(),
                date = date
            ))
            .on_press(Message::Application(ApplicationAction::LaunchUrl(format!(
                "{repository}/commits/{hash}"
            ))))
            .padding(spacing.space_none)
            .into(),
        ])
        .align_x(Alignment::Center)
        .spacing(spacing.space_xxs)
        .width(Length::Fill)
        .into()
    }

    fn settings(&self) -> Element<Message> {
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };
        widget::settings::view_column(vec![widget::settings::section()
            .title(fl!("appearance"))
            .add(
                widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                    &self.app_themes,
                    Some(app_theme_selected),
                    |index| Message::Application(ApplicationAction::AppTheme(index)),
                )),
            )
            .into()])
        .into()
    }

    fn create_nav_item(&mut self, list: &List) -> EntityMut<SingleSelect> {
        self.nav_model
            .insert()
            .text(format!(
                "{} {}",
                list.icon
                    .clone()
                    .unwrap_or(emojis::get_by_shortcode("pencil").unwrap().to_string()),
                list.name.clone()
            ))
            .data(list.clone())
    }
}

impl Application for Tasks {
    type Executor = executor::Default;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = "dev.edfloreshz.Tasks";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, app::Task<Self::Message>) {
        let nav_model = segmented_button::ModelBuilder::default().build();
        let service = TaskService::new(Self::APP_ID, Provider::Computer);
        let mut app = Tasks {
            core,
            service: service.clone(),
            nav_model,
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

        app.core.nav_bar_toggle_condensed();

        let mut tasks = vec![app::Task::perform(
            TaskService::migrate(Self::APP_ID),
            |_| message::app(Message::Tasks(TasksAction::FetchLists)),
        )];

        if let Some(id) = app.core.main_window_id() {
            tasks.push(app.set_window_title(fl!("tasks"), id));
        }

        (app, app::Task::batch(tasks))
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

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(&self.key_binds)]
    }

    fn nav_context_menu(
        &self,
        id: widget::nav_bar::Id,
    ) -> Option<Vec<widget::menu::Tree<CosmicMessage<Self::Message>>>> {
        Some(cosmic::widget::menu::items(
            &HashMap::new(),
            vec![
                cosmic::widget::menu::Item::Button(fl!("rename"), NavMenuAction::Rename(id)),
                cosmic::widget::menu::Item::Button(fl!("icon"), NavMenuAction::SetIcon(id)),
                cosmic::widget::menu::Item::Button(fl!("delete"), NavMenuAction::Delete(id)),
            ],
        ))
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
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
            let message = Message::Content(content::Message::List(Some(list.clone())));
            let window_title = format!("{} - {}", list.name, fl!("tasks"));
            if let Some(window_id) = self.core.main_window_id() {
                tasks.push(self.set_window_title(window_title, window_id));
            }
            return self.update(message);
        }

        app::Task::batch(tasks)
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Self::Message> {
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

        let mut tasks = vec![];

        match message {
            Message::Content(message) => {
                let content_tasks = self.content.update(message);
                for content_task in content_tasks {
                    match content_task {
                        content::Task::Iced(task) => return task,
                        content::Task::GetTasks(list_id) => {
                            tasks.push(app::Task::perform(
                                todo::fetch_tasks(list_id, self.service.clone()),
                                |result| match result {
                                    Ok(data) => message::app(Message::Content(
                                        content::Message::SetItems(data),
                                    )),
                                    Err(_) => message::none(),
                                },
                            ));
                        }
                        content::Task::DisplayTask(task) => {
                            let entity =
                                self.details.priority_model.entity_at(task.priority as u16);
                            if let Some(entity) = entity {
                                self.details.priority_model.activate(entity);
                            }
                            self.details.subtasks.clear();
                            self.details.sub_task_input_ids.clear();
                            self.details.task = Some(task.clone());
                            task.sub_tasks.into_iter().for_each(|task| {
                                let id = self.details.subtasks.insert(task);
                                self.details
                                    .sub_task_input_ids
                                    .insert(id, widget::Id::unique());
                            });
                            tasks.push(
                                self.update(Message::ToggleContextPage(ContextPage::TaskDetails)),
                            );
                        }
                        content::Task::UpdateTask(task) => {
                            self.details.task = Some(task.clone());
                            let task = app::Task::perform(
                                todo::update_task(task, self.service.clone().clone()),
                                |result| match result {
                                    Ok(()) | Err(_) => message::none(),
                                },
                            );
                            tasks.push(task);
                        }
                        content::Task::Delete(id) => {
                            if let Some(list) = self.nav_model.active_data::<List>() {
                                let task = app::Task::perform(
                                    todo::delete_task(
                                        list.id().clone(),
                                        id.clone(),
                                        self.service.clone().clone(),
                                    ),
                                    |result| match result {
                                        Ok(()) | Err(_) => message::none(),
                                    },
                                );
                                tasks.push(task);
                            }
                        }
                        content::Task::CreateTask(task) => {
                            let task = app::Task::perform(
                                todo::create_task(task, self.service.clone()),
                                |result| match result {
                                    Ok(()) | Err(_) => message::none(),
                                },
                            );
                            tasks.push(task);
                        }
                        content::Task::Export(exported_tasks) => {
                            tasks.push(
                                self.update(Message::Tasks(TasksAction::Export(exported_tasks))),
                            );
                        }
                    }
                }
            }
            Message::Details(message) => {
                let details_tasks = self.details.update(message);
                for details_task in details_tasks {
                    match details_task {
                        details::Task::UpdateTask(task) => {
                            tasks.push(self.update(Message::Content(
                                content::Message::UpdateTask(task.clone()),
                            )));
                        }
                        details::Task::OpenCalendarDialog => {
                            tasks.push(self.update(Message::Dialog(DialogAction::Open(
                                DialogPage::Calendar(Local::now().date_naive()),
                            ))));
                        }
                        details::Task::Focus(id) => {
                            tasks.push(
                                self.update(Message::Application(ApplicationAction::Focus(id))),
                            );
                        }
                        details::Task::Iced(task) => return task,
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
            Message::Dialog(dialog_action) => match dialog_action {
                DialogAction::Open(page) => {
                    match page {
                        DialogPage::Icon(icon) => {
                            if self.nav_model.active_data::<List>().is_some() {
                                self.dialog_pages.push_back(DialogPage::Icon(icon));
                            }
                        }
                        DialogPage::Rename(_) => {
                            if let Some(list) = self.nav_model.active_data::<List>() {
                                self.dialog_pages
                                    .push_back(DialogPage::Rename(list.name.clone()));
                            }
                        }
                        DialogPage::Delete => {
                            if self.nav_model.active_data::<List>().is_some() {
                                self.dialog_pages.push_back(DialogPage::Delete);
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
                                tasks.push(app::Task::perform(
                                    todo::create_list(list, self.service.clone()),
                                    |result| match result {
                                        Ok(list) => {
                                            message::app(Message::Tasks(TasksAction::AddList(list)))
                                        }
                                        Err(_) => message::none(),
                                    },
                                ));
                            }
                            DialogPage::Rename(name) => {
                                let entity = self.nav_model.active();
                                self.nav_model.text_set(entity, name.clone());
                                if let Some(list) = self.nav_model.active_data_mut::<List>() {
                                    list.name.clone_from(&name);
                                    let task = app::Task::perform(
                                        todo::update_list(list.clone(), self.service.clone()),
                                        |_| message::none(),
                                    );
                                    tasks.push(task);
                                }
                            }
                            DialogPage::Delete => {
                                tasks.push(self.update(Message::Tasks(TasksAction::DeleteList)));
                            }
                            DialogPage::Icon(icon) => {
                                if let Some(list) = self.nav_model.active_data::<List>() {
                                    let entity = self.nav_model.active();
                                    let title = format!("{} {}", icon.clone(), list.name.clone());
                                    self.nav_model.text_set(entity, title);
                                }
                                if let Some(list) = self.nav_model.active_data_mut::<List>() {
                                    list.icon = Some(icon);
                                    let task = app::Task::perform(
                                        todo::update_list(list.clone(), self.service.clone()),
                                        |_| message::none(),
                                    );
                                    tasks.push(task);
                                }
                            }
                            DialogPage::Calendar(date) => {
                                self.details.update(details::Message::SetDueDate(date));
                            }
                            DialogPage::Export(content) => {
                                let mut clipboard = ClipboardContext::new().unwrap();
                                clipboard.set_contents(content).unwrap();
                            }
                        }
                    }
                }
            },
            Message::Tasks(tasks_action) => match tasks_action {
                TasksAction::FetchLists => {
                    tasks.push(app::Task::perform(
                        todo::fetch_lists(self.service.clone()),
                        |result| match result {
                            Ok(data) => {
                                message::app(Message::Tasks(TasksAction::PopulateLists(data)))
                            }
                            Err(_) => message::none(),
                        },
                    ));
                }
                TasksAction::PopulateLists(lists) => {
                    for list in lists {
                        self.create_nav_item(&list);
                    }
                    let Some(entity) = self.nav_model.iter().next() else {
                        return app::Task::none();
                    };
                    self.nav_model.activate(entity);
                    let task = self.on_nav_select(entity);
                    tasks.push(task);
                }
                TasksAction::AddList(list) => {
                    self.create_nav_item(&list);
                    let Some(entity) = self.nav_model.iter().last() else {
                        return app::Task::none();
                    };
                    let task = self.on_nav_select(entity);
                    tasks.push(task);
                }
                TasksAction::DeleteList => {
                    if let Some(list) = self.nav_model.active_data::<List>() {
                        let task = app::Task::perform(
                            todo::delete_list(list.id().clone(), self.service.clone()),
                            |result| match result {
                                Ok(()) | Err(_) => message::none(),
                            },
                        );

                        tasks.push(self.update(Message::Content(content::Message::List(None))));

                        tasks.push(task);
                    }
                    self.nav_model.remove(self.nav_model.active());
                }
                TasksAction::Export(exported_tasks) => {
                    if let Some(list) = self.nav_model.active_data() {
                        let exported_markdown = todo::export_list(list, &exported_tasks);
                        tasks.push(self.update(Message::Dialog(DialogAction::Open(
                            DialogPage::Export(exported_markdown),
                        ))));
                    }
                }
            },
            Message::Application(application_action) => {
                match application_action {
                    ApplicationAction::WindowClose => {
                        if let Some(window_id) = self.core.main_window_id() {
                            return window::close(window_id);
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
                    ApplicationAction::LaunchUrl(url) => match open::that_detached(&url) {
                        Ok(()) => {}
                        Err(err) => {
                            log::warn!("failed to open {:?}: {}", url, err);
                        }
                    },
                    ApplicationAction::AppTheme(index) => {
                        let app_theme = match index {
                            1 => AppTheme::Dark,
                            2 => AppTheme::Light,
                            _ => AppTheme::System,
                        };
                        config_set!(app_theme, app_theme);
                        return self.update_config();
                    }
                    ApplicationAction::SystemThemeModeChange => {
                        return self.update_config();
                    }
                    ApplicationAction::Key(modifiers, key) => {
                        for (key_bind, action) in &self.key_binds {
                            if key_bind.matches(modifiers, &key) {
                                return self.update(action.message());
                            }
                        }
                    }
                    ApplicationAction::Modifiers(modifiers) => {
                        self.modifiers = modifiers;
                    }
                    ApplicationAction::Focus(id) => tasks.push(widget::text_input::focus(id)),
                    ApplicationAction::NavMenuAction(nav_menu_action) => match nav_menu_action {
                        NavMenuAction::Rename(entity) => {
                            if self.nav_model.data::<List>(entity).is_some() {
                                tasks.push(self.update(Message::Dialog(DialogAction::Open(
                                    DialogPage::Rename(String::new()),
                                ))));
                            }
                        }
                        NavMenuAction::SetIcon(entity) => {
                            if self.nav_model.data::<List>(entity).is_some() {
                                tasks.push(self.update(Message::Dialog(DialogAction::Open(
                                    DialogPage::Icon(String::new()),
                                ))));
                            }
                        }
                        NavMenuAction::Delete(entity) => {
                            if self.nav_model.data::<List>(entity).is_some() {
                                tasks.push(self.update(Message::Dialog(DialogAction::Open(
                                    DialogPage::Delete,
                                ))));
                            }
                        }
                    },
                }
            }
        }

        app::Task::batch(tasks)
    }

    fn view(&self) -> Element<Self::Message> {
        let content_view = self.content.view().map(Message::Content);
        content_view
    }

    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = self.dialog_pages.front()?;

        let spacing = theme::active().cosmic().spacing;

        let dialog = match dialog_page {
            DialogPage::New(name) => widget::dialog(fl!("create-list"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::Dialog(DialogAction::Close)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(self.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::Dialog(DialogAction::Update(DialogPage::New(name)))
                            })
                            .on_submit(Message::Dialog(DialogAction::Complete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::Rename(name) => widget::dialog(fl!("rename-list"))
                .primary_action(
                    widget::button::suggested(fl!("save"))
                        .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::Dialog(DialogAction::Close)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(self.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::Dialog(DialogAction::Update(DialogPage::Rename(name)))
                            })
                            .on_submit(Message::Dialog(DialogAction::Complete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::Delete => widget::dialog(fl!("delete-list"))
                .body(fl!("delete-list-confirm"))
                .primary_action(
                    widget::button::suggested(fl!("ok"))
                        .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::Dialog(DialogAction::Close)),
                ),
            DialogPage::Icon(icon) => {
                let icon_buttons: Vec<Element<_>> = emojis::iter()
                    .map(|emoji| {
                        widget::button::custom(
                            widget::container(widget::text(emoji.to_string()))
                                .width(spacing.space_l)
                                .height(spacing.space_l)
                                .align_y(Vertical::Center)
                                .align_x(Horizontal::Center),
                        )
                        .on_press(Message::Dialog(DialogAction::Update(DialogPage::Icon(
                            emoji.to_string(),
                        ))))
                        .into()
                    })
                    .collect();
                let mut dialog = widget::dialog(fl!("icon-select"))
                    .body(fl!("icon-select-body"))
                    .primary_action(
                        widget::button::suggested(fl!("ok"))
                            .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(Message::Dialog(DialogAction::Close)),
                    )
                    .control(
                        widget::container(scrollable(widget::row::with_children(vec![
                            widget::flex_row(icon_buttons).into(),
                            horizontal_space().into(),
                        ])))
                        .height(Length::Fixed(300.0)),
                    );

                if !icon.is_empty() {
                    dialog = dialog.icon(widget::container(
                        widget::text(icon.as_str()).size(spacing.space_l),
                    ));
                }

                dialog
            }
            DialogPage::Calendar(date) => {
                let dialog = widget::dialog(fl!("select-date"))
                    .primary_action(
                        widget::button::suggested(fl!("ok"))
                            .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(Message::Dialog(DialogAction::Close)),
                    )
                    .control(
                        widget::container(widget::calendar(date, |date| {
                            Message::Dialog(DialogAction::Update(DialogPage::Calendar(date)))
                        }))
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                    );
                dialog
            }
            DialogPage::Export(contents) => {
                let dialog = widget::dialog(fl!("export"))
                    .control(
                        widget::container(scrollable(widget::text(contents)).width(Length::Fill))
                            .height(Length::Fixed(200.0))
                            .width(Length::Fill),
                    )
                    .primary_action(
                        widget::button::suggested(fl!("copy"))
                            .on_press_maybe(Some(Message::Dialog(DialogAction::Complete))),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(Message::Dialog(DialogAction::Close)),
                    );

                dialog
            }
        };

        Some(dialog.into())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let mut subscriptions = vec![
            event::listen_with(|event, _status, _window_id| match event {
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
                    log::info!(
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
                    log::info!(
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
