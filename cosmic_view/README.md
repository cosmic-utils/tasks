# Cosmic View

A simple image viewer application built with libcosmic, following the same architecture as the msToDO application.

## Features

- **Simple Image Viewer**: Clean interface for viewing images
- **Navigation Controls**: Left and right arrow buttons to navigate through images in a folder
- **Keyboard Shortcuts**: Arrow keys for navigation
- **About Page**: Information about the application
- **Modern UI**: Built with libcosmic for a native Cosmic desktop experience

## Architecture

The application follows the same architectural patterns as the msToDO application:

### Core Structure
- `main.rs`: Application entry point with logging initialization
- `app.rs`: Main application implementation using libcosmic Application trait
- `config.rs`: Configuration management with cosmic-config
- `image_viewer.rs`: Image viewer component with navigation controls
- `file_manager.rs`: File system operations and image discovery

### Key Components

#### Application Structure
- **CosmicViewApp**: Main application struct implementing the Application trait
- **Message System**: Clean message-based architecture for handling user interactions
- **Key Bindings**: Configurable keyboard shortcuts for navigation

#### Image Viewer
- **Navigation Controls**: Previous/Next buttons positioned above the image
- **File Management**: Automatic discovery of image files in directories
- **Image Display**: Support for common image formats (JPG, PNG, GIF, etc.)

#### Configuration
- **Theme Support**: System, Dark, and Light theme options
- **Settings Persistence**: Configuration saved using cosmic-config
- **User Preferences**: Customizable display options

## Usage

### Navigation
- **Mouse**: Click the ← and → buttons to navigate
- **Keyboard**: Use left/right arrow keys to navigate
- **Wrapping**: Navigation wraps around from last to first image

### Supported Formats
- JPEG/JPG
- PNG
- GIF
- BMP
- WebP
- TIFF
- SVG
- ICO

## Building

```bash
cargo build --release
```

## Dependencies

- **libcosmic**: Core UI framework
- **image**: Image processing and loading
- **walkdir**: Directory traversal
- **tracing**: Logging system
- **cosmic-config**: Configuration management

## Code Comments

The codebase includes comprehensive comments explaining:
- Architecture decisions and patterns
- Component responsibilities
- Message flow and state management
- File system operations
- Image loading and display logic

## Future Enhancements

- File dialog for opening specific directories
- Image zoom and pan functionality
- Slideshow mode with automatic progression
- Image metadata display
- Thumbnail navigation
- Fullscreen mode

## License

GPL-3.0-only