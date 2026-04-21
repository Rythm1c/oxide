use super::camera::CameraMovement;

use engine_core::{
    context::VkContext,
    pipeline::{GraphicsPipeline, GraphicsPipelineConfig},
    renderer::Renderer,
};

use super::scene::Scene;

use std::error::Error;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

struct VulkanCore {
    context: Arc<VkContext>,
    pipeline: GraphicsPipeline,
    renderer: Renderer,
}

impl VulkanCore {
    pub fn new(app_name: &str, window: &Window) -> Result<Self, Box<dyn Error>> {
        let context = Arc::new(VkContext::new(
            app_name,
            window,
            window.inner_size().width,
            window.inner_size().height,
        )?);

        let cfg = GraphicsPipelineConfig::default()
            .vertex_shader("shaders/vert.spv")
            .fragment_shader("shaders/frag.spv");

        let pipeline = GraphicsPipeline::create(&cfg, Arc::clone(&context))?;

        let renderer = Renderer::new(Arc::clone(&context));

        Ok(VulkanCore {
            context,
            pipeline,
            renderer,
        })
    }
}

impl Drop for VulkanCore {
    fn drop(&mut self) {
        unsafe { self.context.device().device_wait_idle().unwrap() };
    }
}

#[derive(Default)]
struct App {
    window: Option<Window>,
    vulkan_core: Option<VulkanCore>,
    scene: Option<Scene>,
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
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
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

                        match core.renderer.render(&core.pipeline, scene.objects()) {
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
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let Some(scene) = &mut self.scene {
                    match key {
                        KeyCode::KeyW => scene.move_camera(CameraMovement::Forward),
                        KeyCode::KeyS => scene.move_camera(CameraMovement::Backward),
                        KeyCode::KeyA => scene.move_camera(CameraMovement::Left),
                        KeyCode::KeyD => scene.move_camera(CameraMovement::Right),
                        KeyCode::Escape => {
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
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            if let Some(scene) = &mut self.scene {
                scene.rotate_camera(dx as f32, dy as f32);
            }
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);
    
    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
