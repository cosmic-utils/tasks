//! Cosmic View - A simple image viewer application built with libcosmic

mod app;

use cosmic::app::Settings;
use tracing::{info, error};

/// Main entry point for the Cosmic View application
pub fn main() -> cosmic::iced::Result {
    // Initialize logging
    init_logging();
    
    info!("🖼️  Starting Cosmic View image viewer...");
    
    // Initialize the application with default settings
    cosmic::app::run::<app::CosmicViewApp>(
        Settings::default()
            .antialiasing(true)
            .client_decorations(true)
            .size(cosmic::iced::Size::new(1000.0, 700.0))
            .debug(false),
        app::Flags::default(),
    )
}

/// Initialize logging system for the application
fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("wgpu_core=error".parse().unwrap())
        .add_directive("naga=error".parse().unwrap())
        .add_directive("cosmic_text=error".parse().unwrap())
        .add_directive("sctk=error".parse().unwrap())
        .add_directive("wgpu_hal=error".parse().unwrap())
        .add_directive("iced_wgpu=error".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
}