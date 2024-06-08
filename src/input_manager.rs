use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::components_territory::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum DevControls {
    TestChord,
    TestSpawnNewWindow,
    TestRemoveAllTerritories
}
impl DevControls {
    pub fn default_input_map() -> InputMap<DevControls> {
        use KeyCode::*;
        InputMap::new([
            (Self::TestChord, UserInput::Chord(
                vec!(InputKind::PhysicalKey(ControlLeft), InputKind::PhysicalKey(ShiftLeft) ))),
            (Self::TestSpawnNewWindow, UserInput::Single(InputKind::PhysicalKey(KeyN))),
            (Self::TestRemoveAllTerritories, UserInput::Chord(
                vec!(InputKind::PhysicalKey(ShiftLeft), InputKind::PhysicalKey(KeyX) )))
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

#[derive(Event)]
pub struct RemoveTerritoriesKeyPressed;

// Send event when key pressed.
pub fn test_delete_all_territories_just_pressed (
    dev_controls: Res<ActionState<DevControls>>,
    mut remove_territories_key_pressed: EventWriter<RemoveTerritoriesKeyPressed>
) {
    if dev_controls.pressed(&DevControls::TestRemoveAllTerritories) {
        remove_territories_key_pressed.send(RemoveTerritoriesKeyPressed);
    }
}

// Remove all Territories when this key is pressed.
pub fn test_delete_all_territories (
    mut commands: Commands,
    mut remove_territories_key_pressed: EventReader<RemoveTerritoriesKeyPressed>,
    window_query: Query<&Children, With<Window>>,
    territory_query: Query<Entity, With<Territory>>
) {
    for _event in remove_territories_key_pressed.read() {
        for window_children in & window_query {
            let mut territories = territory_query.iter_many(window_children);
            while let Some(territory_entity) =  territories.fetch_next(){
                commands.entity(territory_entity).despawn_recursive();
            }
        }
    }
}

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
