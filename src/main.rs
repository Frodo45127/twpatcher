//---------------------------------------------------------------------------//
// Copyright (c) 2025-2025 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Total War Patcher (TWPatcher) project,
// which can be found here: https://github.com/Frodo45127/twpatcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/twpatcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a small CLI tool to patch Total War load orders with custom patches.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::type_complexity,                // Disabled due to useless warnings.
    clippy::too_many_arguments              // Disabled because it gets annoying really quick.
)]

use clap::Parser;
use lazy_static::lazy_static;

#[cfg(target_os = "windows")]use std::fs::{read_dir, remove_dir_all};
use std::path::PathBuf;
use std::process::exit;

use common_utils::updater::*;

use rpfm_lib::games::supported_games::SupportedGames;
use rpfm_lib::integrations::{git::GitIntegration, log::*};
use rpfm_lib::schema::*;

use crate::app::Cli;
use crate::games::*;
use crate::utils::*;

mod app;
mod games;
mod utils;

lazy_static!{

    #[derive(Debug)]
    pub static ref PROGRAM_PATH: PathBuf = if cfg!(debug_assertions) {
        std::env::current_dir().unwrap()
    } else {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path
    };
}

const REPO_OWNER: &str = "Frodo45127";
const REPO_NAME: &str = "twpatcher";

/// Guess you know what this function does....
fn main() {

    let logger = Logger::init(&PathBuf::from("."), true, true, release_name!());
    if logger.is_err() {
        warn!("Logging initialization has failed. No logs will be saved.");
    }

    // Parse the entire cli command.
    let cli = Cli::parse();

    // Clean up folders from previous updates, if they exist. Windows-only.
    //
    // Done here because that way we cover executions without UI.
    #[cfg(target_os = "windows")] {
        if !cfg!(debug_assertions) {
            if let Ok(folders) = read_dir(&*PROGRAM_PATH) {
                for folder in folders.flatten() {
                    let folder_path = folder.path();
                    if folder_path.is_dir() && folder_path.file_name().unwrap().to_string_lossy().starts_with("update") {
                        let _ = remove_dir_all(&folder_path);
                    }
                }
                info!("Update folders cleared.");
            }
        }
    }

    // Perform an update check before doing anything else.
    if !cli.skip_updates_check {
        info!("Update Checks enabled. Checking if there are updates available.");

        let updater = Updater::new(UpdateChannel::Stable, REPO_OWNER, REPO_NAME);
        match updater.check(env!("CARGO_PKG_VERSION")) {
            Ok(response) => match response {
                APIResponse::NewBetaUpdate(update) |
                APIResponse::NewStableUpdate(update) |
                APIResponse::NewUpdateHotfix(update) => {
                    info!("- New update available: {}. Downlaoding and installing update...", update);
                    if let Err(error) = updater.download() {
                        error!("- Error when downloading/installing the update: {}", error);
                    } else {
                        info!("- Update downloaded and installed. Restart the program to use it.");
                    }
                }
                APIResponse::NoUpdate => info!("- No new updates available."),
                APIResponse::UnknownVersion => info!("- Unknown Version returned from Update Check."),
            }

            Err(error) => {
                error!("- Update Checks failed due to: {}.", error);
            }
        }
    }

    let game = match SupportedGames::default().game(&cli.game).cloned() {
        Some(game) => game,
        None => return error_path(&format!("Invalid game provided: {}", cli.game)),
    };

    let game_path = match game.find_game_install_location() {
        Ok(Some(game_path)) => game_path,
        _ => return error_path("Game Path not found"),
    };

    let data_path = match game.data_path(&game_path) {
        Ok(path) => path,
        _ => return error_path("Data Path not found"),
    };

    let mut reserved_pack = match init_reserved_pack(&game) {
        Ok(pack) => pack,
        Err(error) => return error_path(&error.to_string()),
    };


    let mut vanilla_pack = match init_vanilla_pack(&game, &game_path) {
        Ok(pack) => pack,
        Err(error) => return error_path(&error.to_string()),
    };

    info!("Vanilla data loaded. Loading load order data for: {}.", game.display_name());

    let load_order_path = game_path.join(&cli.load_order_file_name);
    if cli.verbose {
        info!("Load order file path: {}.", load_order_path.display());
    }

    let load_order = match load_order_from_file(&load_order_path, &game, &game_path, &data_path) {
        Ok(load_order) => load_order,
        Err(error) => return error_path(&error.to_string()),
    };

    info!("Load order found with the following mods:");
    for entry in &load_order {
        info!("- {}", entry.to_string_lossy().replace("\\", "/"));
    }

    let mut modded_pack = match init_modded_pack(&load_order) {
        Ok(pack) => pack,
        Err(error) => return error_path(&error.to_string()),
    };

    info!("Mod data loaded.");

    // Prepare the schemas. This includes downloading them in the background if we don't have them in RPFM's config folder or are outdated.
    let schema = match schemas_path() {
        Ok(local_path) => {

            info!("Checking and downloading schema updates...");

            // For now, ignore this failure. This can happen due to network issues, and as long as we have a valid schema, we can ignore it.
            let git_integration = GitIntegration::new(&local_path, SCHEMA_REPO, SCHEMA_BRANCH, SCHEMA_REMOTE);
            let _ = git_integration.update_repo();

            info!("Checking and downloading schema updates done.");

            match Schema::load(&local_path.join(game.schema_file_name()), None) {
                Ok(schema) => schema,
                Err(error) => return error_path(&error.to_string()),
            }
        },
        Err(error) => return error_path(&error.to_string()),
    };

    info!("Schema loaded. Processing selected options...");

    // Save it to disk once empty so its disk path is saved correctly.
    let custom_path = cli.generated_pack_path.clone().map(PathBuf::from);
    save_reserved_pack(&game, &mut reserved_pack, &load_order, &data_path, &custom_path).unwrap_or_else(|error| error_path(&error.to_string()));

    // With all the needed data initialized, check what flags we passed through the cli.
    prepare_launch_options(&cli, &game, &mut reserved_pack, &mut vanilla_pack, &mut modded_pack, &schema, &load_order, &game_path).unwrap_or_else(|error| error_path(&error.to_string()));
    info!("Options processed. Saving Pack");

    // If everything worked as expected, save the reserved pack.
    save_reserved_pack(&game, &mut reserved_pack, &load_order, &data_path, &custom_path).unwrap_or_else(|error| error_path(&error.to_string()));

    info!("All done. Closing. Bye!");

    exit(0)
}

fn error_path(error: &str) {
    error!("{}", error.to_string());

    info!("This terminal will close itself in 60 seconds to give you some time to read the log, but if you want, you can close it now.");
    std::thread::sleep(std::time::Duration::from_millis(60000));

    exit(1);
}
