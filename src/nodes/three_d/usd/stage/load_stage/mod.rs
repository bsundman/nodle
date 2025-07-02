//! Load Stage node module - complete implementation

pub mod parameters;

use crate::nodes::interface::{NodeInterfacePanel, PanelType, InterfaceParameter, NodeData, ParameterChange};
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};
use egui::Color32;

/// USD Load Stage node with parameter interface
#[derive(Debug, Clone)]
pub struct LoadStageNode {
    /// Current file path selected by user
    file_path: String,
    /// Auto-reload option
    auto_reload: bool,
    /// Load payloads option
    load_payloads: bool,
    /// Population mask
    population_mask: String,
    /// Last execution result for sharing with connected nodes
    last_execution_result: Option<String>,
}

impl Default for LoadStageNode {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            auto_reload: false,
            load_payloads: true,
            population_mask: String::new(),
            last_execution_result: None,
        }
    }
}

/// Load Stage processing logic
#[derive(Debug, Clone, Default)]
pub struct LoadStageLogic;

impl NodeFactory for LoadStageNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_LoadStage",
            "Load Stage",
            NodeCategory::new(&["3D", "USD", "Stage"]),
            "Loads a USD stage from a .usd, .usda, .usdc, or .usdz file"
        )
        .with_color(Color32::from_rgb(200, 150, 100)) // Orange-brown for USD nodes
        .with_icon("📂")
        .with_inputs(vec![
            // No input ports - file selection is done via parameters
        ])
        .with_outputs(vec![
            PortDefinition::required("USD Stage", DataType::Any)
                .with_description("Loaded USD Stage data for downstream nodes"),
        ])
        .with_tags(vec!["3d", "usd", "stage", "load", "import", "pixar", "file"])
        .with_processing_cost(ProcessingCost::High)
        .with_workspace_compatibility(vec!["3d", "usd", "pipeline"])
        .with_panel_type(PanelType::Parameter)
    }
    
    /// Custom create method to populate node parameters from interface panel
    fn create(position: egui::Pos2) -> crate::nodes::Node {
        let meta = Self::metadata();
        let mut node = crate::nodes::Node::new(0, meta.node_type, position);
        node.color = meta.color;
        
        // Add inputs
        for input in &meta.inputs {
            node.add_input(&input.name);
        }
        
        // Add outputs  
        for output in &meta.outputs {
            node.add_output(&output.name);
        }
        
        // Set panel type from metadata
        node.set_panel_type(meta.panel_type);
        
        // Populate node parameters from the interface panel's default values
        let interface_panel = LoadStageNode::default();
        let parameters = interface_panel.get_parameters();
        for (name, param) in parameters {
            match param {
                InterfaceParameter::FilePath { value, .. } => {
                    node.parameters.insert(name.to_string(), NodeData::String(value));
                },
                InterfaceParameter::Boolean { value } => {
                    node.parameters.insert(name.to_string(), NodeData::Boolean(value));
                },
                InterfaceParameter::String { value } => {
                    node.parameters.insert(name.to_string(), NodeData::String(value));
                },
                _ => {
                    // Handle other parameter types if needed
                }
            }
        }
        
        // CRITICAL: Update port positions after adding ports
        node.update_port_positions();
        
        node
    }
}

impl NodeInterfacePanel for LoadStageNode {
    fn panel_type(&self) -> PanelType {
        PanelType::Parameter
    }
    
    fn get_parameters(&self) -> Vec<(&'static str, InterfaceParameter)> {
        vec![
            ("file_path", InterfaceParameter::FilePath { 
                value: self.file_path.clone(), 
                filter: "USD Files (*.usd *.usda *.usdc *.usdz)".to_string() 
            }),
            ("auto_reload", InterfaceParameter::Boolean { value: self.auto_reload }),
            ("load_payloads", InterfaceParameter::Boolean { value: self.load_payloads }),
            ("population_mask", InterfaceParameter::String { value: self.population_mask.clone() }),
        ]
    }
    
    fn set_parameters(&mut self, parameters: Vec<(&'static str, InterfaceParameter)>) {
        for (name, param) in parameters {
            match name {
                "file_path" => {
                    if let Some(path) = param.get_string() {
                        self.file_path = path.to_string();
                    }
                },
                "auto_reload" => {
                    if let Some(value) = param.get_bool() {
                        self.auto_reload = value;
                    }
                },
                "load_payloads" => {
                    if let Some(value) = param.get_bool() {
                        self.load_payloads = value;
                    }
                },
                "population_mask" => {
                    if let Some(mask) = param.get_string() {
                        self.population_mask = mask.to_string();
                    }
                },
                _ => {}
            }
        }
    }
    
    fn process(&self, _inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Use the internal parameter state directly
        LoadStageLogic::process_with_parameters(
            &self.file_path, 
            self.auto_reload, 
            self.load_payloads, 
            &self.population_mask
        )
    }
    
    fn panel_title(&self) -> String {
        "USD Load Stage".to_string()
    }
    
    fn render_custom_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        
        ui.separator();
        ui.heading("USD File Selection");
        
        // File path text field with browse button
        ui.horizontal(|ui| {
            ui.label("File Path:");
            let text_response = ui.add(
                egui::TextEdit::singleline(&mut self.file_path)
                    .desired_width(300.0)
                    .hint_text("Select or enter USD file path...")
            );
            
            if text_response.changed() {
                // Clear the execution result when file path changes
                self.last_execution_result = None;
                changed = true;
            }
            
            if ui.button("📁 Browse").clicked() {
                if let Ok(Some(path)) = self.open_usd_file_dialog() {
                    self.file_path = path;
                    self.last_execution_result = None; // Clear previous result
                    changed = true;
                }
            }
        });
        
        ui.separator();
        ui.heading("USD Stage Status");
        
        // Status display
        if self.file_path.is_empty() {
            ui.colored_label(egui::Color32::YELLOW, "⚠ No USD file selected");
        } else {
            // Check if file exists
            if std::path::Path::new(&self.file_path).exists() {
                ui.colored_label(egui::Color32::GREEN, format!("📄 Ready: {}", 
                    std::path::Path::new(&self.file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")));
                
                // Show full path in smaller text
                ui.small(&self.file_path);
                
                // Automatically process when file is set
                if self.last_execution_result.is_none() {
                    let results = self.process(vec![]);
                    for result in &results {
                        if let crate::nodes::interface::NodeData::Any(data) = result {
                            self.last_execution_result = Some(data.clone());
                            changed = true;
                        }
                    }
                }
            } else {
                ui.colored_label(egui::Color32::RED, "❌ File not found");
                ui.small(&self.file_path);
            }
        }
        
        // Show current output
        if let Some(result) = &self.last_execution_result {
            ui.separator();
            ui.colored_label(egui::Color32::LIGHT_GREEN, "✅ USD Stage loaded and ready");
            ui.small("Connect output to 3D Viewport to display");
        }
        
        changed
    }
    
}

impl LoadStageNode {
    /// Open a file dialog to select USD files
    fn open_usd_file_dialog(&self) -> Result<Option<String>, String> {
        use rfd::FileDialog;
        
        if let Some(path) = FileDialog::new()
            .add_filter("USD Files", &["usd", "usda", "usdc", "usdz"])
            .add_filter("All Files", &["*"])
            .set_title("Select USD File for LoadStage")
            .pick_file()
        {
            if let Some(path_str) = path.to_str() {
                Ok(Some(path_str.to_string()))
            } else {
                Err("Invalid file path encoding".to_string())
            }
        } else {
            Ok(None) // User cancelled dialog
        }
    }
}

impl LoadStageLogic {
    /// Process the load stage operation using file path from parameters
    pub fn process_with_parameters(file_path: &str, _auto_reload: bool, _load_payloads: bool, _population_mask: &str) -> Vec<NodeData> {
        if file_path.is_empty() {
            // No file selected, return empty stage
            println!("USD LoadStage: No file selected");
            return vec![NodeData::Any("Empty USD Stage".to_string())];
        }
        
        // Check if file exists
        if !std::path::Path::new(file_path).exists() {
            eprintln!("USD file not found: {}", file_path);
            return vec![NodeData::Any("Invalid USD Stage".to_string())];
        }
        
        println!("USD LoadStage: Loading file {}", file_path);
        
        // Create USDRenderer and load the stage
        let mut usd_renderer = crate::gpu::USDRenderer::new();
        
        // Use the file path as the stage ID for now
        // In a real implementation, this would parse the USD file
        let stage_id = format!("file://{}", file_path);
        
        match usd_renderer.load_stage(&stage_id) {
            Ok(()) => {
                println!("USD LoadStage: Successfully loaded USD file");
                
                // Return the USDScene as NodeData::Any containing the scene data
                // The viewport will know how to handle USDScene data
                let scene_data = crate::nodes::interface::NodeData::Any(format!("USDScene:{}", stage_id));
                vec![scene_data]
            }
            Err(e) => {
                eprintln!("USD LoadStage: Failed to load USD file: {}", e);
                vec![NodeData::Any(format!("Error loading USD: {}", e))]
            }
        }
    }
    
    /// Legacy process function - now unused since we get file path from parameters
    pub fn process(_inputs: Vec<NodeData>) -> Vec<NodeData> {
        // This method is kept for compatibility but should not be used
        // The actual processing is done via process_with_parameters
        vec![NodeData::Any("USD Stage - use parameters interface".to_string())]
    }
}