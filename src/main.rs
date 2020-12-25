use bevy::prelude::*;
use std::time::Duration;

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
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

#[derive(Copy, Clone, Eq, PartialEq)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}
impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

struct Snake;
struct SnakeSegment {
    front: Option<Entity>,
    back: Option<Entity>,
}

struct Materials {
    head_material: Handle<ColorMaterial>,
    body_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

struct Player {
    snake: Entity,
    direction: Direction,
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
    let snake = spawn_snake(
        commands,
        materials.head_material.clone(),
        Position { x: 0, y: 0 },
    );
    commands.insert_resource(Player {
        snake,
        direction: Direction::Up,
    });
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
    mut player: ResMut<Player>,
    mut head_positions: Query<&mut Position, With<SnakeSegment>>,
) {
    let dir = if keys.pressed(KeyCode::Left) {
        Direction::Left
    } else if keys.pressed(KeyCode::Right) {
        Direction::Right
    } else if keys.pressed(KeyCode::Down) {
        Direction::Down
    } else if keys.pressed(KeyCode::Up) {
        Direction::Up
    } else {
        player.direction
    };

    if dir != player.direction.opposite() {
        player.direction = dir;
    }

    if !timer.0.finished() {
        return;
    }

    let mut player_head_pos = head_positions.get_mut(player.snake).unwrap();
    match player.direction {
        Direction::Left => {
            player_head_pos.x -= 1;
        }
        Direction::Right => {
            player_head_pos.x += 1;
        }
        Direction::Down => {
            player_head_pos.y -= 1;
        }
        Direction::Up => {
            player_head_pos.y += 1;
        }
    }
}

fn segment_movement(timer: Res<TurnTimer>, mut q: Query<(&mut Position, &SnakeSegment)>) {
    if !timer.0.finished() {
        return;
    }

    let heads: Vec<_> = q
        .iter_mut()
        .filter(|(_, s)| s.front.is_none())
        .map(|(p, s)| (s.back, p.clone()))
        .collect();
    for (mut e, mut p) in heads {
        while let Some(es) = e {
            let oldp = *q.get_component::<Position>(es).unwrap();
            q.set::<Position>(es, p).unwrap();
            p = oldp;
            e = q.get_component::<SnakeSegment>(es).unwrap().back;
        }
    }
}

fn turn_timer(time: Res<Time>, mut timer: ResMut<TurnTimer>) {
    timer.0.tick(time.delta_seconds());
}

fn spawn_head(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            material,
            sprite: Sprite::new(Vec2::new(0., 0.)),
            ..Default::default()
        })
        .with(position)
        .with(Size::square(0.8))
        .with(Snake)
        .current_entity()
        .unwrap()
}

fn spawn_segment(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            material,
            sprite: Sprite::new(Vec2::new(0., 0.)),
            ..Default::default()
        })
        .with(position)
        .with(Size::square(0.65))
        .with(Snake)
        .current_entity()
        .unwrap()
}

fn spawn_snake(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    let snake = spawn_head(commands, material.clone(), position);

    let mut pos = position;
    let mut segments = vec![None, Some(snake)];
    for _ in 0..3 {
        pos.x += 1;
        segments.push(Some(spawn_segment(commands, material.clone(), pos)));
    }
    segments.push(None);
    for w in segments.windows(3) {
        let seg = w[1].unwrap();
        commands.set_current_entity(seg);
        commands.with(SnakeSegment {
            front: w[0],
            back: w[2],
        });
    }
    snake
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
        .add_system(segment_movement.system())
        .add_system(snake_movement.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .run();
}
