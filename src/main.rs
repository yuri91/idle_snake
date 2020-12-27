use bevy::prelude::*;
use bevy::core::FixedTimestep;
use bevy::diagnostic::*;
use bevy::app::AppExit;
use rand::seq::IteratorRandom;
use std::time::Duration;
use std::collections::HashSet;

const ARENA_WIDTH: u32 = 15;
const ARENA_HEIGHT: u32 = 15;
const ARENA_MARGIN: f32 = 50.;

const FIXED_TIMESTEP: f64 = 0.15;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    board_material: Handle<ColorMaterial>,
}

struct Player {
    snake: Entity,
    direction: Direction,
    food: u32,
}

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
struct BumpEvent {
    head: Entity,
    wall: Entity,
}

struct LastInput {
    direction: Direction,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameState {
    Playing,
    Paused,
    Lost,
}

struct FpsText;
struct FoodText;

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(CameraUiBundle::default());
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
        board_material: materials
            .add(ColorMaterial {
                color: Color::rgb(1.0, 1.0, 1.0),
                texture: None,
            })
            .into(),
    });
    commands.spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(10.),
                    right: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                value: "FPS:".to_string(),
                font: asset_server.load("fonts/DejaVuSans.ttf"),
                style: TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(FpsText);
    commands.spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(10.),
                    left: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                value: "Food:".to_string(),
                font: asset_server.load("fonts/DejaVuSans.ttf"),
                style: TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(FoodText);
    commands.spawn(NodeBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(0.),
                left: Val::Px(0.),
                ..Default::default()
            },
            size: bevy::prelude::Size {
                width: Val::Percent(100.),
                height: Val::Px(ARENA_MARGIN),
            },
            ..Default::default()
        },
        material: materials.add(ColorMaterial {
            color: Color::rgb(0., 0., 0.),
            texture: None,
        }),
        ..Default::default()
    });
    commands.spawn(NodeBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(0.),
                left: Val::Px(0.),
                ..Default::default()
            },
            size: bevy::prelude::Size {
                width: Val::Px(ARENA_MARGIN),
                height: Val::Percent(100.),
            },
            ..Default::default()
        },
        material: materials.add(ColorMaterial {
            color: Color::rgb(0., 0., 0.),
            texture: None,
        }),
        ..Default::default()
    });
    commands.spawn(NodeBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(0.),
                right: Val::Px(0.),
                ..Default::default()
            },
            size: bevy::prelude::Size {
                width: Val::Percent(100.),
                height: Val::Px(ARENA_MARGIN),
            },
            ..Default::default()
        },
        material: materials.add(ColorMaterial {
            color: Color::rgb(0., 0., 0.),
            texture: None,
        }),
        ..Default::default()
    });
    commands.spawn(NodeBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(0.),
                right: Val::Px(0.),
                ..Default::default()
            },
            size: bevy::prelude::Size {
                width: Val::Px(ARENA_MARGIN),
                height: Val::Percent(100.),
            },
            ..Default::default()
        },
        material: materials.add(ColorMaterial {
            color: Color::rgb(0., 0., 0.),
            texture: None,
        }),
        ..Default::default()
    });

}

fn game_setup(commands: &mut Commands, materials: Res<Materials>) {
    let snake = spawn_snake(commands, &materials, Position { x: 0, y: 0 });
    commands.insert_resource(Player {
        snake,
        direction: Direction::Up,
        food: 0,
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
        let bound_window_margin = bound_window - 2.*ARENA_MARGIN;
        let tile_size = bound_window_margin / bound_game;
        pos / bound_game * bound_window_margin - (bound_window / 2.) + (tile_size / 2.) + ARENA_MARGIN
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        let z = transform.translation.z;
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            z,
        );
    }
}

fn input_events_sender(
    keys: Res<Input<KeyCode>>,
    mut last_input: ResMut<LastInput>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut gamestate: ResMut<State<GameState>>,
) {
    if keys.pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
    if keys.pressed(KeyCode::Space) {
        if *gamestate.current() == GameState::Paused {
            gamestate.set_next(GameState::Playing).unwrap();
        } else if *gamestate.current() == GameState::Playing {
            gamestate.set_next(GameState::Paused).unwrap();
        }
    }
    let direction = if keys.pressed(KeyCode::Left) {
        Direction::Left
    } else if keys.pressed(KeyCode::Right) {
        Direction::Right
    } else if keys.pressed(KeyCode::Down) {
        Direction::Down
    } else if keys.pressed(KeyCode::Up) {
        Direction::Up
    } else {
        return;
    };
    last_input.direction = direction;
}

fn snake_movement(
    last_input: Res<LastInput>,
    mut player: ResMut<Player>,
    mut head_positions: Query<&mut Position, With<SnakeHead>>,
) {
    if last_input.direction != player.direction.opposite() {
        player.direction = last_input.direction;
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
    if player_head_pos.x < 0 {
        player_head_pos.x = ARENA_WIDTH as i32 - 1;
    } else if player_head_pos.x >= ARENA_WIDTH as i32 {
        player_head_pos.x = 0;
    }
    if player_head_pos.y < 0 {
        player_head_pos.y = ARENA_HEIGHT as i32 - 1;
    } else if player_head_pos.y >= ARENA_HEIGHT as i32 {
        player_head_pos.y = 0;
    }
}

fn segment_movement(mut q: Query<(&mut Position, &SnakeSegment)>) {
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

fn spawn_head(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            material,
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
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
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
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
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..Default::default()
        })
        .with(Food)
        .with(position)
        .with(Size::square(0.4));
}

fn food_spawner(
    commands: &mut Commands,
    occupied: Query<&Position>,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
) {
    timer.0.tick(time.delta_seconds()+ FIXED_TIMESTEP as f32);
    if !timer.0.finished() {
        return;
    }
    let mut grid = std::collections::HashSet::new();
    for x in 0..ARENA_WIDTH as i32 {
        for y in 0..ARENA_HEIGHT as i32 {
            grid.insert(Position{x,y});
        }
    }
    let occupied: HashSet<Position> = occupied.iter().cloned().collect();
    let free = grid.difference(&occupied);
    let pos = free.into_iter().choose(&mut rand::thread_rng());
    if let Some(pos) = pos {
        spawn_food(commands, materials.food_material.clone(), *pos);
    }
}

fn collision_solver(
    heads_positions: Query<(Entity, &Position), With<SnakeHead>>,
    body_positions: Query<(Entity, &Position), (With<Snake>, Without<SnakeHead>)>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    mut eat_events: ResMut<Events<EatEvent>>,
    mut bump_events: ResMut<Events<BumpEvent>>,
) {
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
    for (e1, p1) in heads_positions.iter() {
        for (e2, p2) in body_positions.iter() {
            if p1 == p2 {
                bump_events.send(BumpEvent {
                    head: e1,
                    wall: e2,
                });
            }
        }
    }
}

fn get_tail(head: Entity, q: &mut Query<(Entity, &mut SnakeSegment)>) -> Entity {
    let mut tail = head;
    while let Ok((_, seg)) = q.get_mut(tail) {
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
    mut player: ResMut<Player>,
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
        if *eater == player.snake {
            player.food += 1;
        }
    }
}

fn bump_events_solver(
    mut gamestate: ResMut<State<GameState>>,
    bump_events: Res<Events<BumpEvent>>,
    mut bump_reader: Local<EventReader<BumpEvent>>,
) {
    while let Some(BumpEvent { head, wall }) = bump_reader.iter(&bump_events).next() {
        gamestate.set_next(GameState::Lost).unwrap();
        return;
    }
}

fn update_fps(diagnostics: Res<Diagnostics>, mut fps_text_q: Query<&mut Text, With<FpsText>>) {

    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            let mut text = fps_text_q.iter_mut().next().unwrap();
            text.value = format!("FPS: {:.2}", average);
        }
    }
}
fn update_hud(player: Res<Player>, mut food_text_q: Query<&mut Text, With<FoodText>>) {
    let mut food_text = food_text_q.iter_mut().next().unwrap();
    food_text.value = format!("Food: {}", player.food);
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Snake!".to_owned(),
            width: 600.,
            height: 600.,
            vsync: true,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup.system())
        .add_startup_stage(
            "game_setup",
            SystemStage::serial().with_system(game_setup.system()),
        )
        .add_event::<EatEvent>()
        .add_event::<BumpEvent>()
        .add_resource(State::new(GameState::Playing))
        .add_resource(LastInput{direction:Direction::Up})
        .add_system(input_events_sender.system())
        .add_system(update_fps.system())
        .add_stage_after(stage::UPDATE, "game_states", StateStage::<GameState>::default()
            .with_update_stage(GameState::Playing, SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(FIXED_TIMESTEP))
                .with_system(food_spawner.system())
                .with_system(segment_movement.system())
                .with_system(snake_movement.system())
                .with_system(collision_solver.system())
                .with_system(eat_events_solver.system())
                .with_system(bump_events_solver.system())
                .with_system(update_hud.system())
            )
        )
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .run();
}
