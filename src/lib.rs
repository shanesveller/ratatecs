#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]

use std::{io::Stdout, time::Duration};

use bevy_app::{
    App, AppExit, Last, Plugin, PluginGroup, PluginGroupBuilder, PostUpdate, ScheduleRunnerPlugin,
};
use bevy_ecs::{
    event::EventReader,
    system::{NonSend, NonSendMut, ResMut, Resource, SystemParam},
};
use bevy_state::app::StatesPlugin;
use ratatui::{prelude::CrosstermBackend, widgets::WidgetRef, Frame, Terminal};

pub mod prelude {
    pub use crate::{
        BackendEvent, RatatEcsPlugins, ScopedWidget, TerminalWrapper, TuiPlugin, WidgetDrawer,
        WidgetsToDraw,
    };
    pub use bevy_app::prelude::*;
    pub use bevy_app::AppExit;
    pub use bevy_ecs::prelude::*;
    pub use bevy_state::prelude::*;
    pub use crossterm::event;
    pub use ratatui::prelude::*;
}

pub struct TerminalWrapper {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

pub struct TuiPlugin;

impl Plugin for TuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BackendEvent(None));

        app.add_systems(Last, (get_backend_events, cleanup_on_exit));
        app.add_systems(PostUpdate, render);

        let terminal = ratatui::init();
        app.insert_non_send_resource(TerminalWrapper { terminal });
        app.insert_non_send_resource(WidgetsToDraw { widgets: vec![] });
    }
}

pub struct RatatEcsPlugins;

impl PluginGroup for RatatEcsPlugins {
    fn build(self) -> bevy_app::PluginGroupBuilder {
        let mut builder = PluginGroupBuilder::start::<Self>();
        builder = builder.add(TuiPlugin);
        builder = builder.add(StatesPlugin);
        builder = builder.add(ScheduleRunnerPlugin {
            run_mode: bevy_app::RunMode::Loop {
                wait: Some(Duration::from_millis(0)),
            },
        });

        builder
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

fn render(mut widget_drawer: WidgetDrawer) {
    let _ = widget_drawer.terminal.terminal.draw(|frame| {
        let buf = frame.buffer_mut();
        widget_drawer.widgets.widgets.sort_by_key(|sw| sw.z_order);
        for ScopedWidget { widget, area, .. } in widget_drawer.widgets.widgets.drain(..) {
            widget.render_ref(area, buf);
        }
    });
}

#[derive(SystemParam)]
pub struct WidgetDrawer<'w> {
    widgets: NonSendMut<'w, WidgetsToDraw>,
    terminal: NonSendMut<'w, TerminalWrapper>,
}

impl WidgetDrawer<'_> {
    pub fn push_widget(
        &mut self,
        widget: Box<dyn WidgetRef>,
        area: ratatui::prelude::Rect,
        z_order: u32,
    ) {
        self.widgets.widgets.push(ScopedWidget {
            widget,
            area,
            z_order,
        });
    }

    pub fn get_frame(&mut self) -> Frame {
        self.terminal.terminal.get_frame()
    }
}
