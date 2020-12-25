use bevy::prelude::*;
use rand::random;
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
struct SnakeHead;
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

struct Food;

struct FoodSpawnTimer(Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), true))
    }
}

struct EatEvent {
    eater: Entity,
    eaten: Entity,
}

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
    let snake = spawn_snake(commands, &materials, Position { x: 0, y: 0 });
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
    mut head_positions: Query<&mut Position, With<SnakeHead>>,
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
            ..Default::default()
        })
        .with(position)
        .with(Size::square(0.8))
        .with(Snake)
        .with(SnakeHead)
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
            ..Default::default()
        })
        .with(position)
        .with(Size::square(0.65))
        .with(Snake)
        .current_entity()
        .unwrap()
}

fn spawn_snake(commands: &mut Commands, materials: &Materials, position: Position) -> Entity {
    let snake = spawn_head(commands, materials.head_material.clone(), position);

    let mut pos = position;
    let mut segments = vec![None, Some(snake)];
    for _ in 0..3 {
        pos.x += 1;
        segments.push(Some(spawn_segment(
            commands,
            materials.body_material.clone(),
            pos,
        )));
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

fn spawn_food(commands: &mut Commands, material: Handle<ColorMaterial>, position: Position) {
    commands
        .spawn(SpriteBundle {
            material,
            ..Default::default()
        })
        .with(Food)
        .with(position)
        .with(Size::square(0.4));
}

fn food_spawner(
    commands: &mut Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
) {
    timer.0.tick(time.delta_seconds());
    if !timer.0.finished() {
        return;
    }
    let pos = Position {
        x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
        y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
    };
    spawn_food(commands, materials.food_material.clone(), pos);
}

fn collision_solver(
    timer: Res<TurnTimer>,
    heads_positions: Query<(Entity, &Position), With<SnakeHead>>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    mut eat_events: ResMut<Events<EatEvent>>,
) {
    if !timer.0.finished() {
        return;
    }
    for (e1, p1) in heads_positions.iter() {
        for (e2, p2) in food_positions.iter() {
            if p1 == p2 {
                eat_events.send(EatEvent {
                    eater: e1,
                    eaten: e2,
                });
            }
        }
    }
}

fn get_tail(head: Entity, q: &mut Query<(Entity, &mut SnakeSegment)>) -> Entity {
    let mut tail = head;
    while let Ok((e, seg)) = q.get_mut(tail) {
        if let Some(t) = seg.back {
            tail = t;
        } else {
            break;
        }
    }
    tail
}

fn eat_events_solver(
    commands: &mut Commands,
    mut segments: Query<(Entity, &mut SnakeSegment)>,
    positions: Query<&Position, With<SnakeSegment>>,
    eat_events: Res<Events<EatEvent>>,
    mut eat_reader: Local<EventReader<EatEvent>>,
    materials: Res<Materials>,
) {
    while let Some(EatEvent { eater, eaten }) = eat_reader.iter(&eat_events).next() {
        let tail = get_tail(*eater, &mut segments);
        let tail_pos = positions.get(tail).unwrap();
        let new_tail = spawn_segment(commands, materials.body_material.clone(), *tail_pos);
        commands.with(SnakeSegment {
            front: Some(tail),
            back: None,
        });
        let (_, mut tail_seg) = segments.get_mut(tail).unwrap();
        tail_seg.back = Some(new_tail);
        commands.despawn(*eaten);
    }
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Snake!".to_owned(),
            width: 400.,
            height: 400.,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_stage(
            "game_setup",
            SystemStage::serial().with_system(game_setup.system()),
        )
        .add_event::<EatEvent>()
        .add_system(turn_timer.system())
        .add_system(segment_movement.system())
        .add_system(snake_movement.system())
        .add_system(food_spawner.system())
        .add_system(collision_solver.system())
        .add_system(eat_events_solver.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .run();
}
