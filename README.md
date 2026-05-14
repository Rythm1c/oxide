# OXIDE

A 3D renderer that uses the vulkan api 
Nothing too fancy. Just a hobby project i've been working on in my spare time for my love of computer graphics.
I used rust to learn more about the language(i also like abit of pain).

end goal is to have a decent physics engine.

dependencies are in the .toml file.

## Project set up
if you dont have cargo installed in your system then go to https://rustup.rs and follow the guide to install rust on your system.

once cargo is installed, clone the repo to a directory of your choice using the following command

```
git clone "https://github.com/Rythm1c/repo_name.git"
```
once the project is cloned open it in an editor of your choice and run using

```
cargo run
```
you should see a sample scene

## learning resources include:
### online 
- https://vulkan-tutorial.com
- https://www.songho.ca/
- https://learnopengl.com/
- [game physics in one weekend series - Gregory Hodges](https://gamephysicsweekend.github.io/)
### Text/Books
- gabor szauer - hands on c++ game animation programming pack 
- Ian Millington - GAME PHYSICS ENGINE DEVELOPMENT(second edition)

## Features inlcude:
- [x] FPS style camera
- [x] directional lighting
- [x] Physical based rendering
- [x] shadow-map(PCF)
- [x] Rigid-body physics(work in progress)
- [ ] soft body physics
- [ ] Multiple point lights
- [ ] GUI
- [ ] screen capture
- [ ] sky box
- [ ] transparency
- [ ] 3D model support
- [ ] animations




## samples
sample render on linux
![Alt text](./images/sample_1.png "Hello Triangle")

sample PBR render(widows)
![Alt text](./images/sample_2.png "pbr sample")

sample with shadow map
![Alt text](./images/sample_3.png "shadow map sample")
