/// Bevy is only used to visualize the simulation
use bevy::{prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}};
use crate::{game_state::GameState, gui::GUITrait, BALL_RADIUS, FIELD_HEIGHT, FIELD_LENGTH, MARGIN};
use std::{sync::{Arc, Mutex}, thread};

const SCALE: f32 = 100.;

#[derive(Resource)]
struct BevyGameState (Arc<Mutex<Option<GameState>>>);

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Robot;

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    cmds.spawn(Camera2dBundle::default());

    cmds.spawn((MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle { radius: BALL_RADIUS })),
        material: materials.add(Color::rgb_u8(247, 107, 49)),
        transform: Transform::from_xyz(MARGIN+(FIELD_LENGTH/2.), MARGIN+(FIELD_HEIGHT/2.), 1.),
        ..default()
    }, Ball));
}

pub struct BevyGUI {
    game_state: Arc<Mutex<Option<GameState>>>
}
impl GUITrait for BevyGUI {
    fn new() -> Self {
        let game_state = Arc::new(Mutex::new(None));
        let gs = game_state.clone();
        thread::spawn(move || {
            App::new()
            .add_plugins(DefaultPlugins)
            .insert_resource(BevyGameState(gs))
            .add_systems(Startup, setup)
            .run();
        });
        Self {
            game_state
        }
    }
    fn update(&mut self, game_state: &GameState) {
        *self.game_state.lock().unwrap() = Some(game_state.clone())
    }
}