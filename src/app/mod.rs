//---------------------------------------------------------------------------//
// Copyright (c) 2025-2025 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Total War Patcher (TWPatcher) project,
// which can be found here: https://github.com/Frodo45127/twpatcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/twpatcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the input and command definitions for the tool.

use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use clap::{builder::PossibleValuesParser, Parser};

use std::path::PathBuf;

use rpfm_lib::games::supported_games::SupportedGames;

//---------------------------------------------------------------------------//
//                          Struct/Enum Definitions
//---------------------------------------------------------------------------//

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {

    /// Make output more detailed.
    #[arg(short, long)]
    pub verbose: bool,

    /// Makes TWPatcher skip the updates check done at the start.
    #[arg(short, long)]
    pub skip_updates_check: bool,

    /// Game we are using this tool for.
    #[arg(short, long, value_name = "GAME", value_parser = PossibleValuesParser::new(SupportedGames::default().game_keys_sorted().to_vec()))]
    pub game: String,

    /// Name of the file that contains the load order. Has to exist in the game folder.
    ///
    /// NOT SUPPORTED/IGNORED IN: Empire, Napoleon. In these TWPatcher will automatically use the user.script file instead.
    #[arg(short, long, value_name = "LOAD_ORDER_FILE_NAME")]
    pub load_order_file_name: String,

    /// Path where we want the Pack to be generated. If no Path is provided, the Pack will be generated in /data.
    ///
    /// NOT SUPPORTED/IGNORED IN: Empire, Napoleon, Shogun 2.
    #[arg(short = 'p', long, value_name = "GENERATED_PACK_PATH")]
    pub generated_pack_path: Option<String>,

    /// If supported, enable the script logging system of the game.
    ///
    /// Supported only in: Warhammer 2, Warhammer 3, Troy, Pharaoh, Pharaoh Dynasties.
    #[arg(short, long)]
    pub enable_logging: bool,

    /// Skip all the intro videos and start the game straight to main menu.
    #[arg(short = 'i', long)]
    pub skip_intro_videos: bool,

    /// Remove the trait limit for characters in Warhammer 3.
    #[arg(short, long)]
    pub remove_trait_limit: bool,

    /// Remove the "Siege Attacker" attribute from everything but artillery units.
    ///
    /// Supported only in: Warhammer 3.
    #[arg(short = 'a', long)]
    pub remove_siege_attacker: bool,

    /// Language for which TWPatcher will apply translations and patch locs for.
    ///
    /// Make sure to use this if you have the "no text from mods and my game is not in english" issue.
    ///
    /// The valid values for this field depend on what language you have your game set to. To know the valid values,
    /// open your game's data folder, and look fo what local_XX.pack files you have installed. The value you have to put here
    /// is the XX of the language you're using in the game.
    ///
    /// For example, for spanish, the file is called local_sp.pack, so here you'll have to use "sp".
    #[arg(short, long, value_name = "TRANSLATION_LANGUAGE")]
    pub translation_language: Option<String>,

    /// Multiplier to apply to unit sizes to make them bigger. In case of single entities, it multiplies their health instead.
    ///
    /// It also takes care of multiplying certain parameters that scale with difficulty, like tower and magic damage,
    /// to try to not alter the balance you had in the game.
    ///
    /// Supported only in: Warhammer 3, Three Kingdoms.
    #[arg(short = 'm', long, value_name = "MULTIPLIER")]
    pub unit_multiplier: Option<f64>,

    /// EXPERIMENTAL
    ///
    /// It tries to rebalance your load order around the overhaul you specify.
    ///
    /// Supported only in: Warhammer 3.
    #[arg(short, long, value_name = "BASE_MOD")]
    pub universal_rebalancer: Option<String>,

    /// EXPERIMENTAL
    ///
    /// It tries to execute the provided sql scripts (yes, admits multiple ones) over the load order.
    ///
    /// For each script, the param is a string with the script path, followed by all the consecutive params in order, everything separated with ;.
    #[arg(long, value_parser = sql_script_parser, value_name = "SCRIPT_PATH;PARAMS")]
    pub sql_script: Option<Vec<(PathBuf, Vec<String>)>>,

    /// It enables the dev-restricted parts of the UI. Note that the dev-restricted buttons may require things not shipped with the game, and will not work.
    #[arg(short = 'd', long)]
    pub enable_dev_ui: bool,
}

//---------------------------------------------------------------------------//
//                          Custom parsers
//---------------------------------------------------------------------------//

fn sql_script_parser(src: &str) -> Result<(PathBuf, Vec<String>)> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .quoting(true)
        .has_headers(false)
        .flexible(true)
        .from_reader(src.as_bytes());

    if let Some(Ok(record)) = reader.records().next() {
        if record.is_empty() {
            return Err(anyhow!("Incorrect CSV input."));
        } else {
            let path = PathBuf::from(&record[0]);
            if !path.is_file() {
                return Err(anyhow!("Path {} doesn't belong to a valid file.", &record[0]));
            }

            let params = if record.len() >= 2 {
                (1..record.len()).map(|x| record[x].to_owned()).collect::<Vec<_>>()
            } else {
                vec![]
            };

            return Ok((path, params));
        }
    }

    Err(anyhow!("Incorrect CSV input."))
}
