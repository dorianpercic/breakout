//! Shows how to render simple primitive shapes with a single color.

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::{WindowMode, WindowTheme},
};
const GRID_COLUMNS: usize = 7; // Number of bricks per row
const GRID_ROWS: usize = 7; // Number of rows of bricks
const BRICK_WIDTH: f32 = 80.0;
const BRICK_HEIGHT: f32 = 10.0;
const HORIZONTAL_SPACING: f32 = 10.0;
const VERTICAL_SPACING: f32 = 10.0;
const OFFSET_X: f32 = -270.0; // Adjust this to center the grid horizontally
const OFFSET_Y: f32 = 270.0; // Adjust this to position the grid vertically

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Breakout".into(),
                resolution: (800.0, 600.0).into(),
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
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let shapes: [Mesh2dHandle; 49] = core::array::from_fn(|_| {
        Mesh2dHandle(meshes.add(Rectangle::new(BRICK_WIDTH, BRICK_HEIGHT)))
    });
    let num_shapes = shapes.len();

    // Add the rectangles to the scene
    for (i, shape) in shapes.into_iter().enumerate() {
        let color = Color::hsl(360., 0.95, 0.7);

        let column = i % GRID_COLUMNS;
        let row = i / GRID_ROWS;

        let x_position = OFFSET_X + column as f32 * (BRICK_WIDTH + HORIZONTAL_SPACING);
        let y_position = OFFSET_Y - row as f32 * (BRICK_HEIGHT + VERTICAL_SPACING);

        commands.spawn(MaterialMesh2dBundle {
            mesh: shape.into(),
            material: materials.add(color),
            transform: Transform::from_xyz(x_position, y_position, 0.0),
            ..Default::default()
        });
    }
}
