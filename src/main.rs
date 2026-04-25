use anyhow;
use std::sync::Arc;

mod app;
mod camera;
mod object;
mod scene;

use app::App;
use scene::Scene;

fn main() -> anyhow::Result<()> {
    // Create the app with the scene reference
    let mut app = App::new();
    // Create the scene
    let scene = Scene::new();
    scene.add_object(object::Object::new(geometry::Shape::Cube {
        size: 1.0,
        color: None,
    }));
    app.set_scene(Arc::new(scene));
    // Run the application
    app::run(app)?;

    Ok(())
}
