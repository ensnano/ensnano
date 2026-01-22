# TODO

## Bugs

- Moving/scrolling and rotating at the same time make the camera go crazy
- Increasing the left panel too much crashes ENSnano
- Clicking "Toggle split of flat scene" twice should go back to the same state
- Request fit doesn't work properly anymore, both in 2D and 3D
- Organizer Tree slider doesn't work with the mouse
- Distance fog is broken ([NS message](https://discord.com/channels/689053746604670995/1419689469472411691/1459186505888170035))

## Bugs that should be fixed

- Crash when selecting "Ellipse" or "Two spheres" in "Revolution Surfaces" tab
- Click on organizer tree should select and reciprocally (select should highlight in red in organizer tree)
- oxDNA export is broken ([NS message](https://discord.com/channels/689053746604670995/1420320954185416745/1459185179594850396))

## Hide scientific WIP

- .ens file encryption
- Naive solution based on system time, hidden files or something
- Authentication once the app is ported to the web
- Possibility to inject an external private crate with more objects

## Scene (3D)

- Slider controlling camera focal length
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
- Slider widget to control the width

## Flat Scene (2D)

- Vector export (SVG?)
- When moving a crossover, show the length of the neighbor too
- Show size of the crossovers

## GUI

- Double click on organizer tree should teleport in 2D and 3D scenes
- Better looking tabs (and show active)
- `3D`, `2D` and `3D+2D` should be "radio buttons" with the active one shown like the selection 

### Help

- Separate menus (scene/flatscene/general)
- Toggleable with `?` icon (at least on scene and flatscene)

## User experience

- App icon
- Finish using `confy` parameters to load saved preferences (`AppStateParameters`)
- FPS counter

## Inputs

- Shortcuts should be the same in the 2D and 3D views (e.g. arrows should translate in 2D)
- Find and eliminate all duplicate keyboard shortcuts:
  - `H` is used for 3D rotation and to switch to the Helix selection mode
  - `K` is used for 3D rotation and to recolor staples
- Use either `LogicalKey` or `PhysicalKey` everywhere, don't mix and match

### Command-line arguments (`clap`)

- Filename
- Maximized/fullscreen/normal on startup
- GPU power preference (`wgpu::PowerPreference::HighPerformance`)

## Long-term

- Add serialization/deserialization tests to detect regressions based on typo fixes
- Check and accept nix merge request
- Fix all GitLab issues
- Update `iced` to 0.14
  - Update `iced_aw` in parallel
  - Replace all occurrences of `text(format!(...))` with the macro `text!(...)`
  - Remove `Command::none()` everywhere
- Convert shaders to WGSL
  - `build.rs` for automatic precompilation
  - Remove shader rules in Makefile
- Port to the web with WASM
- Distribution of strand lengths (analysis tab)
- Triple-stranded DNA
- Split 2D view when clicking on xover in 3D view whose nucleotides are far apart
- Local crossover optimization
- Animation timeline
- Publish `ensnano` on crates.io?
- README for collaborators (dependencies, crates graph, clippy rules...)

## Refactor

- Merge `MainState` and `MainStateView` structs?
- Remove in-file modules:
  - `abscissa_converter`
  - `input_color`
  - `hue_column`
  - `light_sat_square`
  - `color_square`
  - `gostop`
  - `fog_kind`
- Clean `AbscissaConverter as AbscissaConverter_`
- kebab-case -> snake_case for the crate directories
- More consistent styling:
  - Create some `rustfmt.toml` rules?
  - Merge imports?
  - `mod` then `use`
- Remove traits implemented only once:
  - `AdditionalStructure` (ensnano_design)
  - `RawDrawer`
  - `DesignElementKeySelection`
  - `MultiplexerExt`
  - `GridInstanceExt`
- Remove enums with one variant -> struct or raw value:
  - `AppOperation`
  - `GridPositionBuilder`
  - `OverlayType`
  - `RotationWidgetOrientation`
  - `IterativeFrameAlgorithm`
- Replace `ensnano_gui/fonts/material_icons.rs` by SVG icons from `icondata` lib
- Share more code between `ensnano_scene` and `ensnano_flatscene`:
  - e.g. `export_2d_png` and `export_3d_png` are pretty much the same
- Remove all `println!` and use the `log` crate everywhere
- Split `src/controller/quit.rs` in multiple files
- Rename one of the color_picker files (or merge)
