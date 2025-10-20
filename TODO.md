# TODO

## Long-term

- Stabilize `dev_iced` and merge on `dev_private`
- Update `iced` to 0.13
- Port to the web with WASM
- Distribution of strand lengths (analysis tab)
- Triple-stranded DNA
- Local crossover optimization
- Animation timeline
- Publish `ensnano` on crates.io?
- Hide scientific WIP, either:
  - Authentification once the app is ported to the web
  - Possibility to inject an external private crate with more objects

## Scene (3D)

- Export PNG max size
- Fix parallel translation and rotation
- Figure out why `H`/`J`/`K`/`L` don't work the same for rotations
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
- Give uniform instead of hardcoding
- Fix on transparent objects (create a separate pass?)

## Flat Scene (2D)

- Vector export (SVG?)
- Show size of the crossovers

## GUI

- Better looking tabs (and show active)

### Organizer Tree

- Fix focus
- Click (fix selection) (and inversely) (I don't remember what I meant but it seems important)
- Double click on strand should teleport in 2d and 3d scenes
- Merge `ensnano_organizer` and `ensnano_gui`

### Help

- Separate menus (scene/flatscene/general)
- Toggleable with `?` icon (at least on scene and flatscene)

## User experience

- App icon
- Finish using `confy` parameters to load saved preferences (`AppStateParameters`)
- FPS counter

### Command-line arguments (`clap`)

- Filename
- Maximized/fullscreen/normal on startup
- GPU power preference (`wgpu::PowerPreference::HighPerformance`)

## Refactor

- Rename structs with the same name in different crates
- Remove as many `pub use` as possible
- Remove as many `use *` as possible
- Remove as many `#[allow(...)]` as possible
- Remove copyright from every file? `LICENSE` at the root should be enough
- Use `mod.rs` everywhere instead of `module.rs` and `<module>/`
- kebab-case -> snake_case for the crate directories
- Fix all typos using `cspell`
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
  - `OverlayType`
  - `AppOperation`
  - `RotationWidgetOrientation`
- Replace `ensnano_iced/fonts/material_icons` by SVG icons from `icondata` lib
- `cargo clippy --workspace --all-targets --all-features`
- `build.rs` for shaders instead of manual compilation
