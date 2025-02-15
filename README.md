# PumpkinVerse
A world management plugin for Pumpkin servers, inspired by the popular plugin Multiverse-Core.

## Building
Currently there are no pre-built binaries for this plugin, as the Pumpkin Plugin API is still in development. To build this plugin you will need to have the [rust toolchain](https://www.rust-lang.org/tools/install) installed. Once you have the rust toolchain installed, you can build the plugin by running the following command in the root directory of the project:
```bash
cargo build --release
```
The built plugin will be located in the `target/release` directory.
The file name will be one of the following depending on your operating system:
- `libpumpkinverse.so` (Linux)
- `pumpkinverse.dll` (Windows)
- `libpumpkinverse.dylib` (macOS)

## Usage
To use this plugin, you will need to have a Pumpkin server running. You can find how to install the Pumpkin server on the official [Pumpkin website](https://pumpkinmc.org/). Once you have the Pumpkin server running, you can place the built plugin in the `plugins` directory of the server. After you have placed the plugin in the `plugins` directory, you can start the server and the plugin will be loaded.

### Available Commands
The main command is `/pv` or `/pumpkinverse`. This command has the following subcommands:
- `/pv create <world_name>`: Creates a new world with the specified name.
- `/pv delete <world_name>`: Deletes the world with the specified name.
- `/pv list`: Lists all the worlds that have been created.
- `/pv tp <world_name> (player_name)`: Teleports the player to the specified world. If a player name is not provided, the command sender will be teleported.
