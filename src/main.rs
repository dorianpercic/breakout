use bevy::{
    ecs::event::Event,
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
const PLAYER_TILE_SPEED: f32 = 550.0;
const SCREEN_WIDTH: f32 = 600.0;
const SCREEN_HEIGHT: f32 = 800.0;
const BALL_RADIUS: f32 = 5.0;
const TILE_SIZE: Vec2 = Vec2::new(BRICK_WIDTH, BRICK_HEIGHT);
const BALL_SIZE: Vec2 = Vec2::new(BALL_RADIUS * 2.0, BALL_RADIUS * 2.0);
const BALL_VELOCITY: f32 = 220.0;
const MAX_BOUNCE_ANGLE: f32 = 0.45;

// Components
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct HitTile;

#[derive(Component)]
struct MoveDirection(Vec3);

// Events
#[derive(Event)]
struct WallHitEvent;

#[derive(Event)]
struct CeilingHitEvent;

#[derive(Event)]
struct FloorHitEvent;

#[derive(Event)]
struct PlayerHitEvent {
    intersection_x: f32,
}

#[derive(Event)]
struct TileHitEvent {
    tile_entity: Entity,
}

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
        .add_event::<WallHitEvent>()
        .add_event::<CeilingHitEvent>()
        .add_event::<PlayerHitEvent>()
        .add_event::<TileHitEvent>()
        .add_event::<FloorHitEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_player,
                move_ball,
                handle_collisions,
                handle_wall_hit_events,
                handle_ceiling_hit_events,
                handle_player_hit_events,
                handle_tile_hit_events,
                handle_floor_hit_events,
            ),
        )
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
            transform: Transform::from_xyz(0.0, -50.0, 0.0),
            ..Default::default()
        },
        Ball,
        MoveDirection(Vec3::new(0.0, -1.0, 0.0)),
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
            direction.x -= 0.5;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 0.5;
        }

        let move_translation =
            transform.translation + (time.delta_seconds() * PLAYER_TILE_SPEED * direction);

        if move_translation.x < (-SCREEN_WIDTH / 2.0) - (BRICK_WIDTH / 1.5)
            || move_translation.x > (SCREEN_WIDTH / 2.0) + (BRICK_WIDTH / 1.5)
        {
            info!("Player reached left/right boundary");
            return;
        }
        transform.translation = move_translation;
    }
}

fn move_ball(
    time: Res<Time>,
    mut ball_query: Query<(&mut Transform, &mut MoveDirection), With<Ball>>,
) {
    for (mut ball_pos_transform, ball_direction) in ball_query.iter_mut() {
        ball_pos_transform.translation += ball_direction.0 * time.delta_seconds() * BALL_VELOCITY;
    }
}

fn handle_collisions(
    mut param_set: ParamSet<(
        Query<&mut Transform, With<Ball>>,
        Query<&Transform, With<Player>>,
        Query<(Entity, &Transform), With<HitTile>>,
    )>,
    mut wall_hit_event_writer: EventWriter<WallHitEvent>,
    mut ceiling_hit_event_writer: EventWriter<CeilingHitEvent>,
    mut floor_hit_event_writer: EventWriter<FloorHitEvent>,
    mut player_hit_event_writer: EventWriter<PlayerHitEvent>,
    mut tile_hit_event_writer: EventWriter<TileHitEvent>,
) {
    let player_transform = param_set.p1();
    if player_transform.iter().count() != 1 {
        return;
    }
    let player_position = player_transform.single().translation;
    let hit_tile_positions: Vec<(Entity, Vec3)> = param_set
        .p2()
        .iter()
        .map(|(e, t)| (e, t.translation))
        .collect();

    for mut ball_pos_transform in param_set.p0().iter_mut() {
        if ball_pos_transform.translation.y >= SCREEN_HEIGHT / 2.6 {
            ball_pos_transform.translation.y -= 2.0 * BALL_RADIUS;
            ceiling_hit_event_writer.send(CeilingHitEvent);
        } else if ball_pos_transform.translation.y <= -SCREEN_HEIGHT / 2.6 {
            floor_hit_event_writer.send(FloorHitEvent);
        }

        if ball_pos_transform.translation.x <= -SCREEN_WIDTH / 1.53 {
            ball_pos_transform.translation.x += 1.4 * BALL_RADIUS;
            wall_hit_event_writer.send(WallHitEvent);
        } else if ball_pos_transform.translation.x >= SCREEN_WIDTH / 1.53 {
            ball_pos_transform.translation.x -= 1.4 * BALL_RADIUS;
            wall_hit_event_writer.send(WallHitEvent);
        }

        if check_collision(
            ball_pos_transform.translation.truncate(),
            BALL_SIZE,
            player_position.truncate(),
            TILE_SIZE,
        ) {
            let intersection_x = ball_pos_transform.translation.x;
            let player_left = player_position.x - TILE_SIZE.x / 2.0;
            let player_right = player_position.x + TILE_SIZE.x / 2.0;

            let normalized_intersection =
                (intersection_x - player_left) / (player_right - player_left) * 2.0 - 1.0;

            player_hit_event_writer.send(PlayerHitEvent {
                intersection_x: normalized_intersection,
            });
        }

        for (tile_entity, hit_tile_position) in &hit_tile_positions {
            if check_collision(
                ball_pos_transform.translation.truncate(),
                BALL_SIZE,
                hit_tile_position.truncate(),
                TILE_SIZE,
            ) {
                tile_hit_event_writer.send(TileHitEvent {
                    tile_entity: *tile_entity,
                });
            }
        }
    }
}

fn handle_player_hit_events(
    mut player_hit_event_reader: EventReader<PlayerHitEvent>,
    mut param_set: ParamSet<(
        Query<&mut Transform, With<Ball>>,
        Query<&mut MoveDirection, With<Ball>>,
    )>,
) {
    for event in player_hit_event_reader.read() {
        info!("Player collision");
        for mut transform in param_set.p0().iter_mut() {
            transform.translation.y -= -2.0 * BALL_RADIUS;
        }
        for mut move_direction in param_set.p1().iter_mut() {
            move_direction.0.y *= -1.0;
            move_direction.0.x += event.intersection_x * MAX_BOUNCE_ANGLE;
        }
    }
}

fn handle_wall_hit_events(
    mut wall_hit_event_reader: EventReader<WallHitEvent>,
    mut query: Query<&mut MoveDirection, With<Ball>>,
) {
    for _event in wall_hit_event_reader.read() {
        info!("Wall collision");
        for mut move_direction in query.iter_mut() {
            move_direction.0.x *= -1.0;
        }
    }
}

fn handle_ceiling_hit_events(
    mut ceiling_hit_event_reader: EventReader<CeilingHitEvent>,
    mut param_set: ParamSet<(
        Query<&mut Transform, With<Ball>>,
        Query<&mut MoveDirection, With<Ball>>,
    )>,
) {
    for _event in ceiling_hit_event_reader.read() {
        info!("Ceiling collision");
        for mut move_direction in param_set.p1().iter_mut() {
            move_direction.0.y *= -1.0;
        }
    }
}

fn handle_floor_hit_events(
<<<<<<< HEAD
    mut commands: Commands,
    mut floor_hit_event_reader: EventReader<FloorHitEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<Entity, With<Player>>,
    hit_tile_query: Query<Entity, With<HitTile>>,
    ball_query: Query<Entity, With<Ball>>,
) {
    // Restart game
    for _event in floor_hit_event_reader.read() {
        info!("Floor collision");
        for ent in player_query.iter() {
            commands.entity(ent).despawn();
        }
        for ent in hit_tile_query.iter() {
            commands.entity(ent).despawn();
        }

        for ent in ball_query.iter() {
            commands.entity(ent).despawn();
        }

        spawn_ball(&mut commands, &mut meshes, &mut materials);
        spawn_player(&mut commands, &mut meshes, &mut materials);
        spawn_hit_tiles(&mut commands, &mut meshes, &mut materials);
=======
    mut floor_hit_event_reader: EventReader<FloorHitEvent>,
    mut param_set: ParamSet<(
        Query<&mut Transform, With<Ball>>,
        Query<&mut MoveDirection, With<Ball>>,
    )>,
) {
    for _event in floor_hit_event_reader.read() {
        info!("Floor collision");
        for mut move_direction in param_set.p1().iter_mut() {
            // One life lost
        }
>>>>>>> fd49594 (Implement life system)
    }
}

fn handle_tile_hit_events(
    mut commands: Commands,
    mut tile_hit_event_reader: EventReader<TileHitEvent>,
    mut param_set: ParamSet<(
        Query<&mut Transform, With<Ball>>,
        Query<&mut MoveDirection, With<Ball>>,
        Query<&mut MoveDirection, With<Ball>>,
    )>,
) {
    for event in tile_hit_event_reader.read() {
        info!("Tile collision");
        for mut transform in param_set.p0().iter_mut() {
            transform.translation.y -= 2.0 * BALL_RADIUS;
        }
        for mut move_direction in param_set.p1().iter_mut() {
            move_direction.0.y *= -1.0;
        }

        // Despawn the tile that was hit
        commands.entity(event.tile_entity).despawn();
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
