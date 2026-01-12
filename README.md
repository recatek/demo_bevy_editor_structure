# demo_bevy_editor_structure

A scaffold demo of how a potential Bevy code/editor integration might work. This is broken down into a few parts:

### lib_game

This is the game-as-a-plugin. All game-specific coding should be done here, and this will likely be the bulk of the code in a given project. This can have other libraries statically linked to it. Essentially, this is the top-level crate of where all the resources, components, and systems exist for "your" game. It is built as both a staticlib and Rust dylib (not cdylib).

### exe_game

This is the standalone executable host/shell binary for the "shipping" game. It doesn't contain much, if any, game logic -- that all lives in lib_game. This executable runs the game the way you'd play it, with all of the game's systems, components, and resources running for gameplay. The lib_game library is *statically* linked to this, and all of its type information can be accessed the way you'd access them in any other imported crate.

### exe_editor

This is the standalone executable host/shell binary for the game's editor. It does not run the game (for now). This is the tool for editing the game's assets. In order to edit the game's assets (scene files, etc.), it needs to know the game's type data. However, the game is also something we will be building and iterating on a lot while using the editor, so we would like to accelerate the reloading process. Because of this, the lib_game library is *dynamically* linked to this and its methods for accessing game type information work through the Bevy reflection type registry. The editor can, while running, unload and reload the lib_game dylib and update the reflected information it has about the game's components and resources. The process for this is documented in exe_editor's main.rs file.

### assets

The root assets directory. This is moved to be at the level of the cargo workspace rather than any particular crate. Currently bevy doesn't support this out of the box, so the root path has to be changed in DefaultPlugins. The assets directory contains a bevy.toml file, which could be used for data configuration and other project-level info.

# OS Support

This is currently built to work for Windows, since that's the OS I run day-to-day. Supporting Linux wouldn't require much work if someone wants to do it, mainly just renaming some ".dll" extensions.

# License


This library may be used under your choice of the Apache 2.0, MIT license, or Unlicense.
