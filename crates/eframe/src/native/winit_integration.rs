use std::{sync::Arc, time::Instant};

use winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[cfg(feature = "accesskit")]
use egui::accesskit;
use egui::ViewportId;
#[cfg(feature = "accesskit")]
use egui_winit::accesskit_winit;

/// Create an egui context, restoring it from storage if possible.
pub fn create_egui_context(storage: Option<&dyn crate::Storage>) -> egui::Context {
    crate::profile_function!();

    pub const IS_DESKTOP: bool = cfg!(any(
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "windows",
    ));

    let egui_ctx = egui::Context::default();

    egui_ctx.set_embed_viewports(!IS_DESKTOP);

    let memory = crate::native::epi_integration::load_egui_memory(storage).unwrap_or_default();
    egui_ctx.memory_mut(|mem| *mem = memory);

    egui_ctx
}

/// The custom even `eframe` uses with the [`winit`] event loop.
#[derive(Debug)]
pub enum UserEvent {
    /// A repaint is requested.
    RequestRepaint {
        /// What to repaint.
        viewport_id: ViewportId,

        /// When to repaint.
        when: Instant,

        /// What the frame number was when the repaint was _requested_.
        frame_nr: u64,
    },

    /// A request related to [`accesskit`](https://accesskit.dev/).
    #[cfg(feature = "accesskit")]
    AccessKitActionRequest {
        request: accesskit::ActionRequest,
        window_id: WindowId,
    },
}

#[cfg(feature = "accesskit")]
impl From<accesskit_winit::Event> for UserEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        match event.window_event {
            accesskit_winit::WindowEvent::InitialTreeRequested => todo!(),
            accesskit_winit::WindowEvent::AccessibilityDeactivated => todo!(),
            accesskit_winit::WindowEvent::ActionRequested(request) => {
                Self::AccessKitActionRequest {
                    request,
                    window_id: event.window_id,
                }
            }
        }
    }
}

pub trait WinitApp {
    /// The current frame number, as reported by egui.
    fn frame_nr(&self, viewport_id: ViewportId) -> u64;

    fn window(&self, window_id: WindowId) -> Option<Arc<Window>>;

    fn window_id_from_viewport_id(&self, id: ViewportId) -> Option<WindowId>;

    fn save_and_destroy(&mut self);

    fn run_ui_and_paint(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
    ) -> EventResult;

    fn on_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: &winit::event::Event<UserEvent>,
    ) -> crate::Result<EventResult>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventResult {
    Wait,

    /// Causes a synchronous repaint inside the event handler. This should only
    /// be used in special situations if the window must be repainted while
    /// handling a specific event. This occurs on Windows when handling resizes.
    ///
    /// `RepaintNow` creates a new frame synchronously, and should therefore
    /// only be used for extremely urgent repaints.
    RepaintNow(WindowId),

    /// Queues a repaint for once the event loop handles its next redraw. Exists
    /// so that multiple input events can be handled in one frame. Does not
    /// cause any delay like `RepaintNow`.
    RepaintNext(WindowId),

    RepaintAt(WindowId, Instant),

    Exit,
}

pub fn system_theme(window: &Window, options: &crate::NativeOptions) -> Option<crate::Theme> {
    if options.follow_system_theme {
        window
            .theme()
            .map(super::epi_integration::theme_from_winit_theme)
    } else {
        None
    }
}

/// Short and fast description of an event.
/// Useful for logging and profiling.
pub fn short_event_description(event: &winit::event::Event<UserEvent>) -> &'static str {
    match event {
        winit::event::Event::UserEvent(user_event) => match user_event {
            UserEvent::RequestRepaint { .. } => "UserEvent::RequestRepaint",
            #[cfg(feature = "accesskit")]
            UserEvent::AccessKitActionRequest { .. } => "UserEvent::AccessKitActionRequest",
        },
        _ => egui_winit::short_generic_event_description(event),
    }
}
