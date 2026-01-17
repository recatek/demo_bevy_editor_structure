use bevy::app::PluginsState;
use bevy::prelude::*;

use bevy::window::exit_on_all_closed;
use bevy_winit::accessibility::AccessKitPlugin;
use bevy_winit::cursor;
use bevy_winit::state::WinitAppRunnerState;
use bevy_winit::system::*;
use bevy_winit::{
    DisplayHandleWrapper, EventLoop, EventLoopProxyWrapper, RawWinitWindowEvent, WinitMonitors,
    WinitSettings, WinitUserEvent,
};

pub fn init_winit(app: &mut App, post_update: fn(&mut App) -> ()) {
    let run_on_any_thread = false;
    let mut event_loop_builder = EventLoop::<WinitUserEvent>::with_user_event();

    // linux check is needed because x11 might be enabled on other platforms.
    #[cfg(all(target_os = "linux", feature = "x11"))]
    {
        use winit::platform::x11::EventLoopBuilderExtX11;

        // This allows a Bevy app to be started and ran outside the main thread.
        // A use case for this is to allow external applications to spawn a thread
        // which runs a Bevy app without requiring the Bevy app to need to reside on
        // the main thread, which can be problematic.
        event_loop_builder.with_any_thread(run_on_any_thread);
    }

    // linux check is needed because wayland might be enabled on other platforms.
    #[cfg(all(target_os = "linux", feature = "wayland"))]
    {
        use winit::platform::wayland::EventLoopBuilderExtWayland;
        event_loop_builder.with_any_thread(run_on_any_thread);
    }

    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        event_loop_builder.with_any_thread(run_on_any_thread);
    }

    #[cfg(target_os = "android")]
    {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        let msg = "Bevy must be setup with the #[bevy_main] macro on Android";
        event_loop_builder.with_android_app(bevy_android::ANDROID_APP.get().expect(msg).clone());
    }

    let event_loop = event_loop_builder
        .build()
        .expect("Failed to build event loop");

    app.init_resource::<WinitMonitors>()
        .init_resource::<WinitSettings>()
        .insert_resource(DisplayHandleWrapper(event_loop.owned_display_handle()))
        .insert_resource(EventLoopProxyWrapper(event_loop.create_proxy()))
        .add_message::<RawWinitWindowEvent>()
        .set_runner(move |app| winit_runner(app, event_loop, post_update))
        .add_systems(
            Last,
            (
                // `exit_on_all_closed` only checks if windows exist but doesn't access data,
                // so we don't need to care about its ordering relative to `changed_windows`
                changed_windows.ambiguous_with(exit_on_all_closed),
                changed_cursor_options,
                despawn_windows,
                check_keyboard_focus_lost,
            )
                .chain(),
        );

    app.add_plugins(AccessKitPlugin);
    app.add_plugins(cursor::WinitCursorPlugin);

    app.add_observer(
        |_window: On<Add, Window>, event_loop_proxy: Res<EventLoopProxyWrapper>| -> Result {
            event_loop_proxy.send_event(WinitUserEvent::WindowAdded)?;

            Ok(())
        },
    );
}

pub fn winit_runner(
    mut app: App,
    event_loop: EventLoop<WinitUserEvent>,
    post_update: fn(&mut App) -> (),
) -> AppExit {
    if app.plugins_state() == PluginsState::Ready {
        app.finish();
        app.cleanup();
    }

    let mut runner_state = WinitAppRunnerState::new(app);
    runner_state.post_update = Some(post_update);

    trace!("starting winit event loop");
    // The winit docs mention using `spawn` instead of `run` on Wasm.
    // https://docs.rs/winit/latest/winit/platform/web/trait.EventLoopExtWebSys.html#tymethod.spawn_app
    let mut runner_state = runner_state;
    if let Err(err) = event_loop.run_app(&mut runner_state) {
        error!("winit event loop returned an error: {err}");
    }
    // If everything is working correctly then the event loop only exits after it's sent an exit code.
    runner_state.app_exit.unwrap_or_else(|| {
        error!("Failed to receive an app exit code! This is a bug");
        AppExit::error()
    })
}
