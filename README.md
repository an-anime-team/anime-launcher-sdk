# 🦀 Anime Launcher SDK

SDK based on [anime-game-core](https://github.com/an-anime-team/anime-game-core) with some basic instruments like launcher state system and configuration file manager, written in Rust

> ⚠️ Current implementation is considered legacy. No deep changes will be made.

## Common features

| Description                                      | Feature       |
|--------------------------------------------------|---------------|
| Manage launcher state                            | `states`      |
| Manage launcher config                           | `config`      |
| Manage components (list wine/dxvk versions, etc) | `components`  |
| Run the game                                     | `game`        |
| Use Discord RPC when the game is running         | `discord-rpc` |
| Run the game in `bwrap` sandbox                  | `sandbox`     |

## Anime Game specific features

| Description                                                        | Feature                 |
|--------------------------------------------------------------------|-------------------------|
| Emulate game environment to get additional in-game payment methods | `environment-emulation` |
| Unlock in-game frame rendering limit                               | `fps-unlocker`          |

## Supported games

| Name                                                                             | Feature                   |
|----------------------------------------------------------------------------------|---------------------------|
| [An Anime Game](https://github.com/an-anime-team/an-anime-game-launcher)         | `gen-shin` (without dash) |
| [Honkers Railway](https://github.com/an-anime-team/the-honkers-railway-launcher) | `star-rail`               |
| [Honkers](https://github.com/an-anime-team/honkers-launcher)                     | `hon-kai` (without dash)  |
| [An Anime Borb](https://github.com/an-anime-team/an-anime-borb-launcher)         | `pgr`                     |
| [Waves](https://github.com/an-anime-team/wavey-launcher)                         | `wuwa`                    |
