//! `ensnano` is a software for designing 3D DNA nanostructures.
//!
//! # Organization of the software
//!
//!
//! The [main] function owns the event loop and the framebuffer. It receives window events
//! and handles the framebuffer.
//!
//! ## Drawing process
//!
//! On each redraw request, the [main] function generates a new frame, and asks the
//! [Multiplexer](multiplexer) to draw on a view of that texture.
//!
//! The [Multiplexer](multiplexer) knows how the window is divided into several regions. For each
//! of these region it knows what application or gui component should draw on it. For each region
//! the [Multiplexer](multiplexer) holds a texture, and at each draw request, it will request the
//! corresponding app or gui element to possibly update the texture.
//!
//!
//! ## Handling of events
//!
//! The Global state of the program is encoded in an automaton defined in the
//! [controller] module. This global state determines whether inputs are handled
//! normally or if the program should wait for the user to interact with dialog windows.
//!
//! When the Global automaton is in NormalState, events are forwarded to the
//! [Multiplexer](multiplexer) which decides what application should handle the event. This is
//! usually the application displayed in the active region (the region under the cursor). Special
//! events like resizing of the window are handled by the multiplexer.
//!
//! When GUIs handle an event. They receive a reference to the state of the main program. This
//! state is encoded in the [AppState] data structure. Each GUI component
//! needs to be able to receive some specific information about the state of the program to handle
//! events and to draw their views. These needs are encoded in traits. GUI component typically
//! defines their own `AppState` trait that must be implemented by the concrete `AppState` type.
//!
//! GUI components may interpret event as a request from the user to modify the design or the state
//! of the main application (for example by changing the selection). These requests are stored in
//! the [Requests] data structure. Each application defines a `Requests` trait
//! that must be implemented by the concrete `Requests` type.
//!
//! On each iteration of the main event loop, if the Global controller is in Normal State,
//! requests are polled and transmitted to the main `AppState` by the main controller. The
//! processing of these requests may have three different kind of consequences:
//!
//!  * An undoable action is performed on the main `AppState`, modifying it. In that case the
//!    current `AppState` is copied on the undo stack and the replaced by the modified one.
//!
//!  * A non-undoable action is performed on the main `AppState`, modifying it. In that case, the
//!    current `AppState` is replaced by the modified one, but not stored on the undo stack.
//!    This typically happens when the `AppState` is in a transient state for example while the user
//!    is performing a drag and drop action. Transient states are not stored on the undo stack
//!    because they are not meant to be restored by undos.
//!   
//!  * An error is returned. In the case the `AppState` is not modified and the user is notified of
//!    the error. Error typically occur when user attempt to make actions on the design that are not
//!    permitted by the current state of the program. For example an error is returned if the user
//!    try to modify the design during a simulation.
//!
//!  # Development detail
//!
//!  ## [wgpu::PresentMode] compatibility experience
//!
//!  The choice of this parameter makes the application to crash at startup, depending on the
//!  environment. Here is a small return of experience.
//!
//!      | PresentMode | Linux (x86) | MacOs (x86) |
//!      |-------------|-------------|-------------|
//!      | AutoVsync   | Yes         | Yes         |
//!      | Immediate   | No          | Yes         |
//!      | Mailbox     | Yes         | No          |

#[cfg(test)]
mod main_tests;

mod app_state;
mod controller;
mod dialog;
mod multiplexer;
mod requests;
mod scheduler;
mod state;

use crate::{
    app_state::{AppState, design_interactor::controller::simulations::SimulationOperation},
    controller::{Controller, channel_reader::ChannelReaderUpdate, normal_state::Action},
    multiplexer::Multiplexer,
    requests::Requests,
    scheduler::Scheduler,
    state::{MainState, MainStateView},
};
use ensnano_flatscene::FlatScene;
use ensnano_gui::{
    GuiManager,
    fonts::{INTER_REGULAR_FONT, load_fonts},
    left_panel::ColorOverlay,
    theme,
};
use ensnano_scene::{Scene, SceneKind};
use ensnano_state::{
    design::operation::DesignOperation, gui::messages::GuiMessages,
    utils::application::Notification,
};
use ensnano_utils::{
    TEXTURE_FORMAT,
    app_state_parameters::AppStateParameters,
    consts::{APP_NAME, NO_DESIGN_TITLE, WELCOME_MSG},
    convert_size_f32, convert_size_u32,
    graphics::{GuiComponentType, PhySize, SplitMode},
    overlay::OverlayType,
};
use iced::{
    advanced::{clipboard, renderer},
    mouse::Cursor,
};
use iced_graphics::{Antialiasing, Viewport};
use iced_runtime::{Debug, program};
use iced_wgpu::Settings;
use std::{
    path::{Component, Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::Window,
};

const PROGRAM_NAME: &str = "ENSnano";

/// Determine if log messages can be printed before the renderer setup.
///
/// Setting it to true will print information in the terminal that are not useful for regular use.
/// By default the value is `false`. It can be set to `true` by enabling the
/// `log_after_renderer_setup` feature.
#[cfg(not(feature = "log_after_renderer_setup"))]
const EARLY_LOG: bool = true;
#[cfg(feature = "log_after_renderer_setup")]
const EARLY_LOG: bool = false;

/// Determine wgpu backends.
///
/// On some windows machine, only the DX12 backends will work. So the `dx12_only` feature forces
/// its use.
#[cfg(not(feature = "dx12_only"))]
const DEFAULT_BACKEND: wgpu::Backends = wgpu::Backends::PRIMARY;
#[cfg(feature = "dx12_only")]
const DEFAULT_BACKEND: wgpu::Backends = wgpu::Backends::DX12;

/// Determine if wgpu errors should panic.
///
/// Set to true because there should not be any "false-positive" in wgpu errors.
///
/// TODO: Make a feature that would set this constant to `false`.
const PANIC_ON_WGPU_ERRORS: bool = true;

/// Main function. Runs the event loop and holds the framebuffer.
///
/// # Initialization
///
/// Before running the event loop, the main function does the following:
///
/// * It requests a connection to the GPU and creates a framebuffer.
/// * It initializes a multiplexer.
/// * It initializes applications and GUI component, and associate regions of the screen to these
///   components
/// * It initializes the [Scheduler] and the [Gui manager](ensnano_gui::Gui)
///
/// # Event loop
///
/// * The event loop waits for an event. If no event is received during 33ms, a new redraw is
///   requested.
/// * When a event is received, it is forwarded to the multiplexer. The Multiplexer may then
///   convert this event into an event for a specific screen region.
/// * When all window events have been handled, the main function reads messages that it received
///   from the [Gui Manager](ensnano_gui::Gui). The consequences of these messages are
///   forwarded to the applications.
/// * The main loops then reads the messages that it received and forwards their consequences to
///   the Gui components.
/// * Finally, a redraw is requested.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if EARLY_LOG {
        pretty_env_logger::init();
    }

    // Parse arguments. If an argument was given it is treated as a file to open.
    let path = std::env::args().nth(1).map(PathBuf::from);

    // Initialize winit. Create an event_loop and a window.
    let event_loop = EventLoop::new()?;
    let window = Arc::new(Window::new(&event_loop)?);
    window.set_title(PROGRAM_NAME);
    window.set_maximized(true);
    window.set_min_inner_size(Some(PhySize::new(500, 500)));

    log::info!("scale factor {}", window.scale_factor());

    // NOTE: Why we don't use window.title() ? Because this method doesn't
    //       work on linux (both X11 and Wayland). See:
    //
    // https://docs.rs/winit/latest/winit/window/struct.Window.html#platform-specific-41
    let mut window_title = String::from(PROGRAM_NAME);

    // Represents the current state of the keyboard modifiers (Shift, Ctrl, etc.)
    let kbd_modifiers = ModifiersState::default();

    // Setup wgpu
    let gpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::util::backend_bits_from_env().unwrap_or(DEFAULT_BACKEND),
        ..Default::default()
    });
    let surface = gpu_instance.create_surface(window.clone())?;
    let (format, device, queue) = futures::executor::block_on(async {
        log::info!(
            "Creating GPU adapter with WGPU_ADAPTER_NAME={:?} and WGPU_POWER_PREF={:?}",
            std::env::var("WGPU_ADAPTER_NAME").ok(),
            std::env::var("WGPU_POWER_PREF").ok(),
        );

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(
            &gpu_instance,
            Some(&surface),
            )
            .await
            .expect("Could not get adapter\n\
                     This might be because gpu drivers are missing.\n\
                     You need Vulkan, Metal (for macOS) or DirectX (for Windows) drivers to run this software");

        let adapter_features = adapter.features();
        let needed_limits = wgpu::Limits::default();
        let capabilities = surface.get_capabilities(&adapter);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: adapter_features & wgpu::Features::default(),
                    required_limits: needed_limits,
                },
                None,
            )
            .await
            .expect("Could not request device nor queue");

        (
            capabilities
                .formats
                .iter()
                .copied()
                .find(wgpu::TextureFormat::is_srgb)
                .or_else(|| capabilities.formats.first().copied())
                .expect("Get preferred format"),
            device,
            queue,
        )
    });

    if !PANIC_ON_WGPU_ERRORS {
        device.on_uncaptured_error(Box::new(|e| log::error!("wgpu error {e:?}")));
    }

    let physical_size = window.inner_size();
    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );

    let parameters: AppStateParameters = confy::load(APP_NAME, APP_NAME).unwrap_or_default();

    let settings = Settings {
        antialiasing: Some(Antialiasing::MSAAx4),
        default_text_size: parameters.ui_size.main_text().into(),
        default_font: INTER_REGULAR_FONT,
        ..Default::default()
    };
    // Initialize the renderer
    let mut overlay_renderer = iced::Renderer::Wgpu(iced_wgpu::Renderer::new(
        iced_wgpu::Backend::new(&device, &queue, settings, format),
        settings.default_font,
        settings.default_text_size,
    ));
    load_fonts(&mut overlay_renderer);
    let device = Rc::new(device);
    let queue = Rc::new(queue);
    let mut resized = false;
    let mut scale_factor_changed = false;

    let gui_theme = theme::gui_theme();

    // Initialize the Scheduler
    let requests = Arc::new(Mutex::new(Requests::default()));
    let messages = Arc::new(Mutex::new(GuiMessages::new()));
    let mut scheduler = Scheduler::new();

    // Initialize the layout
    let mut multiplexer = Multiplexer::new(
        window.inner_size(),
        window.scale_factor(),
        Rc::clone(&device),
        Arc::clone(&requests),
        parameters.ui_size,
    );
    multiplexer.change_split(SplitMode::Both);

    // Initialize the scenes
    //
    // The `encoder` encodes a series of GPU operations.
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let scene_area = multiplexer
        .get_element_area(GuiComponentType::Scene)
        .unwrap();
    let scene = Arc::new(Mutex::new(Scene::new(
        Rc::clone(&device),
        Rc::clone(&queue),
        window.inner_size(),
        scene_area,
        Arc::clone(&requests),
        &mut encoder,
        Default::default(),
        SceneKind::Cartesian,
    )));
    let stereographic_scene = Arc::new(Mutex::new(Scene::new(
        Rc::clone(&device),
        Rc::clone(&queue),
        window.inner_size(),
        scene_area,
        Arc::clone(&requests),
        &mut encoder,
        Default::default(),
        SceneKind::Stereographic,
    )));

    queue.submit(Some(encoder.finish()));
    scheduler.add_application(scene.clone(), GuiComponentType::Scene);
    scheduler.add_application(
        stereographic_scene.clone(),
        GuiComponentType::StereographicScene,
    );

    let flat_scene = Arc::new(Mutex::new(FlatScene::new(
        Rc::clone(&device),
        Rc::clone(&queue),
        window.inner_size(),
        scene_area,
        requests.clone(),
        Default::default(),
    )));
    scheduler.add_application(flat_scene.clone(), GuiComponentType::FlatScene);

    // Initialize the UI
    let mut main_state = MainState::new(messages.clone());

    let mut gui = GuiManager::new(
        Rc::clone(&device),
        Rc::clone(&queue),
        &window,
        &multiplexer,
        Arc::clone(&requests),
        parameters,
        &main_state.app_state,
        Default::default(),
    );

    let mut overlay_manager =
        OverlayManager::new(Arc::clone(&requests), &window, &mut overlay_renderer);

    // Run event loop
    let mut last_render_time = Instant::now();
    let mut mouse_interaction = iced::mouse::Interaction::Pointer;

    main_state
        .applications
        .insert(GuiComponentType::Scene, scene);
    main_state
        .applications
        .insert(GuiComponentType::FlatScene, flat_scene);
    main_state
        .applications
        .insert(GuiComponentType::StereographicScene, stereographic_scene);

    // Add a design to the scene if one was given as a command line argument
    if path.is_some() {
        main_state.push_action(Action::LoadDesign(path));
    }
    main_state.update();
    main_state.last_saved_state = main_state.app_state.clone();

    let mut controller = Controller::new();

    println!("{WELCOME_MSG}");

    if !EARLY_LOG {
        pretty_env_logger::init();
    }

    let mut first_iteration = true;

    let mut last_gui_state = (
        main_state.app_state.clone(),
        main_state.gui_state(&multiplexer),
    );
    messages
        .lock()
        .unwrap()
        .push_application_state(main_state.get_app_state(), last_gui_state.1.clone());

    event_loop.run(move |window_event, window_target| {
        // Wait for event or redraw a frame every 33 ms (30 frame per seconds)
        window_target.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(33),
        ));

        let mut main_state_view = MainStateView {
            main_state: &mut main_state,
            window_target,
            multiplexer: &mut multiplexer,
            gui: &mut gui,
            scheduler: &mut scheduler,
            window: &window,
            resized: false,
        };

        match window_event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => {
                    main_state_view
                        .main_state
                        .pending_actions
                        .push_back(Action::Exit);
                }

                WindowEvent::Focused(false) => {
                    main_state_view.notify_apps(Notification::WindowFocusLost);
                }

                WindowEvent::ModifiersChanged(modifiers) => {
                    main_state_view.multiplexer.update_modifiers(modifiers);
                    messages.lock().unwrap().update_modifiers(modifiers);
                    main_state_view.notify_apps(Notification::ModifiersChanged(modifiers));
                }

                // NOTE: Escape fullscreen mode.
                //
                WindowEvent::KeyboardInput {
                    event: key_event, ..
                } if (key_event.logical_key == Key::Named(NamedKey::Escape)
                    && window.fullscreen().is_some()) =>
                {
                    window.set_fullscreen(None);
                }

                // NOTE: KEYBOARD PRIORITY MODE
                //       Some widgets –such as [text_input]– need to intercept keys that are otherwise used
                //       as shortcuts by the UI. The “keyboard priority” feature has been made for this,
                //       and the interception is made here.
                //
                WindowEvent::KeyboardInput { .. } if main_state.keyboard_priority.is_some() => {
                    if let Some(iced_event) = iced_winit::conversion::window_event(
                        iced::window::Id::MAIN,
                        // NOTE: Used to be window.id(). It seems dirty,
                        //       but the same is done in iced/examples/integration
                        window_event,
                        window.scale_factor(),
                        kbd_modifiers,
                    ) {
                        gui.forward_event_all(iced_event);
                    }
                }

                WindowEvent::RedrawRequested
                    if window.inner_size().width > 0 && window.inner_size().height > 0 =>
                {
                    if resized {
                        multiplexer.generate_textures();
                        scheduler.forward_new_size(window.inner_size(), &multiplexer);
                        let window_size = window.inner_size();

                        surface.configure(
                            &device,
                            &wgpu::SurfaceConfiguration {
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                format: TEXTURE_FORMAT,
                                width: window_size.width,
                                height: window_size.height,
                                present_mode: wgpu::PresentMode::AutoVsync,
                                desired_maximum_frame_latency: 2,
                                alpha_mode: Default::default(),
                                view_formats: Default::default(),
                            },
                        );

                        gui.resize(&multiplexer, &window);
                        log::trace!(
                            "Will draw on texture of size {}x {}",
                            window_size.width,
                            window_size.height
                        );
                    }
                    if scale_factor_changed {
                        multiplexer.generate_textures();
                        gui.notify_scale_factor_change(
                            &window,
                            &multiplexer,
                            &main_state.app_state,
                            main_state.gui_state(&multiplexer),
                        );
                        log::info!("Notified of scale factor change: {}", window.scale_factor());
                        scheduler.forward_new_size(window.inner_size(), &multiplexer);
                        let window_size = window.inner_size();

                        surface.configure(
                            &device,
                            &wgpu::SurfaceConfiguration {
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                format: TEXTURE_FORMAT,
                                width: window_size.width,
                                height: window_size.height,
                                present_mode: wgpu::PresentMode::AutoVsync,
                                desired_maximum_frame_latency: 2,
                                alpha_mode: Default::default(),
                                view_formats: Default::default(),
                            },
                        );

                        gui.resize(&multiplexer, &window);
                    }
                    // Get viewports from the partition

                    // If there are events pending
                    gui.update(
                        &multiplexer,
                        &gui_theme,
                        &theme::gui_style(&gui_theme),
                        &window,
                    );

                    overlay_manager.process_event(
                        &mut overlay_renderer,
                        &gui_theme,
                        &theme::gui_style(&gui_theme),
                        resized,
                        &multiplexer,
                        &window,
                    );

                    resized = false;
                    scale_factor_changed = false;

                    if let Ok(frame) = surface.get_current_texture() {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        // We draw the applications first
                        scheduler.draw_apps(&mut encoder, &multiplexer);

                        gui.render(
                            &mut encoder,
                            None, // TODO: See if another value of clear_color is more appropriate.
                            &window,
                            &multiplexer,
                            &mut mouse_interaction,
                        );

                        if multiplexer.resize(window.inner_size(), window.scale_factor()) {
                            resized = true;
                            window.request_redraw();
                            return;
                        }
                        log::trace!("window size {:?}", window.inner_size());
                        multiplexer.draw(
                            &mut encoder,
                            &frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                            &window,
                        );
                        overlay_manager.render(
                            &device,
                            &queue,
                            &mut encoder,
                            frame.texture.format(),
                            &frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                            &multiplexer,
                            &window,
                            &mut overlay_renderer,
                        );

                        // Then we submit the work
                        queue.submit(Some(encoder.finish()));
                        frame.present();

                        // And update the mouse cursor
                        main_state.gui_cursor =
                            iced_winit::conversion::mouse_interaction(mouse_interaction);
                        main_state.update_cursor(&multiplexer);
                        window.set_cursor_icon(main_state.cursor);
                    } else {
                        log::warn!("Error getting next frame, attempt to recreate swap chain");
                        resized = true;
                    }
                }

                // NOTE: Any other [WindowEvent]
                //
                _ => {
                    //let modifiers = multiplexer.modifiers();

                    // Feed the event to the multiplexer
                    let event =
                        multiplexer.event(window_event, &mut resized, &mut scale_factor_changed);

                    if let Some((event, gui_component_type)) = event {
                        // Update the focused gui component
                        if main_state.focused_component != Some(gui_component_type) {
                            if let Some(app) = main_state
                                .focused_component
                                .as_ref()
                                .and_then(|elt| main_state.applications.get(elt))
                            {
                                app.lock().unwrap().on_notify(Notification::WindowFocusLost);
                            }
                            main_state.focused_component = Some(gui_component_type);
                            main_state.update_candidates(vec![]);
                        }
                        main_state.applications_cursor = None;

                        // Feed the event to the gui component on which it happened
                        match gui_component_type {
                            component if component.is_panel() => {
                                if let Some(e) = iced_winit::conversion::window_event(
                                    iced::window::Id::MAIN,
                                    // NOTE: Used to be window.id(). It seems dirty,
                                    //       but the same is done in iced/examples/integration
                                    event,
                                    window.scale_factor(),
                                    kbd_modifiers,
                                ) {
                                    gui.forward_event(component, e);
                                }
                            }
                            GuiComponentType::Overlay(n) => {
                                if let Some(e) = iced_winit::conversion::window_event(
                                    iced::window::Id::MAIN,
                                    // NOTE: Used to be window.id(). It seems dirty,
                                    //       but the same is done in iced/examples/integration
                                    event,
                                    window.scale_factor(),
                                    kbd_modifiers,
                                ) {
                                    overlay_manager.forward_event(e, n);
                                }
                            }
                            area if area.is_scene() => {
                                let cursor_position = multiplexer.get_cursor_position();
                                let state = main_state.get_app_state();
                                main_state.applications_cursor =
                                    scheduler.forward_event(&event, area, cursor_position, state);
                                if matches!(event, WindowEvent::MouseInput { .. }) {
                                    gui.clear_focus();
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            },
            Event::AboutToWait => {
                scale_factor_changed |= multiplexer.check_scale_factor(&window);
                let mut redraw = resized || scale_factor_changed;
                redraw |= main_state.update_cursor(&multiplexer);
                redraw |= gui.fetch_change(
                    &window,
                    &gui_theme,
                    &theme::gui_style(&gui_theme),
                    &multiplexer,
                );

                // When there is no more event to deal with
                requests::poll::poll_all(requests.lock().unwrap(), &mut main_state);

                let mut main_state_view = MainStateView {
                    main_state: &mut main_state,
                    window_target,
                    multiplexer: &mut multiplexer,
                    gui: &mut gui,
                    scheduler: &mut scheduler,
                    window: &window,
                    resized: false,
                };

                if main_state_view.main_state.wants_fit {
                    main_state_view.notify_apps(Notification::FitRequest);
                    main_state_view.main_state.wants_fit = false;
                }
                controller.make_progress(&mut main_state_view);
                resized |= main_state_view.resized;
                resized |= first_iteration;
                first_iteration = false;

                for update in main_state.channel_reader.get_updates() {
                    match update {
                        ChannelReaderUpdate::ScaffoldShiftOptimizationProgress(x) => {
                            main_state
                                .messages
                                .lock()
                                .unwrap()
                                .push_progress("Optimizing: ".to_owned(), x);
                        }
                        ChannelReaderUpdate::ScaffoldShiftOptimizationResult(result) => {
                            main_state.messages.lock().unwrap().finish_progress();
                            if let Ok(result) = result {
                                main_state.apply_operation(DesignOperation::SetScaffoldShift(
                                    result.position,
                                ));
                                let msg = format!(
                                    "Scaffold position set to {}\n {}",
                                    result.position, result.score
                                );
                                main_state.pending_actions.push_back(Action::ErrorMsg(msg));
                            } else {
                                // unwrap because in this block, result is necessarily an Err
                                log::warn!("{:?}", result.err().unwrap());
                            }
                        }
                        ChannelReaderUpdate::SimulationUpdate(update) => {
                            main_state.app_state.apply_simulation_update(update);
                        }
                        ChannelReaderUpdate::SimulationExpired => {
                            main_state.update_simulation(SimulationOperation::Stop);
                        }
                    }
                }

                log::trace!("call update from main");
                main_state.update();

                let new_title = format!(
                    "{} {}",
                    PROGRAM_NAME,
                    match main_state.get_current_file_name() {
                        Some(path) => formatted_path_end(path),
                        None => NO_DESIGN_TITLE.to_owned(),
                    }
                );
                if window_title != new_title {
                    window.set_title(&new_title);
                    window_title = new_title;
                }

                // Treat eventual event that happened in the gui left panel.
                let _overlay_change = overlay_manager.fetch_change(
                    &multiplexer,
                    &window,
                    &mut overlay_renderer,
                    &gui_theme,
                    &theme::gui_style(&gui_theme),
                );
                {
                    let mut messages = messages.lock().unwrap();
                    gui.forward_messages(&mut messages);
                    overlay_manager.forward_messages(&mut messages);
                }

                let now = Instant::now();
                let dt = now - last_render_time;
                redraw |= scheduler.check_redraw(&multiplexer, dt, main_state.get_app_state());
                let new_gui_state = (
                    main_state.app_state.clone(),
                    main_state.gui_state(&multiplexer),
                );
                if new_gui_state != last_gui_state {
                    last_gui_state = new_gui_state;
                    messages.lock().unwrap().push_application_state(
                        main_state.get_app_state(),
                        last_gui_state.1.clone(),
                    );
                    redraw = true;
                }
                last_render_time = now;

                if redraw {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

struct OverlayManager {
    color_state: program::State<ColorOverlay<Requests>>,
    color_debug: Debug,
    overlay_types: Vec<OverlayType>,
}

impl OverlayManager {
    fn new(requests: Arc<Mutex<Requests>>, window: &Window, renderer: &mut iced::Renderer) -> Self {
        let color = ColorOverlay::new(
            requests,
            PhysicalSize::new(250., 250.).to_logical(window.scale_factor()),
        );
        let mut color_debug = Debug::new();
        let color_state = program::State::new(
            color,
            convert_size_f32(PhysicalSize::new(250, 250)),
            renderer,
            &mut color_debug,
        );

        Self {
            color_state,
            color_debug,
            overlay_types: Vec::new(),
        }
    }

    fn forward_event(&mut self, event: iced::Event, n: usize) {
        match self.overlay_types.get(n) {
            None => {
                log::error!("receive event from non existing overlay");
                unreachable!();
            }
            Some(OverlayType::Color) => self.color_state.queue_event(event),
        }
    }

    fn process_event(
        &mut self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        resized: bool,
        multiplexer: &Multiplexer,
        window: &Window,
    ) {
        for (n, overlay) in self.overlay_types.iter().enumerate() {
            let cursor = if multiplexer.focused_element() == Some(GuiComponentType::Overlay(n)) {
                let point = iced_winit::conversion::cursor_position(
                    multiplexer.get_cursor_position(),
                    window.scale_factor(),
                );
                Cursor::Available(point)
            } else {
                Cursor::Unavailable
            };
            let mut clipboard = clipboard::Null;
            match overlay {
                OverlayType::Color => {
                    if !self.color_state.is_queue_empty() || resized {
                        let _ = self.color_state.update(
                            convert_size_f32(PhysicalSize::new(250, 250)),
                            cursor,
                            renderer,
                            theme,
                            style,
                            &mut clipboard,
                            &mut self.color_debug,
                        );
                    }
                }
            }
        }
    }

    fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        format: wgpu::TextureFormat,
        target: &wgpu::TextureView,
        multiplexer: &Multiplexer,
        window: &Window,
        renderer: &mut iced::Renderer,
    ) {
        for overlay_type in &self.overlay_types {
            match overlay_type {
                OverlayType::Color => {
                    let color_viewport = Viewport::with_physical_size(
                        convert_size_u32(multiplexer.window_size),
                        window.scale_factor(),
                    );
                    match renderer {
                        iced::Renderer::Wgpu(wgpu_renderer) => {
                            wgpu_renderer.with_primitives(|backend, primitives| {
                                backend.present(
                                    device,
                                    queue,
                                    encoder,
                                    None, // TODO: Examine what clear_color is.
                                    format,
                                    target,
                                    primitives,
                                    &color_viewport,
                                    &self.color_debug.overlay(),
                                );
                            });
                        }
                        iced::Renderer::TinySkia(_) => unreachable!(),
                    }
                }
            }
        }
    }

    fn forward_messages(&self, _messages: &mut GuiMessages<AppState>) {}

    fn fetch_change(
        &mut self,
        multiplexer: &Multiplexer,
        window: &Window,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
    ) -> bool {
        let mut ret = false;
        for (n, overlay) in self.overlay_types.iter().enumerate() {
            let cursor = if multiplexer.focused_element() == Some(GuiComponentType::Overlay(n)) {
                let point = iced_winit::conversion::cursor_position(
                    multiplexer.get_cursor_position(),
                    window.scale_factor(),
                );
                Cursor::Available(point)
            } else {
                Cursor::Unavailable
            };
            let mut clipboard = clipboard::Null;
            match overlay {
                OverlayType::Color => {
                    if !self.color_state.is_queue_empty() {
                        ret = true;
                        let _ = self.color_state.update(
                            convert_size_f32(PhysicalSize::new(250, 250)),
                            cursor,
                            renderer,
                            theme,
                            style,
                            &mut clipboard,
                            &mut self.color_debug,
                        );
                    }
                }
            }
        }
        ret
    }
}

fn formatted_path_end<P: AsRef<Path>>(path: P) -> String {
    let components: Vec<_> = path
        .as_ref()
        .components()
        .map(Component::as_os_str)
        .collect();
    let mut ret = if components.len() > 3 {
        vec!["..."]
    } else {
        vec![]
    };
    let mut iter = components.iter().rev().take(3).rev();
    for _ in 0..3 {
        if let Some(comp) = iter.next().and_then(|s| s.to_str()) {
            ret.push(comp);
        }
    }
    ret.join("/")
}
