use anyhow;
use std::sync::Arc;

mod app;
mod camera;
mod object;
mod scene;

use app::App;
use scene::Scene;

use math::quaternion::Quat;
use math::vec3::Vec3;

use crate::object::Material;

fn main() -> anyhow::Result<()> {
    // Create the app with the scene reference
    let mut app = App::new();
    // Create the scene
    let scene = Scene::new();
    scene.add_object(
        object::Object::new(
            geometry::Shape::Cube {
                size: 1.0,
                color: None,
            },
            Material::default(),
        ),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
        Quat::ZERO,
    );

    scene.add_object(
        object::Object::new(
            geometry::Shape::UVSphere {
                radius: 0.5,
                segments: 40,
                rings: 40,
                color: None,
            },
            Material::stone(false, 0.0, 0.0),
        ),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
        Quat::ZERO,
    );

    scene.add_object(
        object::Object::new(
            geometry::Shape::CubeSphere {
                radius: 2.0,
                subdivisions: 50,
                color: None,
            },
            Material::rubber(true, 8.0, 0.0),
        ),
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
        Quat::ZERO,
    );
    app.set_scene(Arc::new(scene));
    // Run the application
    app::run(app)?;

    Ok(())
}
