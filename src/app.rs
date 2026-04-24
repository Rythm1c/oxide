use super::camera::CameraMovement;
use super::scene::Scene;

use engine_core::{
    context::VkContext, 
    descriptor::GlobalDescriptorSet, 
    drawable::RenderObject, pipeline::{
        GraphicsPipeline, 
        GraphicsPipelineConfig, 
        PushConstants}, 
    renderer::Renderer
};

use geometry;

use std::error::Error;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

struct VulkanCore {
    renderer: Renderer,
    pipeline: GraphicsPipeline,
    globals : GlobalDescriptorSet,
    context : Arc<VkContext>,
}

impl VulkanCore {
    pub fn new(app_name: &str, window: &Window) -> Result<Self, Box<dyn Error>> {
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
        
        let cfg = GraphicsPipelineConfig::default()
            .vertex_shader("shaders/vert.spv")
            .fragment_shader("shaders/frag.spv")
            .cull_mode(ash::vk::CullModeFlags::BACK)
            .polygon_mode(ash::vk::PolygonMode::FILL)
            .descriptor_layouts(vec![globals.layout()])
            .push_constant_ranges(vec![PushConstants::push_range()]);

        let pipeline = GraphicsPipeline::create(&cfg, Arc::clone(&context))?;

        let renderer = Renderer::new(Arc::clone(&context));

        Ok(VulkanCore {
            globals,
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

#[derive(Default)]
struct App {
    window         : Option<Window>,
    vulkan_core    : Option<VulkanCore>,
    scene          : Option<Scene>,
    last_frame_time: Option<std::time::Instant>,
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

        // Create and setup the scene
        self.scene = Some(Scene::new());
        let cube = geometry::Geometry::new(geometry::GeometryType::Cube { size: 1.0, color: None });
        self.scene.as_mut().unwrap().add_object(RenderObject{
            vertex_buffer: cube.vertex_buffer(Arc::clone(&self.vulkan_core.as_ref().unwrap().context.device_ctx)).unwrap(),
            index_buffer: Some(cube.index_buffer(Arc::clone(&self.vulkan_core.as_ref().unwrap().context.device_ctx)).unwrap()),
            index_count: cube.index_count() as u32,
            vertex_count: cube.vertex_count() as u32,
            push_constants: PushConstants::identity(),
        });

    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                
                // Wait for GPU to finish all rendering before destroying geometry buffers
                if let Some(ref core) = self.vulkan_core {
                    unsafe { core.context.device().device_wait_idle().unwrap() };
                }
                
                self.scene.take();
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
                    if let Some(scene) = &mut self.scene {
                        scene.update(dt);

                        let frame = core.renderer.get_current_frame();

                        core.globals.flush(frame, 
                            &scene.camera_ubo(),
                            &scene.light_ubo())
                        .expect("failed to update globals");

                        match core
                            .renderer
                            .render(&core.pipeline, &core.globals, scene.objects())
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

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state       : ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let Some(scene) = &mut self.scene {
                    match key {
                        KeyCode::KeyW   => scene.move_camera(CameraMovement::Forward),
                        KeyCode::KeyS   => scene.move_camera(CameraMovement::Backward),
                        KeyCode::KeyA   => scene.move_camera(CameraMovement::Left),
                        KeyCode::KeyD   => scene.move_camera(CameraMovement::Right),
                        KeyCode::Escape => {
                            if let Some(ref core) = self.vulkan_core {
                                unsafe { core.context.device().device_wait_idle().unwrap() };
                            }
                            self.scene.take();
                            self.vulkan_core.take();
                            event_loop.exit();
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::Resized(_) => {
                // TODO: recreate swapchain on resize
            }
            _ => (),
        }
    }

    // Mouse look via raw device events (more reliable than cursor delta)
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id : DeviceId,
        event      : DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            if let Some(scene) = &mut self.scene {
                scene.rotate_camera(dx as f32, -dy as f32);
            }
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
