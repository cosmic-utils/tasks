//! Main application structure for Cosmic View

use cosmic::{
    app::{self, Core}, iced::Alignment, widget::{self, button, column, container, row, text}, Application, ApplicationExt, Element
};

/// Main application structure
pub struct CosmicViewApp {
    /// Core application state
    core: Core,
    /// Current image index
    current_image: usize,
    /// Total images
    total_images: usize,
}

/// Application flags passed during initialization
#[derive(Debug, Clone, Default)]
pub struct Flags {}

/// Main application messages
#[derive(Debug, Clone)]
pub enum Message {
    /// Navigate to next image
    NextImage,
    /// Navigate to previous image
    PreviousImage,
}

impl Application for CosmicViewApp {
    type Executor = cosmic::executor::Default;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = "com.github.cosmic-view";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, app::Task<Self::Message>) {
        let mut app = CosmicViewApp {
            core,
            current_image: 0,
            total_images: 5, // Mock data for now
        };

        // Set initial window title
        let mut tasks = vec![];
        if let Some(id) = app.core.main_window_id() {
            tasks.push(app.set_window_title("Cosmic View".to_string(), id));
        }

        (app, app::Task::batch(tasks))
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Self::Message> {
        match message {
            Message::NextImage => {
                if self.current_image < self.total_images - 1 {
                    self.current_image += 1;
                } else {
                    self.current_image = 0; // Wrap around
                }
                app::Task::none()
            }
            Message::PreviousImage => {
                if self.current_image > 0 {
                    self.current_image -= 1;
                } else {
                    self.current_image = self.total_images - 1; // Wrap around
                }
                app::Task::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let mut content = column().align_x(Alignment::Center);
            
            

        // Navigation controls
        let nav_controls = cosmic::widget::row()
            .spacing(20)
            .align_y(Alignment::Center);


        // Previous button
        let prev_button = button::standard("← Previous".to_string())
            .on_press(Message::PreviousImage)
            .padding(10);

        // Next button  
        let next_button = button::standard("Next →".to_string())
            .on_press(Message::NextImage)
            .padding(10);

        let nav_controls = nav_controls
            .push(prev_button)
            .push(next_button);

        content = content.push(nav_controls);

        // Image placeholder
        content = content.push(
            container(text("Image Viewer\n\nThis is a placeholder for the image display.\nUse the buttons to navigate."))
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .align_y(cosmic::iced::alignment::Vertical::Center)
                .padding(20)
        );

        // Image information
        let info_text = format!(
            "Image {} of {}",
            self.current_image + 1,
            self.total_images
        );
        content = content.push(text(info_text));

        container(content)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .padding(20)
            .into()
    }
}