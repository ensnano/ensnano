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

mod controller;
mod dialog;
mod multiplexer;
mod requests;
mod scheduler;
mod state;

use crate::{
    controller::{
        Controller,
        set_scaffold_sequence::{
            SetScaffoldSequenceError, SetScaffoldSequenceOk, TargetScaffoldLength,
        },
    },
    multiplexer::Multiplexer,
    scheduler::Scheduler,
    state::MainState,
};
use ensnano_design::{CameraId, grid::GridId, group_attributes::GroupPivot};
use ensnano_exports::{ExportResult, ExportType};
use ensnano_flatscene::FlatScene;
use ensnano_gui::{
    GuiManager,
    fonts::{INTER_REGULAR_FONT, load_fonts},
    left_panel::ColorOverlay,
    theme,
};
use ensnano_scene::{Scene, SceneKind};
use ensnano_state::{
    app_state::{
        AppState, LoadDesignError, SaveDesignError,
        action::Action,
        channel_reader::ChannelReaderUpdate,
        design_interactor::{
            DesignInteractor,
            controller::{
                InteractorNotification,
                clipboard::{CopyOperation, PastePosition},
                simulations::SimulationOperation,
            },
        },
        transitions::OkOperation,
    },
    design::{
        operation::{DesignOperation, DesignRotation, DesignTranslation, IsometryTarget},
        selection::{
            Selection, extract_nucls_from_selection, list_of_bezier_vertices, list_of_free_grids,
            list_of_helices, list_of_strands, list_of_xover_as_nucl_pairs,
        },
    },
    gui::{messages::GuiMessages, state::GuiAppState as _},
    requests::Requests,
    utils::application::{Camera3D, Notification},
};
use ensnano_utils::{
    RigidBodyConstants, TEXTURE_FORMAT,
    app_state_parameters::AppStateParameters,
    consts::{APP_NAME, NO_DESIGN_TITLE, SEC_BETWEEN_BACKUPS, WELCOME_MSG},
    convert_size_f32, convert_size_u32,
    graphics::{GuiComponentType, PhySize, SplitMode},
    overlay::OverlayType,
    surfaces::RevolutionSurfaceSystemDescriptor,
    ui_size::UiSize,
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
use ultraviolet::{Rotor3, Vec3};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
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
/// A temporary view of the main state and the control flow.
pub(crate) struct MainStateView<'a> {
    pub(crate) main_state: &'a mut MainState,
    pub(crate) window_target: &'a EventLoopWindowTarget<()>,
    pub(crate) multiplexer: &'a mut Multiplexer,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) gui: &'a mut GuiManager<Requests, AppState>,
    pub(crate) window: &'a Window,
    pub(crate) resized: bool,
}

impl MainStateView<'_> {
    pub(crate) fn pop_action(&mut self) -> Option<Action> {
        if !self.main_state.pending_actions.is_empty() {
            log::debug!("pending actions {:?}", self.main_state.pending_actions);
        }
        self.main_state.pending_actions.pop_front()
    }

    pub(crate) fn check_backup(&mut self) {
        if !self
            .main_state
            .last_backed_up_state
            .design_was_modified(&self.main_state.app_state)
            || !self
                .main_state
                .last_saved_state
                .design_was_modified(&self.main_state.app_state)
        {
            self.main_state.last_backup_date = Instant::now();
        }
    }

    pub(crate) fn main_state(&mut self) -> &mut MainState {
        self.main_state
    }

    pub(crate) fn need_backup(&self) -> bool {
        self.main_state.last_backup_date.elapsed() > Duration::from_secs(SEC_BETWEEN_BACKUPS)
    }

    pub(crate) fn exit_control_flow(&self) {
        self.window_target.exit();
    }

    pub(crate) fn new_design(&mut self) {
        self.notify_apps(Notification::ClearDesigns);
        self.main_state.new_design();
    }

    pub(crate) fn export(&mut self, path: &PathBuf, export_type: ExportType) -> ExportResult {
        let ret = self.main_state.app_state.export(path, export_type);
        self.set_exporting(false);
        ret
    }

    pub(crate) fn load_design(&mut self, path: PathBuf) -> Result<(), LoadDesignError> {
        let state = AppState::import_design(path)?;
        self.notify_apps(Notification::ClearDesigns);
        self.main_state.clear_app_state(state);
        if let Some((position, orientation)) = self
            .main_state
            .app_state
            .get_design_interactor()
            .get_favorite_camera()
        {
            self.notify_apps(Notification::TeleportCamera(Camera3D {
                position,
                orientation,
                pivot_position: None,
            }));
        } else {
            self.main_state.wants_fit = true;
        }
        self.main_state.update_current_file_name();
        Ok(())
    }

    pub(crate) fn apply_operation(&mut self, operation: DesignOperation) {
        self.main_state.apply_operation(operation);
    }

    pub(crate) fn apply_silent_operation(&mut self, operation: DesignOperation) {
        self.main_state.apply_silent_operation(operation);
    }

    pub(crate) fn undo(&mut self) {
        self.main_state.undo();
    }

    pub(crate) fn redo(&mut self) {
        self.main_state.redo();
    }

    pub(crate) fn get_design_interactor(&self) -> DesignInteractor {
        self.main_state.app_state.get_design_interactor()
    }

    pub(crate) fn save_design(&mut self, path: &PathBuf) -> Result<(), SaveDesignError> {
        self.main_state.save_design(path)?;
        self.main_state.last_backup_date = Instant::now();
        Ok(())
    }

    pub(crate) fn save_backup(&mut self) -> Result<(), SaveDesignError> {
        self.main_state.save_backup()?;
        self.main_state.last_backup_date = Instant::now();
        Ok(())
    }

    pub(crate) fn toggle_split_mode(&mut self, mode: SplitMode) {
        self.multiplexer.change_split(mode);
        self.scheduler
            .forward_new_size(self.window.inner_size(), self.multiplexer);
        self.gui.resize(self.multiplexer, self.window);
    }

    pub(crate) fn change_ui_size(&mut self, ui_size: UiSize) {
        self.gui.new_ui_size(
            ui_size,
            self.window,
            self.multiplexer,
            &self.main_state.app_state,
            self.main_state.gui_state(self.multiplexer),
        );
        self.multiplexer.change_ui_size(ui_size, self.window);
        self.main_state
            .messages
            .lock()
            .unwrap()
            .new_ui_size(ui_size);
        self.main_state
            .modify_state(|s| s.with_ui_size(ui_size), None);
        self.resized = true;
    }

    pub(crate) fn notify_apps(&mut self, notification: Notification) {
        log::info!("Notify apps {notification:?}");
        for app in self.main_state.applications.values_mut() {
            app.lock().unwrap().on_notify(notification.clone());
        }
    }

    pub(crate) fn get_selection(&self) -> &[Selection] {
        self.main_state.app_state.get_selection()
    }

    pub(crate) fn get_design_reader(&self) -> DesignInteractor {
        self.main_state.app_state.get_design_interactor()
    }

    pub(crate) fn get_grid_creation_position(&self) -> Option<(Vec3, Rotor3)> {
        self.main_state.get_grid_creation_position()
    }

    pub(crate) fn get_bezier_sheet_creation_position(&self) -> Option<(Vec3, Rotor3)> {
        self.main_state.get_bezier_sheet_creation_position()
    }

    pub(crate) fn finish_operation(&mut self) {
        self.main_state.modify_state(
            |s| s.notified(InteractorNotification::FinishOperation),
            None,
        );
        self.main_state.app_state.finish_operation();
    }

    pub(crate) fn request_copy(&mut self) {
        self.main_state.request_copy();
    }

    pub(crate) fn init_paste(&mut self) {
        self.main_state
            .apply_copy_operation(CopyOperation::PositionPastingPoint(None));
    }

    pub(crate) fn apply_paste(&mut self) {
        self.main_state.apply_paste();
    }

    pub(crate) fn duplicate(&mut self) {
        self.main_state.request_duplication();
    }

    pub(crate) fn request_pasting_candidate(&mut self, candidate: Option<PastePosition>) {
        self.main_state
            .apply_copy_operation(CopyOperation::PositionPastingPoint(candidate));
    }

    pub(crate) fn delete_selection(&mut self) {
        let selection = self.get_selection();
        if let Some((_, nucl_pairs)) =
            list_of_xover_as_nucl_pairs(selection, &self.get_design_reader())
        {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmXovers { xovers: nucl_pairs });
        } else if let Some((_, strand_ids)) = list_of_strands(selection) {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmStrands { strand_ids });
        } else if let Some((_, h_ids)) = list_of_helices(selection) {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmHelices { h_ids });
        } else if let Some(grid_ids) = list_of_free_grids(selection) {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmFreeGrids { grid_ids });
        } else if let Some(vertices) = list_of_bezier_vertices(selection) {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmBezierVertices { vertices });
        }
    }

    pub(crate) fn scaffold_to_selection(&mut self) {
        let scaffold_id = self
            .main_state
            .get_app_state()
            .get_design_interactor()
            .get_scaffold_info()
            .map(|info| info.id);
        if let Some(s_id) = scaffold_id {
            self.main_state
                .update_selection(vec![Selection::Strand(0, s_id as u32)], None);
        }
    }

    pub(crate) fn start_helix_simulation(&mut self, parameters: RigidBodyConstants) {
        self.main_state.start_helix_simulation(parameters);
    }

    pub(crate) fn start_grid_simulation(&mut self, parameters: RigidBodyConstants) {
        self.main_state.start_grid_simulation(parameters);
    }

    pub(crate) fn start_revolution_simulation(&mut self, desc: RevolutionSurfaceSystemDescriptor) {
        self.main_state.start_revolution_simulation(desc);
    }

    pub(crate) fn start_roll_simulation(&mut self, target_helices: Option<Vec<usize>>) {
        self.main_state.start_roll_simulation(target_helices);
    }

    pub(crate) fn update_simulation(&mut self, request: SimulationOperation) {
        self.main_state.update_simulation(request);
    }

    pub(crate) fn set_roll_of_selected_helices(&mut self, roll: f32) {
        self.main_state.set_roll_of_selected_helices(roll);
    }

    pub(crate) fn turn_selection_into_anchor(&mut self) {
        let selection = self.get_selection();
        let nucls = extract_nucls_from_selection(selection);
        self.main_state
            .apply_operation(DesignOperation::FlipAnchors { nucls });
    }

    pub(crate) fn set_visibility_sieve(&mut self, compl: bool) {
        let selection = self.get_selection().to_vec();
        self.main_state.set_visibility_sieve(selection, compl);
    }

    pub(crate) fn clear_visibility_sieve(&mut self) {
        self.main_state.set_visibility_sieve(vec![], true);
    }

    pub(crate) fn need_save(&self) -> Option<Option<PathBuf>> {
        self.main_state
            .need_save()
            .then(|| self.get_current_file_name().map(Path::to_path_buf))
    }

    pub(crate) fn get_current_design_directory(&self) -> Option<&Path> {
        let mut ancestors = self
            .main_state
            .app_state
            .path_to_current_design()
            .as_ref()
            .map(|p| p.ancestors())?;
        let first_ancestor = ancestors.next()?;
        if first_ancestor.is_dir() {
            Some(first_ancestor)
        } else {
            let second_ancestor = ancestors.next()?;
            second_ancestor.is_dir().then_some(second_ancestor)
        }
    }

    pub(crate) fn get_current_file_name(&self) -> Option<&Path> {
        self.main_state.get_current_file_name()
    }

    pub(crate) fn get_design_path_and_notify(
        &mut self,
        notificator: fn(Option<Arc<Path>>) -> Notification,
    ) {
        if let Some(filename) = self.get_current_file_name() {
            self.main_state
                .push_action(Action::NotifyApps(notificator(Some(Arc::from(filename)))));
        } else {
            println!("Design has not been saved yet");
            self.main_state
                .push_action(Action::NotifyApps(notificator(None)));
        }
    }

    pub(crate) fn set_current_group_pivot(&mut self, pivot: GroupPivot) {
        if let Some(group_id) = self.main_state.app_state.get_current_group_id() {
            self.apply_operation(DesignOperation::SetGroupPivot { group_id, pivot });
        } else {
            self.main_state.app_state.set_current_group_pivot(pivot);
        }
    }

    pub(crate) fn translate_group_pivot(&mut self, translation: Vec3) {
        if let Some(group_id) = self.main_state.app_state.get_current_group_id() {
            self.apply_operation(DesignOperation::Translation(DesignTranslation {
                target: IsometryTarget::GroupPivot(group_id),
                translation,
                group_id: None,
            }));
        } else {
            self.main_state.app_state.translate_group_pivot(translation);
        }
    }

    pub(crate) fn rotate_group_pivot(&mut self, rotation: Rotor3) {
        if let Some(group_id) = self.main_state.app_state.get_current_group_id() {
            self.apply_operation(DesignOperation::Rotation(DesignRotation {
                target: IsometryTarget::GroupPivot(group_id),
                rotation,
                origin: Vec3::zero(),
                group_id: None,
            }));
        } else {
            self.main_state.app_state.rotate_group_pivot(rotation);
        }
    }

    pub(crate) fn create_new_camera(&mut self) {
        if let Some(camera) = self
            .main_state
            .applications
            .get(&GuiComponentType::Scene)
            .and_then(|s| s.lock().unwrap().get_camera())
        {
            self.main_state
                .apply_operation(DesignOperation::CreateNewCamera {
                    position: camera.0.position,
                    orientation: camera.0.orientation,
                    pivot_position: camera.0.pivot_position,
                });
        } else {
            log::error!("Could not get current camera position");
        }
    }

    pub(crate) fn select_camera(&mut self, camera_id: CameraId) {
        let reader = self.main_state.app_state.get_design_interactor();
        if let Some(camera) = reader.get_camera_with_id(camera_id) {
            self.notify_apps(Notification::TeleportCamera(camera));
        } else {
            log::error!("Could not get camera {camera_id:?}");
        }
    }

    pub(crate) fn select_favorite_camera(&mut self, n_camera: u32) {
        let reader = self.main_state.app_state.get_design_interactor();
        if let Some(camera) = reader.get_nth_camera(n_camera) {
            self.notify_apps(Notification::TeleportCamera(camera));
        } else {
            log::error!("Design has less than {} cameras", n_camera + 1);
        }
    }

    pub(crate) fn toggle_2d(&mut self) {
        self.multiplexer.toggle_2d();
        self.scheduler
            .forward_new_size(self.window.inner_size(), self.multiplexer);
    }

    pub(crate) fn make_all_suggested_xover(&mut self, doubled: bool) {
        let reader = self.main_state.app_state.get_design_interactor();
        let xovers = reader.get_suggestions();
        self.apply_operation(DesignOperation::MakeSeveralXovers { xovers, doubled });
    }

    pub(crate) fn flip_split_views(&mut self) {
        self.notify_apps(Notification::FlipSplitViews);
    }

    pub(crate) fn start_twist(&mut self, g_id: GridId) {
        self.main_state.start_twist(g_id);
    }

    pub(crate) fn set_expand_insertions(&mut self, expand: bool) {
        self.main_state
            .modify_state(|app| app.with_expand_insertion_set(expand), None);
    }

    pub(crate) fn set_exporting(&mut self, exporting: bool) {
        self.main_state
            .modify_state(|app| app.exporting(exporting), None);
    }

    pub(crate) fn load_3d_object(&mut self, path: PathBuf) {
        let design_path = self
            .get_current_design_directory()
            .map(Path::to_path_buf)
            .or_else(dirs::home_dir)
            .unwrap();
        self.apply_operation(DesignOperation::Add3DObject {
            file_path: path,
            design_path,
        });
    }

    pub(crate) fn load_svg(&mut self, path: PathBuf) {
        self.apply_operation(DesignOperation::ImportSvgPath { path });
    }

    pub(crate) fn set_scaffold_sequence(
        &mut self,
        sequence: String,
        shift: usize,
    ) -> Result<SetScaffoldSequenceOk, SetScaffoldSequenceError> {
        let len = sequence.chars().filter(|c| c.is_alphabetic()).count();
        match self
            .main_state
            .app_state
            .apply_design_op(DesignOperation::SetScaffoldSequence { sequence, shift })
        {
            Ok(OkOperation::Undoable { state, label }) => {
                self.main_state.save_old_state(state, label);
            }
            Ok(OkOperation::NotUndoable) => (),
            Err(e) => return Err(SetScaffoldSequenceError(format!("{e:?}"))),
        }
        let default_shift = self.get_design_interactor().default_shift();
        let scaffold_length = self.get_scaffold_length().unwrap_or(0);
        let target_scaffold_length = if len == scaffold_length {
            TargetScaffoldLength::Ok
        } else {
            TargetScaffoldLength::NotOk {
                design_length: scaffold_length,
                input_scaffold_length: len,
            }
        };
        Ok(SetScaffoldSequenceOk {
            default_shift,
            target_scaffold_length,
        })
    }

    pub(crate) fn optimize_shift(&mut self) {
        self.main_state.optimize_shift();
    }

    pub(crate) fn get_scaffold_length(&self) -> Option<usize> {
        self.main_state
            .app_state
            .get_scaffold_info()
            .map(|info| info.length)
    }
}
