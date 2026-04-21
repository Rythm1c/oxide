use std::error::Error;

mod app;
mod camera;
mod cube;
mod scene;
mod triangle;

fn main() -> Result<(), Box<dyn Error>> {
    app::run()?;
    Ok(())
}
