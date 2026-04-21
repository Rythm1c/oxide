use super::camera::Camera;
/* use super::cube::Cube;
use super::triangle::Triangle; */

use engine_core::{
    context::VkContext,
    pipeline::GraphicsPipeline,
    renderer::{RenderObject, Renderer, Scene},
};
use math::vec3::vec3;

use std::error::Error;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
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

        let pipeline = GraphicsPipeline::default()
            .create_renderpass(&context)
            .create_framebuffers(&context)
            .create_layout(&context)
            .create_shader_modules(&context, "shaders/vert.spv", "shaders/frag.spv")
            .build(&context)?;

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
        unsafe { self.context.device.device_wait_idle().unwrap() };
        self.pipeline.destroy(&self.context);
    }
}

#[derive(Default)]
struct App {
    window: Option<Window>,
    vulkan_core: Option<VulkanCore>,
    camera: Option<Camera>, //scene: Option<Scene>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        self.window
            .as_ref()
            .unwrap()
            .set_title("Vulkan Application");

        self.vulkan_core =
            Some(VulkanCore::new("Vulkan App", self.window.as_ref().unwrap()).unwrap());

        // Create and setup the scene
        let mut scene = Scene::new();
        if let Some(core) = &self.vulkan_core {
        /*     let triangle = Triangle::new(&core.context);
            let render_object = RenderObject {
                vertex_buffer: triangle.vertex_buffer.buffer,
                index_buffer: Some(triangle.index_buffer.buffer),
                index_count: triangle.index_buffer.data.len() as u32,
                vertex_count: triangle.vertex_buffer.data.len() as u32,
            };
            scene.add_object(render_object); */

            //let cube = Cube::new(&core.context, vec3(0.0, 2.0, 3.0), 1.0, vec3(1.0, 1.0, 1.0));
            /* let cube_render_object = RenderObject {
                vertex_buffer: cube.vertex_buffer.buffer,
                index_buffer: Some(cube.index_buffer.buffer),
                index_count: cube.index_buffer.data.len() as u32,
                vertex_count: cube.vertex_buffer.data.len() as u32,
            }; */
        }

        if let Some(core) = &mut self.vulkan_core {
            core.renderer.set_scene(scene);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
                if let Some(core) = &mut self.vulkan_core {
                    core.renderer.render(&core.pipeline).unwrap();
                }
            }
            _ => (),
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
