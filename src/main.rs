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
const BALL_RADIUS: f32 = 5.0;
const TILE_SIZE: Vec2 = Vec2::new(BRICK_WIDTH, BRICK_HEIGHT);
const BALL_SIZE: Vec2 = Vec2::new(BALL_RADIUS * 2.0, BALL_RADIUS * 2.0);
const BALL_VELOCITY: f32 = 220.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct HitTile;

#[derive(Component)]
struct Velocity(Vec3);

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
        .add_systems(Update, move_player)
        .add_systems(Update, move_ball)
        .add_systems(Update, handle_collisions)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    spawn_hit_tiles(&mut commands, &mut meshes, &mut materials);
    spawn_player(&mut commands, &mut meshes, &mut materials);
    spawn_ball(&mut commands, &mut meshes, &mut materials);
}

fn spawn_hit_tiles(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let hit_tiles: [Mesh2dHandle; GRID_COLUMNS * GRID_ROWS] = core::array::from_fn(|_| {
        Mesh2dHandle(meshes.add(Mesh::from(Rectangle::new(BRICK_WIDTH, BRICK_HEIGHT))))
    });

    for (i, shape) in hit_tiles.into_iter().enumerate() {
        let color = Color::hsl(360., 0.95, 0.7);

        let column = i % GRID_COLUMNS;
        let row = i / GRID_ROWS;

        let x_position = PADDING_X + column as f32 * (BRICK_WIDTH + HORIZONTAL_SPACING);
        let y_position = PADDING_Y - row as f32 * (BRICK_HEIGHT + VERTICAL_SPACING);

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: shape.into(),
                material: materials.add(color),
                transform: Transform::from_xyz(x_position, y_position, 0.0),
                ..Default::default()
            },
            HitTile,
        ));
    }
}

fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let player_tile: Mesh2dHandle =
        Mesh2dHandle(meshes.add(Mesh::from(Rectangle::new(BRICK_WIDTH, BRICK_HEIGHT))));

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

fn spawn_ball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let ball: Mesh2dHandle = Mesh2dHandle(meshes.add(Mesh::from(Circle::new(BALL_RADIUS))));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: ball.into(),
            material: materials.add(Color::hsl(180., 0.7, 0.75)),
            transform: Transform::from_xyz(0.0, -150.0, 0.0),
            ..Default::default()
        },
        Ball,
        Velocity(Vec3::new(0.0, -1.0, 0.0)),
    ));
}

fn move_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    for mut transform in player_query.iter_mut() {
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

fn move_ball(time: Res<Time>, mut ball_query: Query<(&mut Transform, &mut Velocity), With<Ball>>) {
    for (mut transform, velocity) in ball_query.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds() * BALL_VELOCITY;
    }
}

fn handle_collisions(
    mut param_set: ParamSet<(
        Query<(&mut Transform, &mut Velocity), With<Ball>>,
        Query<&Transform, With<Player>>,
        Query<&Transform, With<HitTile>>,
    )>,
) {
    let player_transform = param_set.p1();
    let player_position = player_transform.single().translation;
    let hit_tile_positions: Vec<Vec3> = param_set.p2().iter().map(|t| t.translation).collect();

    for (transform, mut velocity) in param_set.p0().iter_mut() {
        // Check collision with player
        if check_collision(
            transform.translation.truncate(),
            BALL_SIZE,
            player_position.truncate(),
            TILE_SIZE,
        ) {
            info!("Ball collided with player!");
            velocity.0.y = 1.0;
        }

        // Check collision with hit tiles
        for hit_tile_position in &hit_tile_positions {
            if check_collision(
                transform.translation.truncate(),
                BALL_SIZE,
                hit_tile_position.truncate(),
                TILE_SIZE,
            ) {
                info!("Ball collided with tile!");
                velocity.0.y = -1.0;
                break; // Handle one collision per frame for simplicity
            }
        }
    }
}

fn check_collision(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    let half_size1 = size1 / 2.0;
    let half_size2 = size2 / 2.0;

    let min1 = pos1 - half_size1;
    let max1 = pos1 + half_size1;

    let min2 = pos2 - half_size2;
    let max2 = pos2 + half_size2;

    min1.x < max2.x && max1.x > min2.x && min1.y < max2.y && max1.y > min2.y
}
