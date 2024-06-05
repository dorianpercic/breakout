//! Shows how to render simple primitive shapes with a single color.

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::{WindowMode, WindowTheme},
};

const GRID_COLUMNS: usize = 7;
const GRID_ROWS: usize = 7;
const BRICK_WIDTH: f32 = 80.0;
const BRICK_HEIGHT: f32 = 10.0;
const HORIZONTAL_SPACING: f32 = 10.0;
const VERTICAL_SPACING: f32 = 10.0;
const PADDING_X: f32 = -270.0;
const PADDING_Y: f32 = 270.0;
const PLAYER_TILE_SPEED: f32 = 250.0;
const SCREEN_WIDTH: f32 = 600.0;
const SCREEN_HEIGHT: f32 = 800.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Breakout".into(),
                resolution: (SCREEN_HEIGHT, SCREEN_WIDTH).into(),
                mode: WindowMode::Windowed,
                resizable: false,
                window_theme: Some(WindowTheme::Dark),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                ..default()
            }),
            ..WindowPlugin::default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, move_player_tile)
        .run();
}

fn move_player_tile(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut sprite_position: Query<&mut Transform, With<Player>>,
) {
    for mut transform in sprite_position.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }
        let move_translation =
            transform.translation + (time.delta_seconds() * PLAYER_TILE_SPEED * direction);

        if move_translation.x < -SCREEN_WIDTH / 2.0 || move_translation.x > SCREEN_WIDTH / 2.0 {
            info!("Player reached left/right boundary");
            return;
        }
        transform.translation = move_translation;
    }
}

#[derive(Component)]
struct Player;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let hit_tiles: [Mesh2dHandle; 49] = core::array::from_fn(|_| {
        Mesh2dHandle(meshes.add(Rectangle::new(BRICK_WIDTH, BRICK_HEIGHT)))
    });
    let player_tile: Mesh2dHandle =
        Mesh2dHandle(meshes.add(Rectangle::new(BRICK_WIDTH, BRICK_HEIGHT)));

    // Add the hit tiles to the scene
    for (i, shape) in hit_tiles.into_iter().enumerate() {
        let color = Color::hsl(360., 0.95, 0.7);

        let column = i % GRID_COLUMNS;
        let row = i / GRID_ROWS;

        let x_position = PADDING_X + column as f32 * (BRICK_WIDTH + HORIZONTAL_SPACING);
        let y_position = PADDING_Y - row as f32 * (BRICK_HEIGHT + VERTICAL_SPACING);

        commands.spawn(MaterialMesh2dBundle {
            mesh: shape.into(),
            material: materials.add(color),
            transform: Transform::from_xyz(x_position, y_position, 0.0),
            ..Default::default()
        });
    }
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: player_tile.into(),
            material: materials.add(Color::hsl(180., 0.45, 0.8)),
            transform: Transform::from_xyz(0.0, -PADDING_Y, 0.0),
            ..Default::default()
        },
        Player,
    ));
}
