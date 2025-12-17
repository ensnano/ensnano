# TODO

## Stabilize `dev_iced`

- Fix text fields in organizer tree
- Fix text fields in revolution surface tab
- Click on organizer tree should select and reciprocally (select should highlight in red in organizer tree)
- Double on organizer tree click should teleport in 2d and 3d scenes
- Add serialization/deserialization tests to detect regressions based on typo fixes

## Bugs

- Increasing the left panel too much crashes ENSnano
- Crash when selecting "Ellipse" or "Two spheres" in "Revolution Surfaces" tab
- Clicking "Toggle split of flat scene" twice should do nothing
- Request fit doesn't work properly anymore, both in 2D and 3D
- Organizer Tree slider doesn't work with the mouse
- Movement with mouse wheel doesn't work on some systems (Axel @ 42), both in 2D and 3D

## Hide scientific WIP

- .ens file encryption
- Naive solution based on system time, hidden files or something
- Authentication once the app is ported to the web
- Possibility to inject an external private crate with more objects

## Scene (3D)

- Slider controlling camera focal length
- Movement is broken when both keyboard and mouse wheel translations (or zoom) are applied
- Fix rotation with `H`/`J`/`K`/`L` while dragging (copy code from `swing`)
- Cut plane (toggleable button next to grids)
- Isometric view

### Background (skybox)

- PNGs should have a choice of background too (including transparent)
- Infinite distance instead of hardcoded 500
- Any plain color instead of just white

### Outline

- Better implementation:
  - https://io7m.com/documents/outline-glsl/
  - https://gamedev.stackexchange.com/questions/68401/how-can-i-draw-outlines-around-3d-models
  - Simply use the normal?
- Give uniform instead of hardcoding
- Fix on transparent objects (create a separate pass?)

## Flat Scene (2D)

- Vector export (SVG?)
- Show size of the crossovers

## GUI

- Better looking tabs (and show active)
- `3D`, `2D` and `3D+2D` should be "radio buttons" with the active one shown like the selection 

### Help

- Separate menus (scene/flatscene/general)
- Toggleable with `?` icon (at least on scene and flatscene)

## User experience

- App icon
- Finish using `confy` parameters to load saved preferences (`AppStateParameters`)
- FPS counter

### Keyboard shortcuts

- Find and eliminate all duplicate keyboard shortcuts:
  - `H` is used for 3D rotation and to switch to the Helix selection mode
  - `K` is used for 3D rotation and to recolor staples
- Use either `LogicalKey` or `PhysicalKey` everywhere, don't mix and match

### Command-line arguments (`clap`)

- Filename
- Maximized/fullscreen/normal on startup
- GPU power preference (`wgpu::PowerPreference::HighPerformance`)

## Long-term

- Check and accept nix merge request
- Fix all GitLab issues
- Update `iced` to 0.13
- Port to the web with WASM
- Distribution of strand lengths (analysis tab)
- Triple-stranded DNA
- Local crossover optimization
- Animation timeline
- Publish `ensnano` on crates.io?
- README for collaborators (dependencies, crates graph, clippy rules...)

## Refactor

- Remove in-file modules:
  - `abscissa_converter`
  - `input_color`
  - `hue_column`
  - `light_sat_square`
  - `color_square`
  - `gostop`
  - `fog_kind`
- Rename structs with the same name in different crates, then remove all use aliases (`use ... as ...`)
- kebab-case -> snake_case for the crate directories
- More consistent styling:
  - Create some `rustfmt.toml` rules?
  - Merge imports?
  - `mod` then `use`
- Remove traits implemented only once:
  - `Data`
  - `AppState`
  - `MainState`
  - `ScaffoldSetter`
  - `Multiplexer`
- Remove enums with one variant -> struct or raw value:
  - `AppOperation`
  - `GridPositionBuilder`
  - `OverlayType`
  - `RotationWidgetOrientation`
  - `IterativeFrameAlgorithm`
- Replace `ensnano_iced/fonts/material_icons.rs` by SVG icons from `icondata` lib
- `build.rs` for shaders instead of manual compilation
- Merge `ensnano_organizer` and `ensnano_gui`
- Merge `ensnano_interactor` and `ensnano_utils`?
- Share more code between `ensnano_scene` and `ensnano_flatscene`:
  - e.g. `export_2d_png` and `export_3d_png` are pretty much the same
- Split `src/controller/quit.rs` in multiple file