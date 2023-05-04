# valve_maps

A parser for the `.map` file format used by Quake 1 & 2 as well as Half-Life 1,
implemented using the [nom](https://www.crates.io/crates/nom) parsing framework. It can
easily be integrated with other `nom` parsers.


### Setup
TrenchBroom:
- set the grid size to 16
- add the files in the trenchbroom folder to a new game folder in TrenchBroom
    - open settings
    - click the folder icon button on the bottom of the screen
    - create a folder called Bevy and copy the files

### Maps
Bevy:
- maps should be saved in the `assets` folder
- textures should be stored in `assets/textures`

TrenchBroom:
- create a new Bevy game map with type Valve map
- in settings set the Game Path to your `assets` folder
- be sure to add the `textures` folder in the Mods panel (may require restart)

```rs
commands.spawn(SceneBundle {
    scene: asset_server.load("test.map"),
    ..default()
});
```


Much of the code was sourced from the following repos:
[nomap](https://github.com/reslario/nomap)
[quarchitect](https://github.com/QodotPlugin/quarchitect/)