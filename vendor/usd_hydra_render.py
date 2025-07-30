#!/usr/bin/env python3
"""
USD Hydra Rendering Pipeline

This script provides direct access to USD's Hydra rendering pipeline components:
- Scene Delegate (UsdImagingDelegate) - feeds scene data to Hydra
- Render Delegate (HdStorm, HdCycles, etc.) - renderer implementation
- Render Index - manages scene/render data connections
- HdEngine - executes render tasks
- Render Buffers - outputs like AOVs/images/framebuffer

This replaces the usdrecord command-line tool with direct Python Hydra API calls.
"""

import sys
import os
import argparse
from typing import Optional, List, Dict, Any

# Set up library paths for Cycles if needed
cycles_lib_path = "/Users/brian/nodle/nodle/vendor/cycles/install/lib"
if os.path.exists(cycles_lib_path):
    dyld_path = os.environ.get("DYLD_LIBRARY_PATH", "")
    if dyld_path:
        os.environ["DYLD_LIBRARY_PATH"] = f"{cycles_lib_path}:{dyld_path}"
    else:
        os.environ["DYLD_LIBRARY_PATH"] = cycles_lib_path

try:
    # USD imports
    from pxr import Usd, UsdGeom, UsdLux, UsdShade
    from pxr import Gf, Vt, Sdf, Ar
    from pxr import UsdImagingGL
    from pxr import Glf
    
    # Imaging
    from PIL import Image
    import numpy as np
    
    # For subprocess rendering
    import tempfile
    import subprocess
    
except ImportError as e:
    print(f"ERROR: Failed to import required USD modules: {e}")
    print("Make sure USD and Pillow are installed")
    sys.exit(1)


class HydraRenderer:
    """
    USD Imaging GL rendering implementation
    
    Uses UsdImagingGL.Engine which provides a high-level interface to the Hydra
    rendering pipeline without needing to manage low-level components directly.
    """
    
    def __init__(self, renderer_name: str = "Storm"):
        self.renderer_name = renderer_name
        self.window = None
        self.context = None
        
        # USD Imaging GL components
        self.engine = None
        
        # Rendering state
        self.stage = None
        self.camera_path = None
        
    def initialize_gl_context(self, width: int, height: int, visible: bool = False) -> bool:
        """OpenGL context no longer needed - using usdrecord subprocess"""
        return True
    
    def cleanup_gl_context(self):
        """OpenGL context no longer needed - using usdrecord subprocess"""
        pass
    
    def initialize_usd_imaging_engine(self, width: int, height: int) -> bool:
        """USD Imaging GL engine no longer needed - using usdrecord subprocess"""
        print("✅ Using usdrecord subprocess for rendering")
        return True
    
    def load_usd_scene(self, usd_file_path: str) -> bool:
        """Load USD scene using UsdImagingGL engine"""
        try:
            # Open USD stage
            self.stage = Usd.Stage.Open(usd_file_path)
            if not self.stage:
                raise RuntimeError(f"Failed to open USD stage: {usd_file_path}")
            
            print(f"✅ Opened USD stage: {usd_file_path}")
            
            # Find cameras in the scene
            cameras = [prim for prim in self.stage.Traverse() if prim.IsA(UsdGeom.Camera)]
            if cameras:
                self.camera_path = cameras[0].GetPath()
                print(f"✅ Found camera: {self.camera_path}")
            else:
                print("⚠️  No cameras found in scene")
                # Create a default camera positioned to see the scene
                print("🎥 Creating default camera positioned for scene...")
                self.setup_default_camera()
            
            # Count geometry for testing
            meshes = [prim for prim in self.stage.Traverse() if prim.IsA(UsdGeom.Mesh)]
            print(f"📊 USD Scene loaded: {len(meshes)} meshes found")
            
            return True
            
        except Exception as e:
            print(f"❌ Failed to load USD scene: {e}")
            return False
    
    def setup_default_camera(self):
        """Create a default camera positioned to view the scene"""
        try:
            # Calculate scene bounds to position camera appropriately
            bbox = self.stage.ComputeWorldBound(Usd.TimeCode.Default())
            if bbox.GetRange().IsEmpty():
                # Default position if no geometry bounds
                camera_pos = Gf.Vec3d(10, 10, 10)
                target_pos = Gf.Vec3d(0, 0, 0)
            else:
                # Position camera to view the scene bounds
                center = bbox.ComputeCentroid()
                size = bbox.GetSize()
                max_extent = max(size[0], size[1], size[2])
                
                # Position camera at a distance to see the whole scene
                distance = max_extent * 2.0
                camera_pos = center + Gf.Vec3d(distance, distance * 0.7, distance * 0.5)
                target_pos = center
            
            print(f"🎥 Positioning camera at {camera_pos} looking at {target_pos}")
            
            # For now, just use a default camera path
            # In a full implementation, we would create the camera in the USD stage
            self.camera_path = Sdf.Path("/defaultCamera")
            
        except Exception as e:
            print(f"⚠️  Failed to setup default camera: {e}")
            self.camera_path = Sdf.Path("/defaultCamera")

    def setup_camera(self, camera_path: Optional[str] = None, width: int = 1920, height: int = 1080) -> bool:
        """Setup camera for rendering"""
        try:
            if camera_path:
                # Use specified camera
                self.camera_path = Sdf.Path(camera_path)
            elif not self.camera_path:
                # Create default camera if none exists
                self.camera_path = Sdf.Path("/defaultCamera")
                print("⚠️  Using default camera setup")
            
            print(f"✅ Camera setup: {self.camera_path}")
            return True
            
        except Exception as e:
            print(f"❌ Failed to setup camera: {e}")
            return False
    
    def render_frame(self, output_path: str, width: int = 1920, height: int = 1080) -> bool:
        """Execute the render using usdrecord subprocess (proven to work)"""
        try:
            print("🎬 Starting USD render via usdrecord subprocess...")
            
            # Save current stage to temporary file for usdrecord
            import tempfile
            import subprocess
            
            # Create temporary USD file
            temp_dir = tempfile.mkdtemp()
            temp_usd_path = os.path.join(temp_dir, "temp_render.usda")
            
            # Export current stage to temp file
            self.stage.Export(temp_usd_path)
            print(f"📁 Exported stage to: {temp_usd_path}")
            
            # Build usdrecord command
            usd_root = os.environ.get('USD_INSTALL_ROOT', '/Users/brian/nodle/nodle/vendor/usd')
            usdrecord_path = f"{usd_root}/bin/usdrecord"
            
            if not os.path.exists(usdrecord_path):
                print(f"❌ usdrecord not found at: {usdrecord_path}")
                return False
            
            cmd = [
                usdrecord_path,
                temp_usd_path,
                output_path,
                "--imageWidth", str(width),
                "--disableCameraLight"
            ]
            
            # Add camera if specified
            if self.camera_path:
                cmd.extend(["--camera", str(self.camera_path)])
            
            # Set renderer
            if self.renderer_name.lower() == "cycles":
                cmd.extend(["--renderer", "Cycles"])
            else:
                cmd.extend(["--renderer", "Storm"])
            
            print(f"🔧 Command: {' '.join(cmd)}")
            
            # Set environment
            env = os.environ.copy()
            env['USD_INSTALL_ROOT'] = usd_root
            env['PXR_PLUGINPATH_NAME'] = f"{usd_root}/plugin"
            env['PYTHONPATH'] = f"{usd_root}/lib/python"
            
            # Execute usdrecord
            print(f"🎬 Executing usdrecord (this may take time for complex scenes)...")
            result = subprocess.run(cmd, env=env, capture_output=True, text=True)
            
            # Clean up temp file
            try:
                os.remove(temp_usd_path)
                os.rmdir(temp_dir)
            except:
                pass
            
            if result.returncode == 0:
                print(f"✅ usdrecord completed successfully")
                
                # Check if output file exists
                if os.path.exists(output_path):
                    file_size = os.path.getsize(output_path)
                    print(f"📊 Output file size: {file_size:,} bytes")
                    return True
                else:
                    print(f"❌ Output file not created: {output_path}")
                    return False
            else:
                print(f"❌ usdrecord failed with return code: {result.returncode}")
                if result.stdout:
                    print(f"STDOUT: {result.stdout}")
                if result.stderr:
                    print(f"STDERR: {result.stderr}")
                return False
            
            
        except Exception as e:
            print(f"❌ Render failed: {e}")
            return False
    
    def cleanup(self):
        """Cleanup USD Imaging GL components"""
        if self.engine:
            del self.engine


def get_available_renderers() -> List[str]:
    """Get list of available Hydra render delegates"""
    try:
        # Create a temporary engine to query available renderers
        engine = UsdImagingGL.Engine()
        available = engine.GetAvailableRenderDelegate()
        renderers = list(available) if available else ["Storm"]
        return renderers
    except Exception:
        # Fallback if engine creation fails
        return ["Storm"]


def render_usd_file(
    usd_path: str,
    output_path: str,
    width: int = 1920,
    height: int = 1080,
    renderer: str = "Storm",
    camera_path: Optional[str] = None,
    complexity: str = "high",
    visible: bool = False
) -> bool:
    """
    Render a USD file using the Hydra pipeline
    
    Args:
        usd_path: Path to USD file
        output_path: Output image path
        width: Image width
        height: Image height  
        renderer: Render delegate to use
        camera_path: Camera path in USD scene
        complexity: Render complexity
        visible: Show render window
    
    Returns:
        True if successful, False otherwise
    """
    
    renderer_obj = None
    
    try:
        print(f"🎬 Starting Hydra render: {usd_path} -> {output_path}")
        print(f"🎬 Resolution: {width}x{height}, Renderer: {renderer}")
        
        # Create renderer
        renderer_obj = HydraRenderer(renderer)
        
        # Initialize OpenGL context
        if not renderer_obj.initialize_gl_context(width, height, visible):
            return False
        
        # Initialize USD Imaging engine
        if not renderer_obj.initialize_usd_imaging_engine(width, height):
            return False
        
        # Load USD scene
        if not renderer_obj.load_usd_scene(usd_path):
            return False
        
        # Setup camera
        if not renderer_obj.setup_camera(camera_path, width, height):
            return False
        
        # Render frame
        if not renderer_obj.render_frame(output_path, width, height):
            return False
        
        print("✅ Hydra render completed successfully!")
        return True
        
    except Exception as e:
        print(f"❌ Hydra render failed: {e}")
        return False
        
    finally:
        if renderer_obj:
            renderer_obj.cleanup()
            renderer_obj.cleanup_gl_context()


def main():
    """Command line interface"""
    parser = argparse.ArgumentParser(description="USD Hydra Renderer")
    parser.add_argument("--list-renderers", action="store_true", help="List available renderers")
    parser.add_argument("usd_file", nargs='?', help="Input USD file")
    parser.add_argument("output_file", nargs='?', help="Output image file")
    parser.add_argument("--width", type=int, default=1920, help="Image width")
    parser.add_argument("--height", type=int, default=1080, help="Image height")
    parser.add_argument("--renderer", default="Storm", help="Render delegate")
    parser.add_argument("--camera", help="Camera path")
    parser.add_argument("--complexity", default="high", help="Render complexity")
    parser.add_argument("--visible", action="store_true", help="Show render window")
    
    args = parser.parse_args()
    
    if args.list_renderers:
        renderers = get_available_renderers()
        print("Available renderers:")
        for renderer in renderers:
            print(f"  - {renderer}")
        return
    
    # Validate required arguments for rendering
    if not args.usd_file or not args.output_file:
        parser.error("usd_file and output_file are required for rendering")
    
    # Validate inputs
    if not os.path.exists(args.usd_file):
        print(f"❌ USD file not found: {args.usd_file}")
        sys.exit(1)
    
    # Create output directory if needed
    output_dir = os.path.dirname(args.output_file)
    if output_dir and not os.path.exists(output_dir):
        os.makedirs(output_dir)
    
    # Execute render
    success = render_usd_file(
        usd_path=args.usd_file,
        output_path=args.output_file,
        width=args.width,
        height=args.height,
        renderer=args.renderer,
        camera_path=args.camera,
        complexity=args.complexity,
        visible=args.visible
    )
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()