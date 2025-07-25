//! Plugin system for dynamic node loading

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use libloading::{Library, Symbol};
use nodle_plugin_sdk::{NodePlugin, PluginInfo, PluginError, NodeRegistryTrait, MenuStructure, PluginHandle, PluginNodeHandle};

// Re-export plugin UI types for core use
pub use nodle_plugin_sdk::{UIElement, UIAction, ParameterUI, NodeData, ParameterChange};
use crate::workspace::WorkspaceMenuItem;

/// Loaded plugin wrapper
struct LoadedPlugin {
    library: Library,
    plugin: Box<dyn NodePlugin>,
    info: PluginInfo,
}

/// Plugin manager for loading and managing external node plugins
pub struct PluginManager {
    loaded_plugins: HashMap<String, LoadedPlugin>,
    plugin_directories: Vec<PathBuf>,
    /// Store active plugin node instances for viewport rendering
    pub plugin_node_instances: HashMap<crate::nodes::NodeId, Box<dyn nodle_plugin_sdk::PluginNode>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        let mut plugin_directories = Vec::new();
        
        // Add standard plugin directories
        if let Some(home) = dirs::home_dir() {
            plugin_directories.push(home.join(".nodle/plugins"));
        }
        plugin_directories.push(PathBuf::from("./plugins"));
        
        Self {
            loaded_plugins: HashMap::new(),
            plugin_directories,
            plugin_node_instances: HashMap::new(),
        }
    }
    
    /// Add a plugin directory to search
    pub fn add_plugin_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.plugin_directories.push(path.as_ref().to_path_buf());
    }
    
    /// Scan directories for plugins and load them
    pub fn discover_and_load_plugins(&mut self) -> Result<Vec<PluginInfo>, PluginError> {
        let mut loaded_plugins = Vec::new();
        
        // Clone the directories to avoid borrowing issues
        let directories = self.plugin_directories.clone();
        
        for dir in &directories {
            if dir.exists() && dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if self.is_plugin_file(&path) {
                            match self.load_plugin(&path) {
                                Ok(info) => {
                                    println!("✅ Successfully loaded plugin: {}", info.name);
                                    loaded_plugins.push(info);
                                }
                                Err(e) => {
                                    println!("❌ Failed to load plugin {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(loaded_plugins)
    }
    
    /// Load a specific plugin from path
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<PluginInfo, PluginError> {
        let path = path.as_ref();
        
        // Load the dynamic library
        let library = unsafe {
            Library::new(path)
                .map_err(|e| PluginError::LoadError(format!("Failed to load library: {}", e)))?
        };
        
        // Get the plugin creation function - now returns a safe concrete type
        let create_plugin: Symbol<unsafe extern "C" fn() -> PluginHandle> = unsafe {
            library.get(b"create_plugin")
                .map_err(|e| PluginError::LoadError(format!("Missing create_plugin function: {}", e)))?
        };
        
        // Create the plugin instance safely
        let plugin_handle = unsafe { create_plugin() };
        let plugin = unsafe { plugin_handle.into_plugin() };
        
        // Get plugin info
        let info = plugin.plugin_info();
        
        // Check version compatibility (basic check)
        if !self.is_compatible_version(&info.compatible_version) {
            return Err(PluginError::CompatibilityError(format!(
                "Plugin {} requires Nodle version {}, but current version is 0.1.0",
                info.name, info.compatible_version
            )));
        }
        
        // Call plugin initialization
        plugin.on_load().map_err(|e| PluginError::InitError(format!("Plugin initialization failed: {}", e)))?;
        
        // Store the loaded plugin
        let loaded_plugin = LoadedPlugin {
            library,
            plugin,
            info: info.clone(),
        };
        
        self.loaded_plugins.insert(info.name.clone(), loaded_plugin);
        
        println!("Successfully loaded plugin: {} v{}", info.name, info.version);
        
        Ok(info)
    }
    
    /// Unload a plugin by name
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        if let Some(loaded_plugin) = self.loaded_plugins.remove(name) {
            // Call plugin cleanup
            loaded_plugin.plugin.on_unload()
                .map_err(|e| PluginError::Other(format!("Plugin cleanup failed: {}", e)))?;
            
            // Library will be dropped automatically
            println!("Unloaded plugin: {}", name);
            Ok(())
        } else {
            Err(PluginError::Other(format!("Plugin '{}' not found", name)))
        }
    }
    
    /// Register all plugin nodes with a registry
    pub fn register_plugin_nodes(&self, registry: &mut dyn NodeRegistryTrait) -> Result<(), PluginError> {
        for loaded_plugin in self.loaded_plugins.values() {
            loaded_plugin.plugin.register_nodes(registry);
        }
        Ok(())
    }
    
    /// Get info about all loaded plugins
    pub fn get_loaded_plugins(&self) -> Vec<&PluginInfo> {
        self.loaded_plugins.values().map(|p| &p.info).collect()
    }
    
    /// Get menu structures from all loaded plugins
    pub fn get_plugin_menu_structures(&self) -> Vec<MenuStructure> {
        let mut menu_structures = Vec::new();
        
        for loaded_plugin in self.loaded_plugins.values() {
            let plugin_menus = loaded_plugin.plugin.get_menu_structure();
            menu_structures.extend(plugin_menus);
        }
        
        menu_structures
    }
    
    /// Convert plugin menu structures to workspace menu items
    pub fn get_workspace_menu_items(&self) -> Vec<WorkspaceMenuItem> {
        let plugin_menus = self.get_plugin_menu_structures();
        plugin_menus.into_iter().map(|menu| self.convert_menu_structure(menu)).collect()
    }
    
    /// Convert a single MenuStructure to WorkspaceMenuItem
    fn convert_menu_structure(&self, menu: MenuStructure) -> WorkspaceMenuItem {
        match menu {
            MenuStructure::Category { name, items } => {
                WorkspaceMenuItem::Category {
                    name,
                    items: items.into_iter().map(|item| self.convert_menu_structure(item)).collect(),
                }
            }
            MenuStructure::Node { name, node_type, .. } => {
                WorkspaceMenuItem::Node {
                    name,
                    node_type,
                }
            }
        }
    }
    
    /// Check if a file is a plugin library
    fn is_plugin_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("dll") => true,  // Windows
                Some("so") => true,   // Linux
                Some("dylib") => true, // macOS
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Check if plugin version is compatible with current Nodle version
    fn is_compatible_version(&self, plugin_version: &str) -> bool {
        // For now, just check if it starts with "0.1"
        // In the future, implement proper semantic version checking
        plugin_version.starts_with("0.1")
    }
    
    /// Store a plugin node instance for viewport rendering
    pub fn store_plugin_node_instance(&mut self, node_id: crate::nodes::NodeId, plugin_node: Box<dyn nodle_plugin_sdk::PluginNode>) {
        self.plugin_node_instances.insert(node_id, plugin_node);
    }
    
    /// Get a plugin node instance for viewport rendering
    pub fn get_plugin_node_for_rendering(&mut self, node_id: crate::nodes::NodeId, _node_title: &str) -> Option<&mut Box<dyn nodle_plugin_sdk::PluginNode>> {
        // Check if we have a stored instance for this node
        if let Some(plugin_node) = self.plugin_node_instances.get_mut(&node_id) {
            // Verify it supports viewport rendering
            if plugin_node.supports_viewport() {
                return Some(plugin_node);
            }
        }
        
        // For now, we only support nodes that were created and stored during node creation
        // Future enhancement: create instances on-demand
        None
    }
    
    /// Helper to find viewport node type in menu structure
    fn find_viewport_node_type_in_menu(&self, menu_item: &MenuStructure) -> Option<String> {
        match menu_item {
            MenuStructure::Category { items, .. } => {
                for item in items {
                    if let Some(node_type) = self.find_viewport_node_type_in_menu(item) {
                        return Some(node_type);
                    }
                }
                None
            }
            MenuStructure::Node { name, node_type, metadata } => {
                if name.contains("Viewport") && metadata.panel_type == nodle_plugin_sdk::PanelType::Viewport {
                    Some(node_type.clone())
                } else {
                    None
                }
            }
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}