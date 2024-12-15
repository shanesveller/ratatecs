# Ratatecs

Experiment with [Bevy ECS](https://bevyengine.org) and [Ratatui](https://ratatui.rs) to build TUI with completely isolated panels (group of widgets).

- Panel inner state is stored in the ECS
- Panel selection through State
- Each panel pushes a z-ordered list of widgets that are rendered at the end of the frame

Bevy handles the run loop, storing state in the world, and passing the needed arguments to each function through dependency injection.

> [!NOTE]  
> This is experimental, and the API will evolve as I collect feedback.

## Example Usage

![example with 5 panels](https://raw.githubusercontent.com/vleue/ratatecs/main/ratatecs.gif)

```rust,no_run
use ratatecs::prelude::*;

fn main() {
    App::new().add_plugins((RatatEcsPlugins::default(), app::panel)).run();
}

mod app {
    use std::io::Stdout;
    use ratatecs::prelude::*;
    use ratatui::widgets::{Block, Paragraph};
    use symbols::border;

    #[derive(Resource)]
    struct Counter(u32);

    pub fn panel(app: &mut App) {
        // Store the state of this panel in the world
        app.insert_resource(Counter(0));

        // Systems that update the state or react to user inputs
        app.add_systems(Update, (exit_on_esc, change_counter));

        // System to render thos panel
        app.add_systems(PostUpdate, render::<CrosstermBackend<Stdout>>);
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

    fn render<B: Backend + 'static>(counter: Res<Counter>, mut drawer: WidgetDrawer<B>) {
        let frame = drawer.get_frame();
        let area = frame.area();

        let title = Line::from(" My Great TUI ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Esc> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            counter.0.to_string().yellow(),
        ])]);

        drawer.push_widget(
            Box::new(Paragraph::new(counter_text).centered().block(block)),
            area,
            0,
        );
    }
}
```
