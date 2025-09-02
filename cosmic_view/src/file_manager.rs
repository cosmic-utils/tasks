//! File management utilities for Cosmic View
//! 
//! This module handles file system operations, image discovery,
//! and navigation through image directories.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;

/// Supported image file extensions
const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif", "svg", "ico"
];

/// File manager for handling image files and directory navigation
#[derive(Debug, Clone)]
pub struct FileManager {
    /// Current directory path
    current_dir: PathBuf,
    /// List of image files in current directory
    image_files: Vec<PathBuf>,
    /// Current image index
    current_index: usize,
}

impl FileManager {
    /// Create a new file manager
    pub fn new() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            image_files: Vec::new(),
            current_index: 0,
        }
    }

    /// Load images from a directory
    pub fn load_directory(&mut self, path: &Path) -> Result<()> {
        self.current_dir = path.to_path_buf();
        self.image_files.clear();
        self.current_index = 0;

        // Scan directory for image files
        for entry in WalkDir::new(path)
            .max_depth(1) // Only look in the current directory, not subdirectories
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let file_path = entry.path();
                if self.is_image_file(file_path) {
                    self.image_files.push(file_path.to_path_buf());
                }
            }
        }

        // Sort files alphabetically for consistent ordering
        self.image_files.sort();

        tracing::info!("Loaded {} images from directory: {:?}", self.image_files.len(), path);
        Ok(())
    }

    /// Check if a file is an image based on its extension
    fn is_image_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return IMAGE_EXTENSIONS.contains(&ext_str.to_lowercase().as_str());
            }
        }
        false
    }

    /// Get the current image path
    pub fn current_image(&self) -> Option<&PathBuf> {
        self.image_files.get(self.current_index)
    }

    /// Get the current image index (0-based)
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get the total number of images
    pub fn total_images(&self) -> usize {
        self.image_files.len()
    }

    /// Navigate to the next image
    pub fn next_image(&mut self) -> bool {
        if self.image_files.is_empty() {
            return false;
        }

        if self.current_index < self.image_files.len() - 1 {
            self.current_index += 1;
            true
        } else {
            // Wrap around to first image
            self.current_index = 0;
            true
        }
    }

    /// Navigate to the previous image
    pub fn previous_image(&mut self) -> bool {
        if self.image_files.is_empty() {
            return false;
        }

        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            // Wrap around to last image
            self.current_index = self.image_files.len() - 1;
            true
        }
    }

    /// Navigate to a specific image by index
    pub fn go_to_image(&mut self, index: usize) -> bool {
        if index < self.image_files.len() {
            self.current_index = index;
            true
        } else {
            false
        }
    }

    /// Get the current directory path
    pub fn current_directory(&self) -> &Path {
        &self.current_dir
    }

    /// Get all image files in the current directory
    pub fn image_files(&self) -> &[PathBuf] {
        &self.image_files
    }

    /// Check if there are any images loaded
    pub fn has_images(&self) -> bool {
        !self.image_files.is_empty()
    }

    /// Get the filename of the current image
    pub fn current_filename(&self) -> Option<String> {
        self.current_image()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }
}