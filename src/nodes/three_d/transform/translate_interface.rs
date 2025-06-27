//! Translate transform node with interface panel for parameter control

use egui::Color32;
use crate::nodes::{
    Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition,
    NodeInterfacePanel, InterfaceParameter, NodeData, SceneData, GeometryData,
};
use crate::{interface_float, interface_vector3, interface_enum};

/// Transform node that translates (moves) geometry with interface panel
#[derive(Debug, Clone)]
pub struct TranslateNodeWithInterface {
    /// Translation vector (X, Y, Z)
    pub translation: [f32; 3],
    /// Whether to use absolute or relative translation
    pub mode: TranslateMode,
    /// Whether to apply to individual objects or entire scene
    pub apply_to: ApplyTo,
    /// Whether to use interface values when no vector input is connected
    pub use_interface_default: bool,
}

#[derive(Debug, Clone)]
pub enum TranslateMode {
    Absolute,  // Set position to exact coordinates
    Relative,  // Add to current position
}

#[derive(Debug, Clone)]
pub enum ApplyTo {
    EntireScene,     // Apply to all geometry in scene
    IndividualParts, // Apply to each geometry piece separately
}

impl Default for TranslateNodeWithInterface {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            mode: TranslateMode::Relative,
            apply_to: ApplyTo::EntireScene,
            use_interface_default: true,
        }
    }
}

impl TranslateNodeWithInterface {
    /// Apply translation to geometry data
    fn apply_translation(&self, mut geometry: GeometryData, translation: [f32; 3]) -> GeometryData {
        // Apply translation to all vertices
        for vertex in &mut geometry.vertices {
            match self.mode {
                TranslateMode::Absolute => {
                    vertex[0] = translation[0];
                    vertex[1] = translation[1];
                    vertex[2] = translation[2];
                }
                TranslateMode::Relative => {
                    vertex[0] += translation[0];
                    vertex[1] += translation[1];
                    vertex[2] += translation[2];
                }
            }
        }
        
        geometry
    }
    
    /// Apply translation to scene data
    fn apply_scene_translation(&self, mut scene: SceneData, translation: [f32; 3]) -> SceneData {
        match self.apply_to {
            ApplyTo::EntireScene => {
                // Apply same translation to all geometry
                for geometry in &mut scene.geometry {
                    *geometry = self.apply_translation(geometry.clone(), translation);
                }
            }
            ApplyTo::IndividualParts => {
                // Could apply different logic here, for now same as entire scene
                for geometry in &mut scene.geometry {
                    *geometry = self.apply_translation(geometry.clone(), translation);
                }
            }
        }
        
        // Update transforms in scene if available
        // For now, just return the modified scene
        scene
    }
}

impl NodeFactory for TranslateNodeWithInterface {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_TranslateInterface",
            display_name: "Translate (Interface)",
            category: NodeCategory::new(&["3D", "Transform"]),
            description: "Translates (moves) geometry with interface panel controls",
            color: Color32::from_rgb(150, 100, 200), // Purple tint for transforms
            inputs: vec![
                PortDefinition::required("Input", DataType::Any)
                    .with_description("Scene or geometry data to transform"),
                PortDefinition::optional("Vector", DataType::Vector3)
                    .with_description("Optional translation vector (overrides interface values)"),
            ],
            outputs: vec![
                PortDefinition::required("Output", DataType::Any)
                    .with_description("Transformed scene or geometry data"),
            ],
        }
    }
}

impl NodeInterfacePanel for TranslateNodeWithInterface {
    fn get_parameters(&self) -> Vec<(&'static str, InterfaceParameter)> {
        vec![
            ("Translation", interface_vector3!(
                self.translation[0], 
                self.translation[1], 
                self.translation[2]
            )),
            ("Mode", interface_enum!(
                match self.mode {
                    TranslateMode::Absolute => 0,
                    TranslateMode::Relative => 1,
                },
                "Absolute", "Relative"
            )),
            ("Apply To", interface_enum!(
                match self.apply_to {
                    ApplyTo::EntireScene => 0,
                    ApplyTo::IndividualParts => 1,
                },
                "Entire Scene", "Individual Parts"
            )),
            ("Use Interface Default", InterfaceParameter::Boolean { 
                value: self.use_interface_default 
            }),
        ]
    }
    
    fn set_parameters(&mut self, parameters: Vec<(&'static str, InterfaceParameter)>) {
        for (name, param) in parameters {
            match name {
                "Translation" => {
                    if let Some(vec) = param.get_vector3() {
                        self.translation = vec;
                    }
                }
                "Mode" => {
                    if let InterfaceParameter::Enum { value, .. } = param {
                        self.mode = match value {
                            0 => TranslateMode::Absolute,
                            1 => TranslateMode::Relative,
                            _ => TranslateMode::Relative,
                        };
                    }
                }
                "Apply To" => {
                    if let InterfaceParameter::Enum { value, .. } = param {
                        self.apply_to = match value {
                            0 => ApplyTo::EntireScene,
                            1 => ApplyTo::IndividualParts,
                            _ => ApplyTo::EntireScene,
                        };
                    }
                }
                "Use Interface Default" => {
                    if let Some(val) = param.get_bool() {
                        self.use_interface_default = val;
                    }
                }
                _ => {}
            }
        }
    }
    
    fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        if inputs.is_empty() {
            return vec![];
        }
        
        // Determine translation vector
        let translation = if inputs.len() > 1 {
            // Use vector input if provided
            match &inputs[1] {
                NodeData::Vector3(vec) => *vec,
                _ => self.translation, // Fallback to interface
            }
        } else if self.use_interface_default {
            self.translation
        } else {
            [0.0, 0.0, 0.0] // No translation
        };
        
        // Apply translation based on input type
        match &inputs[0] {
            NodeData::Scene(scene) => {
                let transformed_scene = self.apply_scene_translation(scene.clone(), translation);
                vec![NodeData::Scene(transformed_scene)]
            }
            NodeData::Geometry(geometry) => {
                let transformed_geometry = self.apply_translation(geometry.clone(), translation);
                vec![NodeData::Geometry(transformed_geometry)]
            }
            other => {
                // Pass through other data types unchanged
                vec![other.clone()]
            }
        }
    }
    
    fn panel_title(&self) -> String {
        format!("Translate Parameters")
    }
    
    fn render_custom_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        
        ui.label("Quick Translation Presets:");
        ui.horizontal(|ui| {
            if ui.button("Reset").clicked() {
                self.translation = [0.0, 0.0, 0.0];
                changed = true;
            }
            if ui.button("Up 1").clicked() {
                self.translation[1] += 1.0;
                changed = true;
            }
            if ui.button("Down 1").clicked() {
                self.translation[1] -= 1.0;
                changed = true;
            }
        });
        
        ui.horizontal(|ui| {
            if ui.button("Left 1").clicked() {
                self.translation[0] -= 1.0;
                changed = true;
            }
            if ui.button("Right 1").clicked() {
                self.translation[0] += 1.0;
                changed = true;
            }
            if ui.button("Forward 1").clicked() {
                self.translation[2] += 1.0;
                changed = true;
            }
            if ui.button("Back 1").clicked() {
                self.translation[2] -= 1.0;
                changed = true;
            }
        });
        
        ui.separator();
        
        ui.label("Coordinate System:");
        ui.horizontal(|ui| {
            ui.label("X: Left(-) / Right(+)");
        });
        ui.horizontal(|ui| {
            ui.label("Y: Down(-) / Up(+)");
        });
        ui.horizontal(|ui| {
            ui.label("Z: Back(-) / Forward(+)");
        });
        
        changed
    }
}