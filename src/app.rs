use super::scene::Scene;

use engine_core::{
    context::VkContext, 
    descriptor::{
        GlobalDescriptorSet,
        MaterialAllocator
        }, 
    pipeline::{
        GraphicsPipeline, 
        GraphicsPipelineConfig, 
        PushConstants
    }, 
    renderer::Renderer
};

use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

struct VulkanCore {
    renderer          : Renderer,
    pipeline          : GraphicsPipeline,
    globals           : GlobalDescriptorSet,
    material_allocator: MaterialAllocator,
    context           : Arc<VkContext>,
}

impl VulkanCore {
    pub fn new(app_name: &str, window: &Window) -> anyhow::Result<Self> {
        let context = Arc::new(VkContext::new(
            app_name,
            window,
            window.inner_size().width,
            window.inner_size().height,
        )?);

        let globals = GlobalDescriptorSet::new(
            Arc::clone(&context.device_ctx),
            Renderer::MAX_FRAMES_IN_FLIGHT,
        )?;

        //100 objects for now
        let material_allocator = 
            MaterialAllocator::new(Arc::clone(&context.device_ctx), 100)?;
        
        let cfg = GraphicsPipelineConfig::default()
            .vertex_shader("shaders/vert.spv")
            .fragment_shader("shaders/frag.spv")
            .cull_mode(ash::vk::CullModeFlags::BACK)
            .polygon_mode(ash::vk::PolygonMode::FILL)
            .descriptor_layouts(vec![globals.layout(), material_allocator.layout()])
            .push_constant_ranges(vec![PushConstants::push_range()]);

        let pipeline = GraphicsPipeline::create(&cfg, Arc::clone(&context))?;

        let renderer = Renderer::new(Arc::clone(&context));

        

        Ok(VulkanCore {
            globals,
            material_allocator,
            context,
            pipeline,
            renderer,
            
        })
    }
}

impl Drop for VulkanCore {
    fn drop(&mut self) {
        unsafe {
            self.context.device().device_wait_idle().unwrap();
        };
    }
}

/// Application state managing Vulkan rendering and scene updates.
pub struct App {
    window           : Option<Window>,
    vulkan_core      : Option<VulkanCore>,
    scene            : Option<Arc<Scene>>,
    last_frame_time  : Option<Instant>,
    is_mouse_dragging: bool,
    last_mouse_pos   : (f64, f64),
}

impl App {
    /// Creates a new App with a reference to the scene.
    pub fn new() -> Self {
        Self {
            window           : None,
            vulkan_core      : None,
            scene            : None,
            last_frame_time  : None,
            is_mouse_dragging: false,
            last_mouse_pos   : (0.0, 0.0),
        }
    }

    pub fn set_scene(&mut self, scene: Arc<Scene>) {
        self.scene = Some(scene);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Enceladus")
                        .with_inner_size(winit::dpi::LogicalSize::new(800u32, 600u32)),
                )
                .unwrap(),
        );

        self.vulkan_core =
            Some(VulkanCore::new("Vulkan App", self.window.as_ref().unwrap()).unwrap());

        // Upload all scene objects to GPU
        if let Some(core) = &mut self.vulkan_core {
            let device_ctx = Arc::clone(&core.context.device_ctx);
            if let Some(scene) = &self.scene {
                if let Err(e) = scene.upload_all_objects(device_ctx, &mut core.material_allocator) {
                    eprintln!("Failed to upload scene objects: {}", e);
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                
                // Wait for GPU to finish all rendering before destroying geometry buffers
                if let Some(ref core) = self.vulkan_core {
                    unsafe { core.context.device().device_wait_idle().unwrap() };
                }
                self.scene.take(); // Drop scene and its objects before Vulkan context
                self.vulkan_core.take();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = self
                    .last_frame_time
                    .map(|t| now.duration_since(t).as_secs_f32())
                    .unwrap_or(1.0 / 60.0);
                self.last_frame_time = Some(now);

                if let Some(core) = &mut self.vulkan_core {
                    if let Some(scene) = &self.scene {
                        scene.update(dt);
                        let frame = core.renderer.get_current_frame();

                        core.globals.flush(frame, 
                            &scene.camera_ubo(),
                            &scene.light_ubo())
                        .expect("failed to update globals");

                        match core
                            .renderer
                            .render(&core.pipeline, &core.globals, &scene.render_objects())
                        {
                            Ok(_) => {}
                            Err(e) => eprintln!("Render error: {e}"),
                        }
                    }
                }

                // In Poll mode we must explicitly request the next redraw
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(scene) = &self.scene {
                        scene.handle_keyboard(
                            code,
                            event.state == winit::event::ElementState::Pressed,
                        );
                    }
                }
            }

            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    match state {
                        ElementState::Pressed  => self.is_mouse_dragging = true,
                        ElementState::Released => self.is_mouse_dragging = false,
                    }
                }
            }

            winit::event::WindowEvent::CursorMoved { position, .. } => {
                if self.is_mouse_dragging {
                    let dx = (position.x - self.last_mouse_pos.0) as f32;
                    let dy = (position.y - self.last_mouse_pos.1) as f32;

                    if let Some(scene) = &self.scene {
                        scene.rotate_camera(dx, dy);
                    }
                }
                self.last_mouse_pos = (position.x, position.y);
            }
            WindowEvent::Resized(_) => {
                // TODO: recreate swapchain on resize
            }
            _ => (),
        }
    }

}

/// Runs the application event loop.
pub fn run(mut app: App) -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)?;
    Ok(())
}
