// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`crate::widget::segmented_button`].

use cosmic::iced::Border;
use cosmic::iced::{border::Radius, Background};
use cosmic::widget::segmented_button::ItemStatusAppearance;
use cosmic::widget::segmented_button::{Appearance, ItemAppearance};

pub fn segmented_control() -> cosmic::theme::SegmentedButton {
    cosmic::style::SegmentedButton::Custom(Box::new(horizontal))
}

fn horizontal(theme: &cosmic::Theme) -> Appearance {
    let container = &theme.current_container();

    let cosmic = theme.cosmic();
    let active = horizontal::selection_active(theme);

    let mut background: cosmic::iced::Color = cosmic.palette.neutral_2.into();
    background.a = 0.50;

    let rad_m = cosmic.corner_radii.radius_m;
    let rad_0 = cosmic.corner_radii.radius_0;
    Appearance {
        background: Some(background.into()),
        border: Border {
            radius: rad_m.into(),
            ..Default::default()
        },
        inactive: ItemStatusAppearance {
            background: None,
            first: ItemAppearance {
                border: Border {
                    radius: Radius::from([rad_m[0], rad_0[1], rad_0[2], rad_m[3]]),
                    ..Default::default()
                },
            },
            middle: ItemAppearance {
                border: Border {
                    radius: cosmic.corner_radii.radius_0.into(),
                    ..Default::default()
                },
            },
            last: ItemAppearance {
                border: Border {
                    radius: Radius::from([rad_0[0], rad_m[1], rad_m[2], rad_0[3]]),
                    ..Default::default()
                },
            },
            text_color: container.component.on.into(),
        },
        hover: hover(theme, &active),
        active,
        ..Default::default()
    }
}

mod horizontal {
    use cosmic::iced::Border;
    use cosmic::iced::{border::Radius, Background};
    use cosmic::widget::segmented_button::{ItemAppearance, ItemStatusAppearance};

    pub fn selection_active(theme: &cosmic::Theme) -> ItemStatusAppearance {
        let mut color = theme.cosmic().palette.neutral_5;
        color.alpha = 0.2;

        let rad_m = theme.cosmic().corner_radii.radius_m;
        let rad_0 = theme.cosmic().corner_radii.radius_0;

        ItemStatusAppearance {
            background: Some(Background::Color(color.into())),
            first: ItemAppearance {
                border: Border {
                    radius: Radius::from([rad_m[0], rad_0[1], rad_0[2], rad_m[3]]),
                    ..Default::default()
                },
            },
            middle: ItemAppearance {
                border: Border {
                    radius: theme.cosmic().corner_radii.radius_0.into(),
                    ..Default::default()
                },
            },
            last: ItemAppearance {
                border: Border {
                    radius: Radius::from([rad_0[0], rad_m[1], rad_m[2], rad_0[3]]),
                    ..Default::default()
                },
            },
            text_color: theme.cosmic().accent.base.into(),
        }
    }
}

pub fn hover(theme: &cosmic::Theme, default: &ItemStatusAppearance) -> ItemStatusAppearance {
    let mut color = theme.cosmic().palette.neutral_8;
    color.alpha = 0.2;
    ItemStatusAppearance {
        background: Some(Background::Color(color.into())),
        text_color: theme.cosmic().accent.base.into(),
        ..*default
    }
}
