# OXIDE

A 3D renderer that uses the vulkan api 
Nothing too fancy. Just a hobby project i've been working on in my spare time for my love of computer graphics.
I used rust to learn more about the language(i also like abit of pain).

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
### Text/Books
- gabor szauer - hands on c++ game animation programming pack 
- Ian Millington - GAME PHYSICS ENGINE DEVELOPMENT(second edition)

## Features inlcude:
- [x] FPS style camera
- [x] directional lighting
- [x] Physical based rendering
- [ ] Multiple point lights
- [ ] shadow-map
- [ ] GUI
- [ ] sky box
- [ ] transparency
- [ ] physics
- [ ] animations


## samples
sample render on linux
![Alt text](./images/sample_1.png "Hello Triangle")

sample render on windows
![Alt text](./images/sample_2.png "eclipse")

sample PBR render(widows)
![Alt text](./images/sample_3.png "pbr sample")
