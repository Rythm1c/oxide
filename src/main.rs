use anyhow;
use std::sync::Arc;

mod app;
mod camera;
mod object;
mod scene;

use app::App;
use scene::Scene;

use math::vec3::Vec3;

use physics::{collider::ColliderType, rigidbody::RigidBody};

use geometry::Shape;

use crate::object::Material;

fn main() -> anyhow::Result<()> {
    // Create the app with the scene reference
    let mut app = App::new();
    // Create the scene
    let scene = Scene::new();

    scene.add_object(
        Material::stone(false, 8.0, 0.4),
        Shape::UVSphere {
            radius: 0.6,
            segments: 40,
            rings: 40,
            color: Some([0.3, 0.6, 0.7]),
        },
        RigidBody::default()
            .collider_type(ColliderType::Sphere { radius: 0.6 })// same radius as Shape struct
            .position(Vec3::new(0.0, 10.0, 0.0))
            .mass(20.0), // using  kilograms
    );

    scene.add_object(
        Material::polished(true, 8.0, 0.4),
        Shape::UVSphere {
            radius: 1.0,
            segments: 40,
            rings: 40,
            color: None,
        },
        RigidBody::default()
            .collider_type(ColliderType::Sphere { radius: 1.0 })
            .position(Vec3::new(3.0, 10.0, 0.0))
            .mass(30.0),
    );

    scene.add_object(
        Material::rubber(false, 10.0, 0.4),
        Shape::CubeSphere {
            radius: 0.8,
            subdivisions: 50,
            color: None,
        },
        RigidBody::default()
            .collider_type(ColliderType::Sphere { radius: 0.8 })
            .position(Vec3::new(-3.0, 10.0, 0.0))
            .mass(30.0),
    );

    // floor
    scene.add_object(
        Material::polished(true, 600.0, 0.01),
        Shape::CubeSphere {
            radius: 1000.0,
            subdivisions: 400,
            color: Some([1.0; 3]),
        },
        RigidBody::default()
            .collider_type(ColliderType::Sphere { radius: 1000.0 })
            .position(Vec3::new(0.0, -1002.0, 0.0))
            .mass(f32::INFINITY),
    );

    app.set_scene(Arc::new(scene));
    // Run the application
    app::run(app)?;

    Ok(())
}
