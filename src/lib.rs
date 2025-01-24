use pumpkin::plugin::Context;
use pumpkin::world::World;
use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin_registry::DimensionType;
use pumpkin_world::dimension::Dimension;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

#[plugin_method]
async fn on_load(&mut self, context: &Context) -> Result<(), String> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let data_dir = context.get_data_folder();
    let config_file = Path::new(&data_dir).join("config.toml");

    if !config_file.exists() {
        let mut file = std::fs::File::create(&config_file).map_err(|e| e.to_string())?;
        let config = toml::to_string(&self).map_err(|e| e.to_string())?;
        file.write(config.as_bytes()).map_err(|e| e.to_string())?;
    } else {
        let mut file = std::fs::File::open(&config_file).map_err(|e| e.to_string())?;
        let mut config = String::new();
        file.read_to_string(&mut config)
            .map_err(|e| e.to_string())?;
        *self = toml::from_str(&config).map_err(|e| e.to_string())?;
    }

    log::info!("PumpkinVerse config loaded!");
    log::debug!("World folder: {}", self.world_folder);
    log::info!("Loading worlds...");

    let world_folder = Path::new(&self.world_folder);
    if !world_folder.exists() {
        std::fs::create_dir(&world_folder).map_err(|e| e.to_string())?;
    }

    {
        let mut worlds = context.server.worlds.write().await;
        for name in self.managed_worlds.iter() {
            let world_folder = world_folder.join(name);
            let world = World::load(
                Dimension::OverWorld.into_level(world_folder),
                DimensionType::OverWorld,
            );

            worlds.push(world);
        }
    }

    log::info!("Worlds loaded!");

    Ok(())
}

#[plugin_impl]
#[derive(Serialize, Deserialize)]
pub struct MyPlugin {
    world_folder: String,
    managed_worlds: Vec<String>,
}

impl MyPlugin {
    pub fn new() -> Self {
        MyPlugin {
            world_folder: "worlds".to_string(),
            managed_worlds: Vec::new(),
        }
    }
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self::new()
    }
}
