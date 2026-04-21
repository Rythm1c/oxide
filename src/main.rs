use anyhow;

mod app;
mod camera;
mod scene;

fn main() -> anyhow::Result<()> {
    app::run()?;
    Ok(())
}
