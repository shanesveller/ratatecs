use ratatecs::prelude::*;

fn main() {
    let mut app = App::new_tui();

    app.add_plugins((
        app::component,
        counter::component,
        progress::component,
        popup::component,
    ));

    app.run();
}

mod app {

    use ratatecs::prelude::*;
    use ratatui::widgets::Block;
    use symbols::border;

    pub fn component(app: &mut App) {
        app.add_systems(Update, exit_on_esc);
        app.add_systems(PostUpdate, render);
    }

    fn exit_on_esc(event: Res<BackendEvent>, mut exit: EventWriter<AppExit>) {
        if let Some(event) = &event.0 {
            if let event::Event::Key(key_event) = event {
                if key_event.code == event::KeyCode::Esc {
                    exit.send(AppExit::Success);
                }
            }
        }
    }

    fn render(mut terminal: NonSendMut<TerminalWrapper>, mut widgets: NonSendMut<WidgetsToDraw>) {
        let frame = terminal.terminal.get_frame();
        let area = frame.area();

        let title = Line::from(" My Great TUI ".bold());
        let instructions = Line::from(vec![
            " Quit ".into(),
            "<Esc> ".blue().bold(),
            " Toggle Popup ".into(),
            "<Space> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        widgets.widgets.push(ScopedWidget {
            widget: Box::new(block),
            area,
            z_order: 0,
        });
    }
}

mod counter {
    use ratatecs::prelude::*;
    use ratatui::widgets::{Block, Paragraph};
    use symbols::border;

    #[derive(Resource)]
    struct Counter(u32);

    pub fn component(app: &mut App) {
        app.insert_resource(Counter(0));

        app.add_systems(Update, change_counter);
        app.add_systems(PostUpdate, render);
    }

    fn change_counter(mut counter: ResMut<Counter>, event: Res<BackendEvent>) {
        if let Some(event) = &event.0 {
            if let event::Event::Key(key_event) = event {
                match key_event.code {
                    event::KeyCode::Left => counter.0 = counter.0.saturating_sub(1),
                    event::KeyCode::Right => counter.0 += 1,
                    _ => (),
                }
            }
        }
    }

    fn render(
        counter: Res<Counter>,
        mut terminal: NonSendMut<TerminalWrapper>,
        mut widgets: NonSendMut<WidgetsToDraw>,
    ) {
        let frame = terminal.terminal.get_frame();
        let area = frame.area();
        let area = Rect {
            x: area.x + 10,
            y: area.y + 10,
            width: area.width / 2 - 20,
            height: area.height - 20,
        };

        let title = Line::from(" Counter ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            counter.0.to_string().yellow(),
        ])]);

        widgets.widgets.push(ScopedWidget {
            widget: Box::new(Paragraph::new(counter_text).centered().block(block)),
            area,
            z_order: 1,
        });
    }
}

mod progress {
    use ratatecs::prelude::*;
    use ratatui::widgets::{Block, Gauge};
    use symbols::border;

    #[derive(Resource)]
    struct Progress(u16);

    pub fn component(app: &mut App) {
        app.insert_resource(Progress(0));

        app.add_systems(Update, change_progress);
        app.add_systems(PostUpdate, render);
    }

    fn change_progress(mut progress: ResMut<Progress>, event: Res<BackendEvent>) {
        if let Some(event) = &event.0 {
            if let event::Event::Key(key_event) = event {
                match key_event.code {
                    event::KeyCode::Down => progress.0 = progress.0.saturating_sub(1),
                    event::KeyCode::Up => progress.0 = (progress.0 + 1) % 101,
                    _ => (),
                }
            }
        }
    }

    fn render(
        progress: Res<Progress>,
        mut terminal: NonSendMut<TerminalWrapper>,
        mut widgets: NonSendMut<WidgetsToDraw>,
    ) {
        let frame = terminal.terminal.get_frame();
        let area = frame.area();
        let area = Rect {
            x: area.width / 2 + area.x + 10,
            y: area.y + 10,
            width: area.width / 2 - 20,
            height: area.height - 20,
        };

        let title = Line::from(" Progress ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Down>".blue().bold(),
            " Increment ".into(),
            "<Up>".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        widgets.widgets.push(ScopedWidget {
            widget: Box::new(
                Gauge::default()
                    .block(block)
                    .gauge_style(Style::new().white().on_black().italic())
                    .percent(progress.0),
            ),
            area,
            z_order: 1,
        });
    }
}

mod popup {
    use ratatecs::prelude::*;
    use ratatui::widgets::{Block, Clear, Paragraph};
    use symbols::border;

    #[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
    enum PopupState {
        Open,
        #[default]
        Closed,
    }

    pub fn component(app: &mut App) {
        app.init_state::<PopupState>();
        app.add_systems(Update, toggle_popup);
        app.add_systems(PostUpdate, render.run_if(in_state(PopupState::Open)));
    }

    fn toggle_popup(
        current_state: Res<State<PopupState>>,
        mut next_state: ResMut<NextState<PopupState>>,
        event: Res<BackendEvent>,
    ) {
        if let Some(event) = &event.0 {
            if let event::Event::Key(key_event) = event {
                if key_event.code == event::KeyCode::Char(' ') {
                    match current_state.get() {
                        PopupState::Open => next_state.set(PopupState::Closed),
                        PopupState::Closed => next_state.set(PopupState::Open),
                    }
                }
            }
        }
    }

    fn render(mut terminal: NonSendMut<TerminalWrapper>, mut widgets: NonSendMut<WidgetsToDraw>) {
        let frame = terminal.terminal.get_frame();
        let area = frame.area();
        let area = Rect {
            x: area.width / 2 - 50,
            y: area.height / 2 - 2,
            width: 100,
            height: 4,
        };

        let title = Line::from(" About ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        widgets.widgets.push(ScopedWidget {
            widget: Box::new(Clear),
            area,
            z_order: 2,
        });
        widgets.widgets.push(ScopedWidget {
            widget: Box::new(
                Paragraph::new(Text::from(vec![Line::from(vec![
                    "Hello from ".into(),
                    "ratatecs".red().bold(),
                    ", an experiment in building a TUI with ".into(),
                    "Ratatui".red().bold(),
                    " and ".into(),
                    "Bevy".red().bold(),
                    "!".into(),
                ])]))
                .centered()
                .block(block),
            ),
            area,
            z_order: 2,
        });
    }
}
