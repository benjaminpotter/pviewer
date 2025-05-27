//! Displays a single [`Sprite`], created from an image.

use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_task)
        .add_systems(Update, update_task)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let image_handle = asset_server.load("mono8_sky_image.png");

    commands.spawn((
        Sprite::from_image(image_handle.clone()),
        Transform::from_scale(Vec3::splat(0.1)),
        ComputeStokes::new(),
    ));
}

#[derive(Component)]
struct ComputeStokes {
    task: Option<Task<CommandQueue>> 
}

impl ComputeStokes {
    fn new() -> Self {
        ComputeStokes { task: None }
    }
}

fn spawn_task(mut commands: Commands, entity_query: Query<Entity, Added<ComputeStokes>> {
    let thread_pool = AsyncComputeTaskPool::get();

    for entity in entity_query {
        let task = thread_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            command_queue.push(move |world: &mut World| {
                world.entity_mut(entity).remove::<ComputeStokes>();
            });

            command_queue
        });
    }
}

fn update_task(mut commands: Commands, mut stokes_tasks: Query<&mut ComputeStokes>) {
    for mut task in &mut stokes_tasks {
        if let Some(task) = task {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
        }
    }
}
