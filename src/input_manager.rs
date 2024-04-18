use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum DevControls {
    TestChord,
    TestSpawnNewWindow
}
impl DevControls {
    pub fn default_input_map() -> InputMap<DevControls> {
        use KeyCode::*;
        InputMap::new([
            (Self::TestChord, UserInput::Chord(
                vec!(InputKind::PhysicalKey(ControlLeft), InputKind::PhysicalKey(ShiftLeft) ))),
            (Self::TestSpawnNewWindow, UserInput::Single(InputKind::PhysicalKey(KeyN)))
        ])
    }
}

// For now, broadcast the dev chord actions as events.
#[derive(Event)]
pub struct TestChordJustPressed(pub Entity);

#[derive(Event)]
pub struct TestChordPressed(pub Entity);

#[derive(Event)]
pub struct TestChordJustReleased(pub Entity);

#[derive(Event)]
pub struct SpawnWindowKeyJustPressed;

// Send window spawn event for testing.
pub fn test_spawn_window (
    dev_controls: Res<ActionState<DevControls>>,
    mut spawn_window_key_just_pressed: EventWriter<SpawnWindowKeyJustPressed>
) {
    if dev_controls.just_pressed(&DevControls::TestSpawnNewWindow) {
        spawn_window_key_just_pressed.send(SpawnWindowKeyJustPressed);
    }
}

// TODO: Find way to gatekeep this with a run condition.
pub fn test_chord_pressed(
    dev_controls: Res<ActionState<DevControls>>,
    window_query: Query<(Entity, &Window)>,
    mut test_chord_just_pressed: EventWriter<TestChordJustPressed>,
    mut test_chord_pressed: EventWriter<TestChordPressed>,
    mut test_chord_just_released: EventWriter<TestChordJustReleased>
) {
    // TODO: Get Tab's parent Territory's parent Window entity id.
    // Until then, we get Window entity id of wherever we set off this stand-in chord.
    // These will be replaced by some kind of TabJustMoved(Tab) events.

    for (entity, window) in &window_query {

        if window.cursor_position().is_none() {continue}
        if dev_controls.just_pressed(&DevControls::TestChord) {
            test_chord_just_pressed.send(TestChordJustPressed(entity));
        }
        if dev_controls.pressed(&DevControls::TestChord) {
            test_chord_pressed.send(TestChordPressed(entity));
        }
        if dev_controls.just_released(&DevControls::TestChord) {
            test_chord_just_released.send(TestChordJustReleased(entity));
        }
    }
}
