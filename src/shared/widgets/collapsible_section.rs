use cosmic::{
    cosmic_theme::Spacing,
    iced::{Alignment, Length},
    theme, widget, Element,
};

pub fn section<'a, Message: Clone + 'static>(
    header: Element<'a, Message>,
    rows: Vec<Element<'a, Message>>,
    collapsed: bool,
) -> Element<'a, Message> {
    let mut list = widget::list_column().list_item_padding(0).add(header);

    if !collapsed {
        for row in rows {
            list = list.add(row);
        }
    }

    widget::container(list)
        .class(cosmic::style::Container::List)
        .into()
}

pub fn section_header<'a, Message: Clone + 'static>(
    title: String,
    subtitle: Option<String>,
    count: usize,
    collapsed: bool,
    extra_buttons: Vec<Element<'a, Message>>,
    toggle_message: Message,
    spacing: &Spacing,
) -> Element<'a, Message> {
    let chevron = if collapsed {
        "go-down-symbolic"
    } else {
        "go-up-symbolic"
    };

    let badge = widget::container(widget::text(count.to_string()))
        .class(theme::Container::Tooltip)
        .padding([spacing.space_xxxs, spacing.space_xs]);

    let mut title_children: Vec<Element<'a, Message>> = vec![widget::text::heading(title).into()];
    if let Some(subtitle) = subtitle {
        title_children.push(widget::text::caption(subtitle).into());
    }
    let title_col = widget::column::with_children(title_children).width(Length::Fill);

    let mut row = widget::row::with_capacity(3 + extra_buttons.len())
        .align_y(Alignment::Center)
        .spacing(spacing.space_s)
        .padding([spacing.space_xxs, spacing.space_s])
        .push(title_col)
        .push(badge);

    for button in extra_buttons {
        row = row.push(button);
    }

    row = row.push(
        widget::button::icon(widget::icon::from_name(chevron).size(16)).on_press(toggle_message),
    );

    row.into()
}

pub fn row_item<'a, Message: Clone + 'static>(row: Element<'a, Message>) -> Element<'a, Message> {
    widget::list_column().list_item_padding(0).add(row).into()
}
