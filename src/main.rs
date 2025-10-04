mod app;
mod core;
mod pages;
mod storage;
mod auth;
mod integration;
mod cli;

use core::settings;
use auth::MsTodoAuth;
use tracing::{info, error};

pub use app::error::*;

fn main() {
    // Check if CLI arguments are provided (more than just the program name)
    if std::env::args().len() > 1 {
        // Run CLI mode with tokio runtime
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        match rt.block_on(cli::run()) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("CLI error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Run GUI mode (synchronous, no tokio)
        match run_gui() {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("GUI error: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}

pub fn run_gui() -> cosmic::iced::Result {
    // Initialize settings
    settings::app::init();
    
    info!("🚀 Starting Tasks app with Microsoft Todo integration...");
    
    // Check authentication first
    let auth = match MsTodoAuth::new() {
        Ok(auth) => auth,
        Err(e) => {
            error!("❌ Failed to initialize authentication: {}", e);
            return cosmic::app::run::<settings::error::View>(
                settings::app::settings(),
                settings::error::flags(crate::LocalStorageError::AuthenticationFailed(e.to_string())),
            );
        }
    };
    
    // Check if we can get a valid access token (with automatic refresh)
    info!("🔍 Checking for valid access token...");
    match auth.get_access_token() {
        Ok(_token) => {
            info!("✅ Valid access token obtained (refreshed if needed), proceeding to main app...");
        }
        Err(_e) => {
            info!("🔐 No valid tokens found or refresh failed, starting authentication flow...");
            
            match auth.authorize() {
                Ok(config) => {
                    info!("✅ Authentication successful! Token expires at: {}", config.expires_at);
                }
                Err(e) => {
                    error!("❌ Authentication failed: {}", e);
                    return cosmic::app::run::<settings::error::View>(
                        settings::app::settings(),
                        settings::error::flags(crate::LocalStorageError::AuthenticationFailed(format!("Authentication failed: {}", e))),
                    );
                }
            }
        }
    }
    
    // Now proceed with the normal app flow
    match settings::app::storage() {
        Ok(storage) => {
            cosmic::app::run::<app::TasksApp>(settings::app::settings(), app::flags(storage))
        }
        Err(error) => cosmic::app::run::<settings::error::View>(
            settings::app::settings(),
            settings::error::flags(error),
        ),
    }
}
