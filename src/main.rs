// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

mod app;
mod content;
mod details;
mod todo;

#[rustfmt::skip]
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (settings, flags) = app::settings::init();
    cosmic::app::run::<app::App>(settings, flags)?;
    Ok(())
}
