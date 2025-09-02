//! Image viewer component for Cosmic View
//! 
//! This module handles the display of images and navigation controls.

use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length, Subscription,
    },
    widget::{self, button, column, container, row, text},
    Element,
};
use std::path::Path;

use crate::file_manager::FileManager;

/// Image viewer component that displays images with navigation controls
#[derive(Debug, Clone)]
pub struct ImageViewer {
    /// File manager for handling image navigation
    file_manager: FileManager,
    /// Currently loaded image data
    current_image_data: Option<cosmic::widget::image::ImageData>,
    /// Whether to show the filename
    show_filename: bool,
}

/// Messages that the image viewer can handle
#[derive(Debug, Clone)]
pub enum Message {
    /// Navigate to the next image
    NextImage,
    /// Navigate to the previous image
    PreviousImage,
    /// Load a new directory
    LoadDirectory(std::path::PathBuf),
    /// Toggle filename display
    ToggleFilename,
    /// Image was loaded successfully
    ImageLoaded(cosmic::widget::image::ImageData),
    /// Failed to load image
    ImageLoadFailed(String),
}

impl ImageViewer {
    /// Create a new image viewer
    pub fn new() -> Self {
        Self {
            file_manager: FileManager::new(),
            current_image_data: None,
            show_filename: true,
        }
    }

    /// Update the image viewer state
    pub fn update(&mut self, message: Message) -> cosmic::iced::Command<Message> {
        match message {
            Message::NextImage => {
                if self.file_manager.next_image() {
                    self.load_current_image()
                } else {
                    cosmic::iced::Command::none()
                }
            }
            Message::PreviousImage => {
                if self.file_manager.previous_image() {
                    self.load_current_image()
                } else {
                    cosmic::iced::Command::none()
                }
            }
            Message::LoadDirectory(path) => {
                if let Err(e) = self.file_manager.load_directory(&path) {
                    tracing::error!("Failed to load directory: {}", e);
                }
                self.load_current_image()
            }
            Message::ToggleFilename => {
                self.show_filename = !self.show_filename;
                cosmic::iced::Command::none()
            }
            Message::ImageLoaded(image_data) => {
                self.current_image_data = Some(image_data);
                cosmic::iced::Command::none()
            }
            Message::ImageLoadFailed(error) => {
                tracing::error!("Failed to load image: {}", error);
                cosmic::iced::Command::none()
            }
        }
    }

    /// Load the current image from the file manager
    fn load_current_image(&self) -> cosmic::iced::Command<Message> {
        if let Some(image_path) = self.file_manager.current_image() {
            let path = image_path.clone();
            cosmic::iced::Command::perform(
                async move {
                    match image::open(&path) {
                        Ok(img) => {
                            // Convert to RGB8 format for display
                            let rgb_img = img.to_rgb8();
                            let image_data = cosmic::widget::image::ImageData::new(
                                cosmic::widget::image::Handle::from_pixels(
                                    rgb_img.width(),
                                    rgb_img.height(),
                                    rgb_img.into_raw(),
                                ),
                            );
                            Message::ImageLoaded(image_data)
                        }
                        Err(e) => Message::ImageLoadFailed(format!("Failed to load image: {}", e)),
                    }
                },
                |result| result,
            )
        } else {
            cosmic::iced::Command::none()
        }
    }

    /// Get the view of the image viewer
    pub fn view(&self) -> Element<Message> {
        let mut content = column!()
            .spacing(20)
            .align_items(Alignment::Center);

        // Navigation controls
        if self.file_manager.has_images() {
            let nav_controls = row!()
                .spacing(20)
                .align_items(Alignment::Center);

            // Previous button
            let prev_button = button("←")
                .on_press(Message::PreviousImage)
                .padding(10);

            // Next button  
            let next_button = button("→")
                .on_press(Message::NextImage)
                .padding(10);

            let nav_controls = nav_controls
                .push(prev_button)
                .push(next_button);

            content = content.push(nav_controls);

            // Image display
            if let Some(image_data) = &self.current_image_data {
                let image_widget = cosmic::widget::image(image_data)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(cosmic::iced::ContentFit::Contain);

                content = content.push(
                    container(image_widget)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(20)
                );
            } else {
                // Loading placeholder
                content = content.push(
                    container(text("Loading image..."))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                );
            }

            // Image information
            if self.show_filename {
                if let Some(filename) = self.file_manager.current_filename() {
                    let info_text = format!(
                        "{} ({}/{})",
                        filename,
                        self.file_manager.current_index() + 1,
                        self.file_manager.total_images()
                    );
                    content = content.push(text(info_text));
                }
            }
        } else {
            // No images message
            content = content.push(
                container(text("No images found in this directory"))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
            );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    /// Get the current file manager
    pub fn file_manager(&self) -> &FileManager {
        &self.file_manager
    }

    /// Get mutable reference to file manager
    pub fn file_manager_mut(&mut self) -> &mut FileManager {
        &mut self.file_manager
    }

    /// Set the show filename preference
    pub fn set_show_filename(&mut self, show: bool) {
        self.show_filename = show;
    }

    /// Get the show filename preference
    pub fn show_filename(&self) -> bool {
        self.show_filename
    }
}