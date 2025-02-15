use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use pumpkin::command::args::players::PlayersArgumentConsumer;
use pumpkin::command::args::simple::SimpleArgConsumer;
use pumpkin::command::args::Arg;
use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::dispatcher::CommandError;
use pumpkin::command::dispatcher::CommandError::InvalidConsumption;
use pumpkin::command::tree::builder::argument;
use pumpkin::command::tree::builder::literal;
use pumpkin::command::tree::CommandTree;
use pumpkin::command::CommandExecutor;
use pumpkin::command::CommandSender;
use pumpkin::server::Server;
use pumpkin::world::World;
use pumpkin_api_macros::with_runtime;
use pumpkin_registry::DimensionType;
use pumpkin_util::text::TextComponent;
use pumpkin_world::dimension::Dimension;

use crate::save_config;

const NAMES: [&str; 2] = ["pumpkinverse", "pv"];

const DESCRIPTION: &str = "Manage multiple worlds thorugh PumpkinVerse.";

const PLAYER_ARG_NAME: &str = "player";
const WORLD_ARG_NAME: &str = "world";

struct CreateWorldExecutor;
struct ListWorldExecutor;
struct DeleteWorldExecutor;
struct TeleportExecutor;

#[async_trait]
impl CommandExecutor for CreateWorldExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(world_name)) = args.get(WORLD_ARG_NAME) else {
            return Err(InvalidConsumption(Some(WORLD_ARG_NAME.into())));
        };

        {
            let config = crate::CONFIG.lock().await;
            if config.managed_worlds.contains(&world_name.to_string()) {
                sender
                    .send_message(
                        TextComponent::text(format!("World \"{world_name}\" already exists."))
                            .color_named(pumpkin_util::text::color::NamedColor::Red),
                    )
                    .await;
                return Ok(());
            }
        }

        sender
            .send_message(
                TextComponent::text(format!("Creating new world \"{world_name}\"..."))
                    .color_named(pumpkin_util::text::color::NamedColor::Aqua),
            )
            .await;

        {
            let mut config = crate::CONFIG.lock().await;
            let world_folder = Path::new(&config.world_folder.clone()).join(world_name);
            let mut level = Dimension::OverWorld.into_level(world_folder);
            level.level_info.level_name = world_name.to_string();
            let world = World::load(level, DimensionType::Overworld);

            {
                let mut worlds = server.worlds.write().await;
                worlds.push(Arc::new(world));
            }

            config.managed_worlds.push(world_name.to_string());
        }

        sender
            .send_message(
                TextComponent::text(format!("World \"{world_name}\" created."))
                    .color_named(pumpkin_util::text::color::NamedColor::Green),
            )
            .await;

        let _ = save_config().await;

        Ok(())
    }
}

#[async_trait]
impl CommandExecutor for ListWorldExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let config = crate::CONFIG.lock().await;
        let worlds = server.worlds.read().await;

        let mut message = TextComponent::text("Worlds: ")
            .color_named(pumpkin_util::text::color::NamedColor::Aqua);

        for world in worlds.iter() {
            let world_name = world.level.level_info.level_name.clone();

            let is_managed = config.managed_worlds.contains(&world_name);
            let color = if is_managed {
                pumpkin_util::text::color::NamedColor::Green
            } else {
                pumpkin_util::text::color::NamedColor::Red
            };

            message = message.add_child(
                TextComponent::text(format!("\n - {world_name}"))
                    .color_named(color)
                    .add_child(
                        TextComponent::text(if is_managed { " (Managed)" } else { "" })
                            .color_named(pumpkin_util::text::color::NamedColor::Gray),
                    ),
            );
        }

        sender.send_message(message).await;
        Ok(())
    }
}

#[async_trait]
impl CommandExecutor for DeleteWorldExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(world_name)) = args.get(WORLD_ARG_NAME) else {
            return Err(InvalidConsumption(Some(WORLD_ARG_NAME.into())));
        };

        {
            let mut config = crate::CONFIG.lock().await;
            if !config.managed_worlds.contains(&world_name.to_string()) {
                sender
                    .send_message(
                        TextComponent::text(format!("World \"{world_name}\" does not exist."))
                            .color_named(pumpkin_util::text::color::NamedColor::Red),
                    )
                    .await;
                return Ok(());
            }

            config
                .managed_worlds
                .retain(|name| name != &world_name.to_string());
        }

        {
            let mut worlds = server.worlds.write().await;
            worlds.retain(|world| world.level.level_info.level_name != world_name.to_string());
        }

        sender
            .send_message(
                TextComponent::text(format!("World \"{world_name}\" deleted."))
                    .color_named(pumpkin_util::text::color::NamedColor::Green),
            )
            .await;

        let _ = save_config().await;
        Ok(())
    }
}

#[with_runtime(global)]
#[async_trait]
impl CommandExecutor for TeleportExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(world_name)) = args.get(WORLD_ARG_NAME) else {
            return Err(InvalidConsumption(Some(WORLD_ARG_NAME.into())));
        };

        let worlds = server.worlds.read().await;
        let world = worlds
            .iter()
            .find(|world| world.level.level_info.level_name == world_name.to_string());

        match world {
            Some(world) => {
                if let Some(Arg::Players(players)) = args.get(PLAYER_ARG_NAME) {
                    for player in players {
                        player
                            .clone()
                            .teleport_world(world.clone(), None, None, None)
                            .await;
                    }
                } else {
                    if let CommandSender::Player(player) = sender {
                        player
                            .clone()
                            .teleport_world(world.clone(), None, None, None)
                            .await;
                    } else {
                        sender
                            .send_message(
                                TextComponent::text("You must specify a player to teleport.")
                                    .color_named(pumpkin_util::text::color::NamedColor::Red),
                            )
                            .await;
                    }
                }
            }
            None => {
                sender
                    .send_message(
                        TextComponent::text(format!("World \"{world_name}\" does not exist."))
                            .color_named(pumpkin_util::text::color::NamedColor::Red),
                    )
                    .await;
            }
        }

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("create")
                .then(argument(WORLD_ARG_NAME, SimpleArgConsumer).execute(CreateWorldExecutor)),
        )
        .then(literal("list").execute(ListWorldExecutor))
        .then(
            literal("delete")
                .then(argument(WORLD_ARG_NAME, SimpleArgConsumer).execute(DeleteWorldExecutor)),
        )
        .then(
            literal("tp").then(
                argument(WORLD_ARG_NAME, SimpleArgConsumer)
                    .execute(TeleportExecutor)
                    .then(
                        argument(PLAYER_ARG_NAME, PlayersArgumentConsumer)
                            .execute(TeleportExecutor),
                    ),
            ),
        )
}
