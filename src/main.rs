use bevy::{prelude::*, render::view::window, window::PrimaryWindow};
use rand::prelude::*;

// player size
pub const PLAYER_SIZE: f32 = 64.0;
pub const PLEYER_SPEED: f32 = 500.0;
pub const NUMBER_OF_ENEMIES: usize = 4;
pub const ENEMY_SPEED: f32 = 200.0;
pub const ENEMY_SIZE: f32 = 64.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(CameraPlugin)
        .add_startup_system(spawn_enemies)
        .add_system(update_enemy_direction)
        .add_system(confine_enemy_movement)
        .add_system(enemy_hit_player)
        .run();
}

// pleyer Plugin
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(pleyer_movement)
            .add_system(confine_player_movement)
            .add_system(enemy_movement);
    }
}

// bundles 是一组预定义的组件
// pleyer Component
#[derive(Component)]
pub struct Player {}
// spawn player  commands 生成实体，windows_query 获取窗口宽度、高度，assert_server 加载图片
pub fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    assert_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
            texture: assert_server.load("sprites/ball_blue_large.png"),
            ..default()
        },
        Player {},
    ));
}

// Enemy Component
#[derive(Component)]
pub struct Enemy {
    direction: Vec2,
}

//spawn enemies
pub fn spawn_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    assert_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    for i in 0..NUMBER_OF_ENEMIES {
        let random_x = random::<f32>() * window.width();
        let random_y = random::<f32>() * window.height();
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(random_x, random_y, 0.0),
                texture: assert_server.load("sprites/ball_red_large.png"),
                ..default()
            },
            Enemy {
                direction: Vec2::new(random::<f32>(), random::<f32>()).normalize(),
            },
        ));
    }
}

// camera Plugin
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera);
    }
}
// camera Component
#[derive(Component)]
pub struct Camera;
pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

// 1. 根据键盘输入移动人物
// 2. 保持人物始终在屏幕中
// 3.

// 输入
// 玩家

pub fn pleyer_movement(
    key_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    // 使用if let Ok(val) = ? 来处理不存在的错误
    if let Ok(mut transform) = player_query.get_single_mut() {
        // 获取方向
        let mut direction = Vec3::ZERO;
        if key_input.pressed(KeyCode::Left) || key_input.pressed(KeyCode::A) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }
        if key_input.pressed(KeyCode::Right) || key_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }
        if key_input.pressed(KeyCode::Up) || key_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }
        if key_input.pressed(KeyCode::Down) || key_input.pressed(KeyCode::S) {
            direction += Vec3::new(0.0, -1.0, 0.0);
        }

        // 判断是否有按键按下
        if direction.length() > 0.0 {
            direction = direction.normalize();
        }
        // delta_seconds 使移动速度在任何帧率都一致
        transform.translation += direction * PLEYER_SPEED * time.delta_seconds();
    }
}

// 限制玩家在地图中
pub fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(mut player) = player_query.get_single_mut() {
        let window = window_query.get_single().unwrap();
        // 获取玩家大小
        let half_player_size = PLAYER_SIZE / 2.0;
        let x_min = 0.0 + half_player_size;
        let x_max = window.width() - half_player_size;
        let y_min = 0.0 + half_player_size;
        let y_max = window.height() - half_player_size;

        // 限制玩家位置
        let mut transform_player = player.translation;
        if transform_player.x < x_min {
            transform_player.x = x_min;
        } else if transform_player.x > x_max {
            transform_player.x = x_max;
        } else if transform_player.y < y_min {
            transform_player.y = y_min;
        } else if transform_player.y > y_max {
            transform_player.y = y_max;
        }
        player.translation = transform_player;
    }
}

pub fn enemy_movement(mut enemy_query: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
    for (mut transform, enemy) in enemy_query.iter_mut() {
        let direction = Vec3::new(enemy.direction.x, enemy.direction.y, 0.0);
        transform.translation += direction * ENEMY_SPEED * time.delta_seconds();
    }
}

pub fn update_enemy_direction(
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    audio: Res<Audio>,
    assert_server: Res<AssetServer>
) {
    let window = windows.get_single().unwrap();
    let half_enemy_size = ENEMY_SIZE / 2.0;
    let x_min = 0.0 + half_enemy_size;
    let x_max = window.width() - half_enemy_size;
    let y_min = 0.0 + half_enemy_size;
    let y_max = window.height() - half_enemy_size;

    for (transform, mut enemy) in enemy_query.iter_mut() {
        let mut is_changed=false;

        let transform_temp = transform.translation;
        if transform_temp.x < x_min || transform_temp.x > x_max {
            enemy.direction.x *= -1.0;
            is_changed=true;
        } else if transform_temp.y < y_min || transform_temp.y > y_max {
            enemy.direction.y *= -1.0;
            is_changed=true;
        }
        if is_changed {
            let sound_effect=assert_server.load("audio/pluck_001.ogg");
            audio.play(sound_effect);
        }
    }
}

// 限制敌人在窗口中
pub fn confine_enemy_movement(
    mut enemy_query: Query<&mut Transform, With<Enemy>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.get_single().unwrap();
    let half_enemy_size = ENEMY_SIZE / 2.0;
    let x_min = 0.0 + half_enemy_size;
    let x_max = window.width() - half_enemy_size;
    let y_min = 0.0 + half_enemy_size;
    let y_max = window.height() - half_enemy_size;

    for mut transform in enemy_query.iter_mut() {
        if transform.translation.x < x_min {
            transform.translation.x = x_min;
        } else if transform.translation.x > x_max {
            transform.translation.x = x_max;
        } else if transform.translation.y < y_min {
            transform.translation.y = y_min;
        } else if transform.translation.y > y_max {
            transform.translation.y = y_max;
        }
    }
}

pub fn enemy_hit_player(
    mut commands: Commands,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut player_query: Query<(Entity,&Transform), With<Player>>,
    assert_server: Res<AssetServer>,
    audio: Res<Audio>
) {
    if let Ok((player_entity,player)) = player_query.get_single_mut()  {
        for enemy in enemy_query.iter() {
            // 判断当前敌人与玩家距离
            let distance=player.translation.distance(enemy.translation);
            if distance < (PLAYER_SIZE/2.0) + (ENEMY_SIZE/2.0) {
                //相撞
                println!("game over!");
                let sound_effect=assert_server.load("audio/explosionCrunch_000.ogg");
                audio.play(sound_effect);
                commands.entity(player_entity).despawn();
            }
        }
    }
}