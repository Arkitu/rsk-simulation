/// Bevy is only used to visualize the simulation
use bevy::{prelude::*, render::camera::ScalingMode, sprite::{MaterialMesh2dBundle, Mesh2dHandle}, window::WindowResolution};
use crate::game_controller::GameController;
use crate::game_state::GameState;
use crate::constants::*;
use crate::gui::GUITrait;

const WINDOW_SCALE: f32 = 400.0;
const FIELD_IMG: (f32, f32) = (9335., 7030.);

#[derive(Resource)]
struct BevyGameState (GameState);

#[derive(Resource)]
struct BevyGameController (GameController);

#[derive(Component)]
struct Ball;

#[derive(Component)]
enum Robot {
    Blue1,
    Blue2,
    Green1,
    Green2
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>
) {
    cmds.spawn(Camera2dBundle {
        transform: Transform::from_xyz(CARPET.0/2., CARPET.1/2., 10.),
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: CARPET.0,
                min_height: CARPET.1
            },
            ..default()
        },
        ..default()
    });

    // Spawn the field background
    cmds.spawn(SpriteBundle {
        texture: asset_server.load("field.jpg"),
        transform: Transform {
            translation: Vec3::new(CARPET.0/2., CARPET.1/2., 0.),
            scale: Vec3::splat(CARPET.0/FIELD_IMG.0),
            ..Default::default()
        },
        ..default()
    });

    // Spawn the ball
    cmds.spawn((MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle { radius: BALL_RADIUS })),
        material: color_materials.add(Color::rgb_u8(247, 107, 49)),
        transform: Transform::from_xyz(CARPET.0/2., CARPET.1/2., 1.),
        ..default()
    }, Ball));

    // Spawn the robots
    let hexagon = Mesh2dHandle(meshes.add(RegularPolygon::new(ROBOT_RADIUS, 6)));

    let blue = color_materials.add(Color::rgb_u8(0, 0, 255));
    cmds.spawn((MaterialMesh2dBundle {
        mesh: hexagon.clone(),
        material: blue.clone(),
        transform: Transform::from_xyz(DEFAULT_BLUE1_POS.x, DEFAULT_BLUE1_POS.y, 1.),
        ..default()
    }, Robot::Blue1));
    cmds.spawn((MaterialMesh2dBundle {
        mesh: hexagon.clone(),
        material: blue,
        transform: Transform::from_xyz(DEFAULT_BLUE2_POS.x, DEFAULT_BLUE2_POS.y, 1.),
        ..default()
    }, Robot::Blue2));

    let green = color_materials.add(Color::rgb_u8(0, 255, 0));
    cmds.spawn((MaterialMesh2dBundle {
        mesh: hexagon.clone(),
        material: green.clone(),
        transform: Transform::from_xyz(DEFAULT_GREEN1_POS.x, DEFAULT_GREEN1_POS.y, 1.),
        ..default()
    }, Robot::Green1));
    cmds.spawn((MaterialMesh2dBundle {
        mesh: hexagon,
        material: green,
        transform: Transform::from_xyz(DEFAULT_GREEN2_POS.x, DEFAULT_GREEN2_POS.y, 1.),
        ..default()
    }, Robot::Green2));

}

fn update_gs(
    mut gc: ResMut<BevyGameController>,
    mut gs: ResMut<BevyGameState>
) {
    gc.0.update_simu();
    gs.0 = gc.0.get_game_state();
}

fn move_objects(
    mut ball: Query<&mut Transform, With<Ball>>,
    mut robots: Query<(&Robot, &mut Transform), Without<Ball>>,
    gs: Res<BevyGameState>
) {
    let gs = &gs.0;
    
    *ball.single_mut() = Transform::from_xyz(gs.ball.x, gs.ball.y, 1.);

    for (r, mut pos) in robots.iter_mut() {
        let new_pos = match r {
            &Robot::Blue1 => &gs.markers.blue1,
            &Robot::Blue2 => &gs.markers.blue2,
            &Robot::Green1 => &gs.markers.green1,
            &Robot::Green2 => &gs.markers.green2
        };

        *pos = Transform::from_xyz(new_pos.position.x, new_pos.position.y, 1.);
    }
}

pub struct BevyGUI;
impl GUITrait for BevyGUI {
    fn run(gc: GameController) {
        let gs = gc.get_game_state();
        App::new()
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RSK Simulator".to_string(),
                    resolution: WindowResolution::new(CARPET.0*WINDOW_SCALE, CARPET.1*WINDOW_SCALE),
                    ..default()
                }),
                ..default()
            }))
            .insert_resource(Time::<Fixed>::from_seconds(DT as f64))
            .insert_resource(BevyGameController(gc))
            .insert_resource(BevyGameState(gs.clone()))
            .add_systems(Startup, setup)
            .add_systems(FixedUpdate, update_gs)
            .add_systems(Update, move_objects)
            .run();
    }
}