//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a small CLI tool to patch Total War load orders with custom patches.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::type_complexity,                // Disabled due to useless warnings.
    clippy::too_many_arguments              // Disabled because it gets annoying really quick.
)]

use clap::Parser;

use std::path::PathBuf;
use std::process::exit;

use rpfm_lib::games::supported_games::SupportedGames;
use rpfm_lib::integrations::{git::GitIntegration, log::*};
use rpfm_lib::schema::*;

use crate::app::Cli;
use crate::games::*;
use crate::utils::*;

mod app;
mod games;
mod utils;

/// Guess you know what this function does....
fn main() {

    let logger = Logger::init(&PathBuf::from("."), true, true, release_name!());
    if logger.is_err() {
        warn!("Logging initialization has failed. No logs will be saved.");
    }

    // Parse the entire cli command.
    let cli = Cli::parse();

    let game = match SupportedGames::default().game(&cli.game).cloned() {
        Some(game) => game,
        None => {
            error!("Invalid game provided: {}", cli.game);
            exit(1);
        },
    };

    let game_path = match game.find_game_install_location() {
        Ok(Some(game_path)) => game_path,
        _ => {
            error!("Game Path not found");
            exit(1);
        },
    };

    let data_path = match game.data_path(&game_path) {
        Ok(path) => path,
        _ => {
            error!("Data Path not found");
            exit(1);
        },
    };

    let mut reserved_pack = match init_reserved_pack(&game) {
        Ok(pack) => pack,
        Err(error) => {
            error!("{}", error.to_string());
            exit(1);
        },
    };


    let mut vanilla_pack = match init_vanilla_pack(&game, &game_path) {
        Ok(pack) => pack,
        Err(error) => {
            error!("{}", error.to_string());
            exit(1);
        },
    };

    // TODO: Make this work with userscript and utf16.
    info!("Vanilla data loaded. Loading load order data for: {}.", game.display_name());

    let load_order_path = game_path.join(&cli.load_order_file_name);
    if cli.verbose {
        info!("Load order file path: {}.", load_order_path.display());
    }

    let load_order = match load_order_from_file(&load_order_path, &game, &game_path, &data_path) {
        Ok(load_order) => load_order,
        Err(error) => {
            error!("{}", error.to_string());
            exit(1);
        },
    };

    info!("Load order found with the following mods:");
    for entry in &load_order {
        info!("- {}", entry.to_string_lossy().replace("\\", "/"));
    }

    let mut modded_pack = match init_modded_pack(&load_order) {
        Ok(pack) => pack,
        Err(error) => {
            error!("{}", error.to_string());
            exit(1);
        },
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
                Err(error) => {
                    error!("{}", error.to_string());
                    exit(1);
                },
            }

        },
        Err(error) => {
            error!("{}", error.to_string());
            exit(1);
        },
    };

    info!("Schema loaded. Processing selected options...");

    // With all the needed data initialized, check what flags we passed through the cli.
    if let Err(error) = prepare_launch_options(&cli, &game, &mut reserved_pack, &mut vanilla_pack, &mut modded_pack, &schema, &load_order, &game_path) {
        error!("{}", error.to_string());
        exit(1);
    }

    info!("Options processed. Saving Pack");

    // If everything worked as expected, save the reserved pack.
    let custom_path = cli.generated_pack_path.clone().map(|x| PathBuf::from(x));
    if let Err(error) = save_reserved_pack(&game, &mut reserved_pack, &load_order, &data_path, custom_path) {
        error!("{}", error.to_string());
        exit(1);
    }

    info!("All done. Closing. Bye!");

    exit(0)
}
