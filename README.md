# Anime Launcher SDK

## Project goals

* Unify backends for [gtk](https://github.com/an-anime-team/an-anime-game-launcher-gtk) and [tauri](https://github.com/an-anime-team/an-anime-game-launcher-tauri) launchers so they will have same functionality;
* Remove excess code from gtk launcher and prepare it for relm4 rewrite;
* Prepare codebase for tauri rewrite;

## Current progress (75%)

| Status | Feature | Description |
| :-: | - | - |
| ✅ | states | Getting current launcher's state (update available, etc.) |
| ✅ | config | Work with config file |
| ✅ | components | Work with components needed to run the game |
| ✅ | | List Wine and DXVK versions |
| ❌ | | Download, delete and select wine |
| ❌ | | Download, delete, select and apply DXVK |
| ✅ | game | Run the game |
| ✅ | fps-unlocker | Support of FPS unlocker. Manage its config, download, use in game runner |
