use chrono::{NaiveDate, NaiveTime, NaiveDateTime, Timelike};
use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Length,
    },
    widget::{self, calendar::CalendarModel, segmented_button},
};

use crate::{app::actions::ApplicationAction, app::Message, fl};

/// Holds both date and time information for dialog inputs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeInfo {
    pub calendar: CalendarModel,
    pub time: NaiveTime,
}

impl DateTimeInfo {
    /// Create a new DateTimeInfo with the given date and current time
    pub fn new(date: NaiveDate) -> Self {
        let now = chrono::Utc::now().time();
        Self {
            calendar: CalendarModel::new(date, date),
            time: now,
        }
    }
    
    /// Create a new DateTimeInfo with the given date and time
    pub fn with_time(date: NaiveDate, time: NaiveTime) -> Self {
        Self {
            calendar: CalendarModel::new(date, date),
            time,
        }
    }
    
    /// Get the selected date from the calendar
    pub fn selected_date(&self) -> NaiveDate {
        self.calendar.selected
    }
    
    /// Combine date and time into a NaiveDateTime
    pub fn to_naive_datetime(&self) -> NaiveDateTime {
        NaiveDateTime::new(self.selected_date(), self.time)
    }
}

#[derive(Debug, Clone)]
pub enum DialogAction {
    Open(DialogPage),
    Update(DialogPage),
    Close,
    Complete,
    #[allow(dead_code)]
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    New(String),
    Rename(Option<segmented_button::Entity>, String),
    Delete(Option<segmented_button::Entity>),
    Calendar(DateTimeInfo),
    ReminderCalendar(DateTimeInfo),
    Export(String),
}

impl DialogPage {
    pub fn view(&self, text_input_id: &widget::Id) -> widget::Dialog<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        match self {
            DialogPage::New(name) => widget::dialog()
                .title(fl!("create-list"))
                .primary_action(widget::button::suggested(fl!("save")).on_press_maybe(Some(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                )))
                .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                ))
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(text_input_id.clone())
                            .on_input(move |name| {
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Update(DialogPage::New(name)),
                                ))
                            })
                            .on_submit(|_| {
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Complete,
                                ))
                            })
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::Rename(entity, name) => widget::dialog()
                .title(fl!("rename-list"))
                .primary_action(widget::button::suggested(fl!("save")).on_press_maybe(Some(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                )))
                .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                ))
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("list-name")).into(),
                        widget::text_input("", name.as_str())
                            .id(text_input_id.clone())
                            .on_input(move |name| {
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Update(DialogPage::Rename(*entity, name)),
                                ))
                            })
                            .on_submit(|_| {
                                Message::Application(ApplicationAction::Dialog(
                                    DialogAction::Complete,
                                ))
                            })
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::Delete(_) => widget::dialog()
                .title(fl!("delete-list"))
                .body(fl!("delete-list-confirm"))
                .primary_action(widget::button::suggested(fl!("ok")).on_press_maybe(Some(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                )))
                .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                    Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                )),
            DialogPage::Calendar(date_time_info) => {
                let date_time_clone_prev = date_time_info.clone();
                let date_time_clone_next = date_time_info.clone();
                let date_time_clone_select = date_time_info.clone();
                let dialog = widget::dialog()
                    .title(fl!("select-date"))
                    .primary_action(widget::button::suggested(fl!("ok")).on_press_maybe(Some(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                    )))
                    .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                    ))
                    .control(
                        widget::column::with_children(vec![
                            widget::container(widget::calendar(
                                &date_time_info.calendar,
                                move |selected_date| {
                                    Message::Application(ApplicationAction::Dialog(
                                        DialogAction::Update(DialogPage::Calendar(DateTimeInfo::with_time(
                                            selected_date,
                                            date_time_clone_select.time,
                                        ))),
                                    ))
                                },
                                move || {
                                    // Previous month - update visible month, keep selected date and time
                                    let mut new_info = date_time_clone_prev.clone();
                                    new_info.calendar.show_prev_month();
                                    Message::Application(ApplicationAction::Dialog(
                                        DialogAction::Update(DialogPage::Calendar(new_info))
                                    ))
                                },
                                move || {
                                    // Next month - update visible month, keep selected date and time
                                    let mut new_info = date_time_clone_next.clone();
                                    new_info.calendar.show_next_month();
                                    Message::Application(ApplicationAction::Dialog(
                                        DialogAction::Update(DialogPage::Calendar(new_info))
                                    ))
                                },
                                chrono::Weekday::Mon,
                            ))
                            .width(Length::Fill)
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center)
                            .into(),
                        ])
                        .spacing(spacing.space_s),
                    );
                dialog
            }
            DialogPage::ReminderCalendar(date_time_info) => {
                let date_time_clone_prev = date_time_info.clone();
                let date_time_clone_next = date_time_info.clone();
                let date_time_clone_select = date_time_info.clone();
                let date_time_clone_hour = date_time_info.clone();
                let date_time_clone_minute = date_time_info.clone();
                let hour_str = format!("{:02}", date_time_info.time.hour());
                let minute_str = format!("{:02}", date_time_info.time.minute());
                let dialog = widget::dialog()
                    .title(fl!("reminder"))
                    .primary_action(widget::button::suggested(fl!("ok")).on_press_maybe(Some(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                    )))
                    .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                    ))
                    .control(
                        widget::column::with_children(vec![
                            widget::container(widget::calendar(
                                &date_time_info.calendar,
                                move |selected_date| {
                                    Message::Application(ApplicationAction::Dialog(
                                        DialogAction::Update(DialogPage::ReminderCalendar(DateTimeInfo::with_time(
                                            selected_date,
                                            date_time_clone_select.time,
                                        ))),
                                    ))
                                },
                                move || {
                                    // Previous month - update visible month, keep selected date and time
                                    let mut new_info = date_time_clone_prev.clone();
                                    new_info.calendar.show_prev_month();
                                    Message::Application(ApplicationAction::Dialog(
                                        DialogAction::Update(DialogPage::ReminderCalendar(new_info))
                                    ))
                                },
                                move || {
                                    // Next month - update visible month, keep selected date and time
                                    let mut new_info = date_time_clone_next.clone();
                                    new_info.calendar.show_next_month();
                                    Message::Application(ApplicationAction::Dialog(
                                        DialogAction::Update(DialogPage::ReminderCalendar(new_info))
                                    ))
                                },
                                chrono::Weekday::Mon,
                            ))
                            .width(Length::Fill)
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center)
                            .into(),
                            // Time input section
                            widget::row::with_children(vec![
                                widget::text::body("Time:").into(),
                                widget::text_input("HH", hour_str)
                                    .width(Length::Fixed(50.0))
                                    .on_input(move |hour_input| {
                                        if let Ok(hour) = hour_input.parse::<u32>() {
                                            if hour <= 23 {
                                                if let Some(time) = NaiveTime::from_hms_opt(hour, date_time_clone_hour.time.minute(), 0) {
                                                    return Message::Application(ApplicationAction::Dialog(
                                                        DialogAction::Update(DialogPage::ReminderCalendar(DateTimeInfo::with_time(
                                                            date_time_clone_hour.selected_date(),
                                                            time,
                                                        ))),
                                                    ));
                                                }
                                            }
                                        }
                                        Message::Application(ApplicationAction::Dialog(DialogAction::None))
                                    })
                                    .into(),
                                widget::text::body(":").into(),
                                widget::text_input("MM", minute_str)
                                    .width(Length::Fixed(50.0))
                                    .on_input(move |minute_input| {
                                        if let Ok(minute) = minute_input.parse::<u32>() {
                                            if minute <= 59 {
                                                if let Some(time) = NaiveTime::from_hms_opt(date_time_clone_minute.time.hour(), minute, 0) {
                                                    return Message::Application(ApplicationAction::Dialog(
                                                        DialogAction::Update(DialogPage::ReminderCalendar(DateTimeInfo::with_time(
                                                            date_time_clone_minute.selected_date(),
                                                            time,
                                                        ))),
                                                    ));
                                                }
                                            }
                                        }
                                        Message::Application(ApplicationAction::Dialog(DialogAction::None))
                                    })
                                    .into(),
                            ])
                            .spacing(spacing.space_xxs)
                            .align_y(Vertical::Center)
                            .into(),
                        ])
                        .spacing(spacing.space_s),
                    );
                dialog
            }
            DialogPage::Export(contents) => {
                let dialog = widget::dialog()
                    .title(fl!("export"))
                    .control(
                        widget::container(
                            widget::scrollable(widget::text(contents)).width(Length::Fill),
                        )
                        .height(Length::Fixed(200.0))
                        .width(Length::Fill),
                    )
                    .primary_action(widget::button::suggested(fl!("copy")).on_press_maybe(Some(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Complete)),
                    )))
                    .secondary_action(widget::button::standard(fl!("cancel")).on_press(
                        Message::Application(ApplicationAction::Dialog(DialogAction::Close)),
                    ));

                dialog
            }
        }
    }
}
