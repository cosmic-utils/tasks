// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`cosmic::widget::text_input`].

use cosmic::iced_core::Color;
use cosmic::prelude::ColorExt;
use cosmic::widget::text_input::Appearance;

pub fn text_input() -> cosmic::theme::TextInput {
    cosmic::widget::text_input::Style::Custom {
        active: Box::new(active),
        error: Box::new(error),
        hovered: Box::new(hovered),
        focused: Box::new(focused),
        disabled: Box::new(disabled),
    }
}

pub fn active(theme: &cosmic::Theme) -> Appearance {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let mut background: Color = cosmic.palette.neutral_2.into();
    background.a = 0.50;

    let corner = cosmic.corner_radii;
    let label_color = cosmic.palette.neutral_9;

    Appearance {
        background: background.into(),
        border_radius: corner.radius_s.into(),
        border_width: 1.0,
        border_offset: None,
        border_color: container.component.divider.into(),
        icon_color: None,
        text_color: None,
        placeholder_color: {
            let color: Color = container.on.into();
            color.blend_alpha(background, 0.7)
        },
        selected_text_color: cosmic.on_accent_color().into(),
        selected_fill: cosmic.accent_color().into(),
        label_color: label_color.into(),
    }
}

pub fn error(theme: &cosmic::Theme) -> Appearance {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let mut background: Color = container.component.base.into();
    background.a = 0.25;

    let corner = cosmic.corner_radii;
    let label_color = cosmic.palette.neutral_9;

    Appearance {
        background: background.into(),
        border_radius: corner.radius_s.into(),
        border_width: 1.0,
        border_offset: Some(2.0),
        border_color: Color::from(cosmic.destructive_color()),
        icon_color: None,
        text_color: None,
        placeholder_color: {
            let color: Color = container.on.into();
            color.blend_alpha(background, 0.7)
        },
        selected_text_color: cosmic.on_accent_color().into(),
        selected_fill: cosmic.accent_color().into(),
        label_color: label_color.into(),
    }
}

pub fn hovered(theme: &cosmic::Theme) -> Appearance {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let mut background: Color = cosmic.palette.neutral_2.into();
    background.a = 0.25;

    let corner = cosmic.corner_radii;
    let label_color = cosmic.palette.neutral_9;

    Appearance {
        background: background.into(),
        border_radius: corner.radius_s.into(),
        border_width: 1.0,
        border_offset: None,
        border_color: cosmic.accent.base.into(),
        icon_color: None,
        text_color: None,
        placeholder_color: {
            let color: Color = container.on.into();
            color.blend_alpha(background, 0.7)
        },
        selected_text_color: cosmic.on_accent_color().into(),
        selected_fill: cosmic.accent_color().into(),
        label_color: label_color.into(),
    }
}

pub fn focused(theme: &cosmic::Theme) -> Appearance {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let mut background: Color = cosmic.palette.neutral_2.into();
    background.a = 0.25;

    let corner = cosmic.corner_radii;
    let label_color = cosmic.palette.neutral_9;

    Appearance {
        background: background.into(),
        border_radius: corner.radius_s.into(),
        border_width: 1.0,
        border_offset: Some(2.0),
        border_color: cosmic.accent.base.into(),
        icon_color: None,
        text_color: None,
        placeholder_color: {
            let color: Color = container.on.into();
            color.blend_alpha(background, 0.7)
        },
        selected_text_color: cosmic.on_accent_color().into(),
        selected_fill: cosmic.accent_color().into(),
        label_color: label_color.into(),
    }
}

pub fn disabled(theme: &cosmic::Theme) -> Appearance {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let mut background: Color = container.component.base.into();
    background.a = 0.25;

    let corner = cosmic.corner_radii;
    let label_color = cosmic.palette.neutral_9;

    Appearance {
        background: background.into(),
        border_radius: corner.radius_s.into(),
        border_width: 1.0,
        border_offset: Some(2.0),
        border_color: cosmic.accent.base.into(),
        icon_color: None,
        text_color: None,
        placeholder_color: {
            let color: Color = container.on.into();
            color.blend_alpha(background, 0.7)
        },
        selected_text_color: cosmic.on_accent_color().into(),
        selected_fill: cosmic.accent_color().into(),
        label_color: label_color.into(),
    }
}
