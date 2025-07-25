//! File management system for the node editor
//!
//! Handles saving, loading, and file state management for node graphs.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::nodes::NodeGraph;
use crate::editor::canvas::Canvas;

/// Save file data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub metadata: SaveMetadata,
    pub viewport: CanvasData,
    pub root_graph: NodeGraph,
}

/// Metadata for save files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub created: String,    // ISO 8601 timestamp
    pub modified: String,   // ISO 8601 timestamp
    pub creator: String,    // "Nōdle 1.0"
    pub description: String,
}

/// Canvas state for save files (2D node editor pan/zoom)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasData {  // Renamed from ViewportData to avoid conflict with 3D viewport
    pub pan_offset: [f32; 2],
    pub zoom: f32,
}

/// Manages file operations for the node editor
pub struct FileManager {
    /// Current file path (None if unsaved/new file)
    current_file_path: Option<PathBuf>,
    /// Whether the file has been modified since last save
    is_modified: bool,
}

impl FileManager {
    /// Create a new file manager
    pub fn new() -> Self {
        Self {
            current_file_path: None,
            is_modified: false,
        }
    }

    /// Get the current file path
    pub fn current_file_path(&self) -> Option<&PathBuf> {
        self.current_file_path.as_ref()
    }

    /// Check if there are unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.is_modified
    }

    /// Mark the file as modified
    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }

    /// Mark the file as saved (no modifications)
    pub fn mark_saved(&mut self) {
        self.is_modified = false;
    }

    /// Get display name for the current file
    pub fn get_file_display_name(&self) -> String {
        match &self.current_file_path {
            Some(path) => {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                
                if self.is_modified {
                    format!("{}*", file_name)
                } else {
                    file_name.to_string()
                }
            }
            None => {
                if self.is_modified {
                    "Untitled*".to_string()
                } else {
                    "Untitled".to_string()
                }
            }
        }
    }

    /// Create a new file (reset state)
    pub fn new_file(&mut self) {
        self.current_file_path = None;
        self.is_modified = false;
    }

    /// Save the current graph to a file
    pub fn save_to_file(&mut self, file_path: &Path, graph: &NodeGraph, canvas: &Canvas) -> Result<(), String> {
        let save_data = SaveData {
            version: "1.0".to_string(),
            metadata: SaveMetadata {
                created: chrono::Utc::now().to_rfc3339(),
                modified: chrono::Utc::now().to_rfc3339(),
                creator: "Nōdle 1.0".to_string(),
                description: "Node graph created with Nōdle".to_string(),
            },
            viewport: CanvasData {
                pan_offset: [canvas.pan_offset.x, canvas.pan_offset.y],
                zoom: canvas.zoom,
            },
            root_graph: graph.clone(),
        };

        let json_content = serde_json::to_string_pretty(&save_data)
            .map_err(|e| format!("Failed to serialize save data: {}", e))?;

        std::fs::write(file_path, json_content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        // Update file manager state
        self.current_file_path = Some(file_path.to_path_buf());
        self.is_modified = false;

        Ok(())
    }

    /// Load a graph from a file
    pub fn load_from_file(&mut self, file_path: &Path) -> Result<(NodeGraph, Canvas), String> {
        let file_content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let save_data: SaveData = serde_json::from_str(&file_content)
            .map_err(|e| format!("Failed to parse save file: {}", e))?;

        // Create canvas from saved data
        let mut canvas = Canvas::new();
        canvas.pan_offset = egui::Vec2::new(
            save_data.viewport.pan_offset[0], 
            save_data.viewport.pan_offset[1]
        );
        canvas.zoom = save_data.viewport.zoom;

        // Update file manager state
        self.current_file_path = Some(file_path.to_path_buf());
        self.is_modified = false;

        Ok((save_data.root_graph, canvas))
    }

    /// Save the current file (use existing path or prompt for new path)
    pub fn save_file(&mut self, graph: &NodeGraph, canvas: &Canvas) -> Result<(), String> {
        if let Some(path) = &self.current_file_path.clone() {
            self.save_to_file(path, graph, canvas)
        } else {
            Err("No file path set. Use save_as instead.".to_string())
        }
    }

    /// Open file dialog and load selected file
    pub fn open_file_dialog(&mut self) -> Result<Option<(NodeGraph, Canvas)>, String> {
        use rfd::FileDialog;
        
        if let Some(path) = FileDialog::new()
            .add_filter("JSON files", &["json"])
            .pick_file()
        {
            match self.load_from_file(&path) {
                Ok((graph, canvas)) => Ok(Some((graph, canvas))),
                Err(error) => Err(error),
            }
        } else {
            Ok(None) // User cancelled dialog
        }
    }

    /// Save as file dialog
    pub fn save_as_file_dialog(&mut self, graph: &NodeGraph, canvas: &Canvas) -> Result<bool, String> {
        use rfd::FileDialog;
        
        if let Some(path) = FileDialog::new()
            .add_filter("JSON files", &["json"])
            .save_file()
        {
            match self.save_to_file(&path, graph, canvas) {
                Ok(()) => Ok(true),
                Err(error) => Err(error),
            }
        } else {
            Ok(false) // User cancelled dialog
        }
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}