//! Test USD functionality

use crate::nodes::Node;
use super::{USDCreateStage, USDSphere, USDCube, with_usd_engine};
use egui::Pos2;

/// Test USD basic functionality
pub fn test_usd_operations() {
    println!("=== Testing USD Operations ===");
    
    // Test 1: Create a stage
    let create_stage_node = Node::new(1, "Test Create Stage", Pos2::new(100.0, 100.0));
    let stage_id = match USDCreateStage::execute(&create_stage_node) {
        Ok(stage_id) => {
            println!("✓ Successfully created stage: {}", stage_id);
            stage_id
        }
        Err(e) => {
            println!("✗ Failed to create stage: {}", e);
            return;
        }
    };
    
    // Now test creating primitives in the same stage
    with_usd_engine(|engine| {
        // Test 2: Create a sphere in the correct stage  
        match engine.create_sphere(&stage_id, "/sphere_test", 1.0) {
            Ok(prim) => {
                println!("✓ Successfully created sphere: {} in stage {}", prim.path, prim.stage_id);
            }
            Err(e) => {
                println!("✗ Failed to create sphere: {}", e);
            }
        }
        
        // Test 3: Create a cube in the correct stage
        match engine.create_cube(&stage_id, "/cube_test", 1.0) {
            Ok(prim) => {
                println!("✓ Successfully created cube: {} in stage {}", prim.path, prim.stage_id);
            }
            Err(e) => {
                println!("✗ Failed to create cube: {}", e);
            }
        }
        
        // Test 4: Create a camera in the correct stage
        match engine.create_camera(&stage_id, "/main_camera", 50.0, 0.1, 1000.0) {
            Ok(prim) => {
                println!("✓ Successfully created camera: {} in stage {}", prim.path, prim.stage_id);
            }
            Err(e) => {
                println!("✗ Failed to create camera: {}", e);
            }
        }
        
        // Test 5: Create lights in the correct stage
        match engine.create_distant_light(&stage_id, "/sun_light", 1.0, 0.53) {
            Ok(prim) => {
                println!("✓ Successfully created distant light: {} in stage {}", prim.path, prim.stage_id);
            }
            Err(e) => {
                println!("✗ Failed to create distant light: {}", e);
            }
        }
        
        match engine.create_sphere_light(&stage_id, "/fill_light", 0.5, 2.0) {
            Ok(prim) => {
                println!("✓ Successfully created sphere light: {} in stage {}", prim.path, prim.stage_id);
            }
            Err(e) => {
                println!("✗ Failed to create sphere light: {}", e);
            }
        }
        
        // Test 6: Create materials and shaders
        match engine.create_material(&stage_id, "/materials/pbr_mat") {
            Ok(prim) => {
                println!("✓ Successfully created material: {} in stage {}", prim.path, prim.stage_id);
            }
            Err(e) => {
                println!("✗ Failed to create material: {}", e);
            }
        }
        
        match engine.create_preview_surface(&stage_id, "/shaders/pbr_surface", [0.7, 0.7, 0.9], 0.1, 0.3, 0.8) {
            Ok(prim) => {
                println!("✓ Successfully created preview surface: {} in stage {}", prim.path, prim.stage_id);
            }
            Err(e) => {
                println!("✗ Failed to create preview surface: {}", e);
            }
        }
        
        match engine.create_texture(&stage_id, "/textures/diffuse_tex", "textures/metal_diffuse.jpg") {
            Ok(prim) => {
                println!("✓ Successfully created texture: {} in stage {}", prim.path, prim.stage_id);
            }
            Err(e) => {
                println!("✗ Failed to create texture: {}", e);
            }
        }
        
        // Test 7: Render the complete scene
        match engine.render_stage(&stage_id, "main_viewport", "/main_camera", 1920, 1080) {
            Ok(render_info) => {
                println!("🎬 Successfully rendered scene: {}", render_info);
            }
            Err(e) => {
                println!("✗ Failed to render scene: {}", e);
            }
        }
        
        // Test 8: Test layer composition
        match engine.add_sublayer(&stage_id, "layers/animation.usda", 24.0) {
            Ok(info) => {
                println!("✓ Successfully added sublayer: {}", info);
            }
            Err(e) => {
                println!("✗ Failed to add sublayer: {}", e);
            }
        }
        
        match engine.add_reference(&stage_id, "/references/character", "assets/hero_character.usda", Some("/Hero")) {
            Ok(info) => {
                println!("✓ Successfully added reference: {}", info);
            }
            Err(e) => {
                println!("✗ Failed to add reference: {}", e);
            }
        }
        
        match engine.add_payload(&stage_id, "/payloads/environment", "assets/large_environment.usda", Some("/Environment")) {
            Ok(info) => {
                println!("✓ Successfully added payload: {}", info);
            }
            Err(e) => {
                println!("✗ Failed to add payload: {}", e);
            }
        }
        
        // Test 9: List final scene composition
        println!("📋 Stages: {:?}", engine.list_stages());
        println!("📋 Prims in {}: {:?}", stage_id, engine.list_prims(&stage_id));
    });
    
    println!("=== USD Test Complete ===");
}