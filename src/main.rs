use bevy::prelude::*;
use std::time::Duration;

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

struct Snake;
struct SnakeSegment {
    prev: Option<Entity>,
}

struct Materials {
    head_material: Handle<ColorMaterial>,
    body_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

struct Player {
    snake: Entity,
}

struct TurnTimer(Timer);

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(Materials {
        head_material: materials
            .add(ColorMaterial {
                color: Color::rgb(0.7, 0.7, 0.7),
                texture: None,
            })
            .into(),
        body_material: materials
            .add(ColorMaterial {
                color: Color::rgb(0.3, 0.3, 0.3),
                texture: None,
            })
            .into(),
        food_material: materials
            .add(ColorMaterial {
                color: Color::rgb(1.0, 0.0, 1.0),
                texture: None,
            })
            .into(),
    });
    commands.insert_resource(TurnTimer(Timer::new(Duration::from_millis(150), true)));
}

fn game_setup(commands: &mut Commands, materials: Res<Materials>) {
    spawn_head(
        commands,
        materials.head_material.clone(),
        Vec2::new(10.0, 10.0),
    );
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.,
        );
    }
}

fn snake_movement(
    timer: Res<TurnTimer>,
    keys: Res<Input<KeyCode>>,
    player: Res<Player>,
    mut head_positions: Query<&mut Position, With<SnakeSegment>>,
) {
    if !timer.0.finished() {
        return;
    }
    let mut player_head_pos = head_positions.get_mut(player.snake).unwrap();
    if keys.pressed(KeyCode::Left) {
        player_head_pos.x -= 1;
    }
    if keys.pressed(KeyCode::Right) {
        player_head_pos.x += 1;
    }
    if keys.pressed(KeyCode::Down) {
        player_head_pos.y -= 1;
    }
    if keys.pressed(KeyCode::Up) {
        player_head_pos.y += 1;
    }
}

fn turn_timer(time: Res<Time>, mut timer: ResMut<TurnTimer>) {
    timer.0.tick(time.delta_seconds());
}

fn spawn_head(commands: &mut Commands, material: Handle<ColorMaterial>, position: Vec2) {
    let snake = commands
        .spawn(SpriteBundle {
            material,
            sprite: Sprite::new(position),
            ..Default::default()
        })
        .with(Position { x: 3, y: 3 })
        .with(Size::square(0.8))
        .with(Snake)
        .with(SnakeSegment { prev: None })
        .current_entity()
        .unwrap();
    commands.insert_resource(Player { snake });
}

fn spawn_segment(commands: &mut Commands, material: Handle<ColorMaterial>, position: Vec2) {
    commands
        .spawn(SpriteBundle {
            material,
            sprite: Sprite::new(position),
            ..Default::default()
        })
        .with(Snake);
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_stage(
            "game_setup",
            SystemStage::serial().with_system(game_setup.system()),
        )
        .add_system(turn_timer.system())
        .add_system(snake_movement.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .run();
}
