#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]

use std::{io::Stdout, time::Duration};

use bevy_app::{App, AppExit, Last, PostUpdate, ScheduleRunnerPlugin};
use bevy_ecs::{
    event::EventReader,
    system::{NonSend, NonSendMut, ResMut, Resource},
};
use bevy_state::app::StatesPlugin;
use ratatui::{prelude::CrosstermBackend, widgets::WidgetRef, Terminal};

pub mod prelude {
    pub use crate::{BackendEvent, Ratatapp, ScopedWidget, TerminalWrapper, WidgetsToDraw};
    pub use bevy_app::prelude::*;
    pub use bevy_app::AppExit;
    pub use bevy_ecs::prelude::*;
    pub use bevy_state::prelude::*;
    pub use crossterm::event;
    pub use ratatui::prelude::*;
}

pub trait Ratatapp {
    fn new_tui() -> Self;
}

pub struct TerminalWrapper {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ratatapp for bevy_app::App {
    fn new_tui() -> Self {
        let mut app = App::new();

        app.insert_resource(BackendEvent(None));

        app.add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_millis(0)),
            StatesPlugin,
        ));

        app.add_systems(Last, (get_backend_events, cleanup_on_exit));
        app.add_systems(PostUpdate, render);

        let terminal = ratatui::init();
        app.insert_non_send_resource(TerminalWrapper { terminal });
        app.insert_non_send_resource(WidgetsToDraw { widgets: vec![] });

        app
    }
}

#[derive(Resource)]
pub struct BackendEvent(pub Option<crossterm::event::Event>);

fn get_backend_events(mut event: ResMut<BackendEvent>) {
    let new_event = crossterm::event::poll(Duration::from_millis(10))
        .ok()
        .and_then(|has_event| {
            if has_event {
                crossterm::event::read().ok()
            } else {
                None
            }
        });
    match (event.0.is_some(), new_event.is_some()) {
        (_, true) => event.0 = new_event,
        (true, false) => event.0 = None,
        _ => (),
    }
}

fn cleanup_on_exit(_: NonSend<TerminalWrapper>, exits: EventReader<AppExit>) {
    if !exits.is_empty() {
        ratatui::restore();
    }
}

pub struct ScopedWidget {
    pub widget: Box<dyn WidgetRef>,
    pub area: ratatui::prelude::Rect,
    pub z_order: u32,
}

pub struct WidgetsToDraw {
    pub widgets: Vec<ScopedWidget>,
}

fn render(mut terminal: NonSendMut<TerminalWrapper>, mut widgets: NonSendMut<WidgetsToDraw>) {
    let _ = terminal.terminal.draw(|frame| {
        let buf = frame.buffer_mut();
        widgets.widgets.sort_by_key(|sw| sw.z_order);
        for ScopedWidget { widget, area, .. } in widgets.widgets.drain(..) {
            widget.render_ref(area, buf);
        }
    });
}
