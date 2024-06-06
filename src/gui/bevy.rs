use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use crate::constants::real::*;
use crate::game_controller::GC;
use crate::game_state::{GameState, Robot};
use crate::gui::GUITrait;
use bevy::log::LogPlugin;
use bevy::window::PrimaryWindow;
/// Bevy is only used to visualize the simulation
use bevy::{
    prelude::*,
    render::camera::ScalingMode,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResolution,
};
use rapier2d_f64::prelude::*;

const WINDOW_SCALE: f32 = 400. as f32;
const FIELD_IMG: (f32, f32) = (9335., 7030.);

struct BevyGC(GC);

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Kicker;

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    cmds.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 10.),
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: CARPET.0 as f32,
                min_height: CARPET.1 as f32,
            },
            ..default()
        },
        ..default()
    });

    // Spawn the field background
    cmds.spawn(SpriteBundle {
        texture: asset_server.load("field.jpg"),
        transform: Transform {
            translation: Vec3::new(0., 0., 0.),
            scale: Vec3::splat(CARPET.0  as f32 / FIELD_IMG.0),
            ..Default::default()
        },
        ..default()
    });

    // Spawn the ball
    cmds.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {
                radius: BALL_RADIUS as f32,
            })),
            material: color_materials.add(Color::rgb_u8(247, 107, 49)),
            transform: Transform::from_xyz(DEFAULT_BALL_POS.x  as f32, DEFAULT_BALL_POS.y  as f32, 1.),
            ..default()
        },
        Ball,
    ));

    // Spawn the robots
    let hexagon = Mesh2dHandle(meshes.add(RegularPolygon::new(ROBOT_RADIUS  as f32, 6)));
    let rect = Mesh2dHandle(meshes.add(Rectangle::new(KICKER_THICKNESS as f32, ROBOT_RADIUS as f32))); //ROBOT_RADIUS as f32 * 0.866, ROBOT_RADIUS as f32 * 0.5, (ROBOT_RADIUS as f32 * 0.866)+(KICKER_THICKNESS as f32), ROBOT_RADIUS as f32 * 0.5)));

    let blue = color_materials.add(Color::rgb_u8(0, 0, 255));
    let green = color_materials.add(Color::rgb_u8(0, 255, 0));
    let grey = color_materials.add(Color::rgb(0.5, 0.5, 0.5));
    for r in Robot::all() {
        let pos = DEFAULT_ROBOTS_POS[r as usize];
        let material = match r {
            Robot::Blue1 | Robot::Blue2 => blue.clone(),
            Robot::Green1 | Robot::Green2 => green.clone(),
        };
        cmds.spawn((
            MaterialMesh2dBundle {
                mesh: hexagon.clone(),
                material,
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 1.),
                ..default()
            },
            r
        )).with_children(|parent| {
            parent.spawn((MaterialMesh2dBundle {
                mesh: rect.clone(),
                material: grey.clone(),
                transform: Transform::from_xyz(ROBOT_RADIUS as f32 * 0.866, 0., 0.1),
                ..default()
            }, Kicker));
        });
    }
}

fn move_objects(
    mut ball: Query<&mut Transform, With<Ball>>,
    mut robots: Query<(&Robot, &mut Transform, &Children), Without<Ball>>,
    mut kickers: Query<&mut Transform, (With<Kicker>, Without<Ball>, Without<Robot>)>,
    gc: NonSendMut<BevyGC>,
) {
    let gs = gc.0.get_game_state();

    if let Some(ball_pos) = gs.ball {
        *ball.single_mut() = Transform::from_xyz(ball_pos.x as f32, ball_pos.y as f32, 1.);
    }

    for (r, mut pos, childs) in robots.iter_mut() {
        let new_pos = match r {
            Robot::Blue1 => &gs.markers.blue1,
            Robot::Blue2 => &gs.markers.blue2,
            Robot::Green1 => &gs.markers.green1,
            Robot::Green2 => &gs.markers.green2,
        };
        *pos = Transform::from_xyz(new_pos.position.x as f32, new_pos.position.y as f32, 1.).looking_to(Vec3::ZERO, Vec3::new((new_pos.orientation + (PI/2.)).cos() as f32, (new_pos.orientation + (PI/2.)).sin() as f32, 0.));
    
        let mut kicker_pos = kickers.get_mut(childs[0]).unwrap();

        let pose = gc.0.get_kicker_pose(*r);
        let d = ((pose.position.x - new_pos.position.x).powi(2) + (pose.position.y - new_pos.position.y).powi(2)).sqrt();


        *kicker_pos = Transform::from_xyz(d as f32, 0., 1.);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct Dragging(Option<RigidBodyHandle>);

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct Dragging(Rc<RefCell<Option<RigidBodyHandle>>>);

fn select_dragging(
    mut gc: NonSendMut<BevyGC>,
    mut dragging: NonSendMut<Dragging>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = q_windows.single().cursor_position() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let entity = gc.0.find_entity_at(bevy_to_simu(position));
                *dragging = Dragging(Some(entity.unwrap_or(gc.0.get_ball_handle())))
            }
            #[cfg(target_arch = "wasm32")]
            {
                gc.0.find_entity_at_rc(bevy_to_simu(position), dragging.0.clone(), Some(gc.0.get_ball_handle()));
            }
        }
    } else if !buttons.pressed(MouseButton::Left) { // better in async because find_entity_at_rc can update dragging after mouse release
        #[cfg(not(target_arch = "wasm32"))]
        {
            *dragging = Dragging(None);
        }
        #[cfg(target_arch = "wasm32")]
        {
            *(*dragging.0).borrow_mut() = None
        }
    }
}

fn update_dragging(
    mut gc: NonSendMut<BevyGC>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    dragging: NonSend<Dragging>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    let entity = dragging.0;
    #[cfg(target_arch = "wasm32")]
    let entity = *dragging.0.borrow();
    match entity {
        None => (),
        Some(d) => {
            if let Some(position) = q_windows.single().cursor_position() {
                gc.0.teleport_entity(d, bevy_to_simu(position), None);
            }
        }
    }
}

fn reset(
    mut gc: NonSendMut<BevyGC>,
    keys: Res<ButtonInput<KeyCode>>
) {
    if keys.just_pressed(KeyCode::KeyR) {
        gc.0.reset();
    }
}
fn kick(
    mut gc: NonSendMut<BevyGC>,
    keys: Res<ButtonInput<KeyCode>>
) {
    if keys.just_pressed(KeyCode::KeyK) {
        for r in Robot::all() {
            gc.0.kick(r, 1.);
        }
    }
}

fn bevy_to_simu(pos: Vec2) -> Point<f64> {
    Point::new((pos.x / WINDOW_SCALE) as f64 - (CARPET.0/2.), -((pos.y / WINDOW_SCALE) as f64 - (CARPET.1/2.)))
}

fn simu_to_bevy(pos: Point<f64>) -> Vec2 {
    Vec2::new((pos.x + (CARPET.0/2.)) as f32 * WINDOW_SCALE, (pos.y + (CARPET.1/2.)) as f32 * WINDOW_SCALE)
}

pub struct BevyGUI;
impl GUITrait for BevyGUI {
    fn run(gc: GC) {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RSK Simulator".to_string(),
                        resolution: WindowResolution::new(
                            CARPET.0 as f32 * WINDOW_SCALE,
                            CARPET.1 as f32 * WINDOW_SCALE,
                        ),
                        ..default()
                    }),
                    ..default()
                }).disable::<LogPlugin>()
            )
            .insert_resource(Time::<Fixed>::from_seconds(DT as f64))
            .add_systems(Startup, setup)
            .add_systems(Update, move_objects)
            .add_systems(Update, select_dragging)
            .add_systems(Update, update_dragging)
            .add_systems(Update, reset)
            .add_systems(Update, kick)
            // BevyGC and Dragging are NonSend with http_client_gc to it's simpler if they always are
            .insert_non_send_resource(BevyGC(gc))
            .insert_non_send_resource(Dragging::default());

        app.run()
    }
}
