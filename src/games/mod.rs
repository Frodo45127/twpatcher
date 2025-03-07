//---------------------------------------------------------------------------//
// Copyright (c) 2025-2025 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Total War Patcher (TWPatcher) project,
// which can be found here: https://github.com/Frodo45127/twpatcher.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/twpatcher/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::Result;
use itertools::Itertools;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rayon::prelude::*;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{PathBuf, Path};

use rpfm_extensions::dependencies::Dependencies;
use rpfm_extensions::optimizer::Optimizable;
use rpfm_extensions::translator::*;

use rpfm_lib::files::{Container, ContainerPath, DecodeableExtraData, EncodeableExtraData, FileType, loc::Loc, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::{*, supported_games::*};
use rpfm_lib::integrations::{git::GitIntegration, log::{error, info, warn}};
use rpfm_lib::schema::Schema;

use crate::app::Cli;
use crate::utils::*;

const EMPTY_CA_VP8: [u8; 595] = [
    0x43, 0x41, 0x4d, 0x56, 0x01, 0x00, 0x29, 0x00, 0x56, 0x50, 0x38, 0x30, 0x80, 0x02, 0xe0, 0x01, 0x55, 0x55,
    0x85, 0x42, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x4a, 0x02, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x21, 0x02, 0x00, 0x00, 0x00, 0x50, 0x42, 0x00, 0x9d, 0x01, 0x2a, 0x80, 0x02, 0xe0, 0x01, 0x00, 0x47, 0x08,
    0x85, 0x85, 0x88, 0x85, 0x84, 0x88, 0x02, 0x02, 0x00, 0x06, 0x16, 0x04, 0xf7, 0x06, 0x81, 0x64, 0x9f, 0x6b,
    0xdb, 0x9b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27, 0x38, 0x7b, 0x27,
    0x38, 0x7b, 0x27, 0x37, 0x80, 0xfe, 0xff, 0xab, 0x50, 0x80, 0x29, 0x00, 0x00, 0x00, 0x21, 0x02, 0x00, 0x00,
    0x01,
];

const EMPTY_BIK: [u8; 520] = [
    0x42, 0x49, 0x4B, 0x69, 0x00, 0x02, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xC8, 0x01, 0x00, 0x00, 0x01, 0x00,
    0x00, 0x00, 0x80, 0x02, 0x00, 0x00, 0xE0, 0x01, 0x00, 0x00, 0x19, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x90, 0x1B, 0x00, 0x00, 0x44, 0xAC, 0x00, 0x70, 0x00, 0x00,
    0x00, 0x00, 0x41, 0x00, 0x00, 0x00, 0x08, 0x02, 0x00, 0x00, 0xE4, 0x00, 0x00, 0x00, 0x90, 0x1B, 0x00, 0x00,
    0x20, 0xF9, 0x1A, 0x30, 0xA5, 0xDB, 0xEF, 0xAF, 0x82, 0x12, 0x02, 0xB2, 0xC1, 0x91, 0xB1, 0x11, 0x42, 0x12,
    0xD2, 0x51, 0x61, 0x21, 0xF1, 0xE0, 0xC0, 0xE0, 0xC0, 0xC0, 0xD0, 0x40, 0x61, 0xE2, 0x85, 0x00, 0x82, 0x40,
    0x43, 0x16, 0x73, 0xD2, 0x29, 0x1A, 0x52, 0x68, 0x82, 0xA5, 0x85, 0x44, 0x8C, 0xE9, 0x0C, 0x71, 0x90, 0x82,
    0x84, 0x11, 0x25, 0x91, 0x13, 0x42, 0x05, 0x81, 0x4B, 0x34, 0x5B, 0x2C, 0x63, 0x15, 0x08, 0x89, 0x11, 0x02,
    0x09, 0xC6, 0x50, 0x94, 0x64, 0xE9, 0xA3, 0x02, 0x80, 0xE8, 0x44, 0xA3, 0x88, 0x5F, 0x01, 0x28, 0x40, 0x4A,
    0x68, 0x54, 0x40, 0x2D, 0x80, 0xC0, 0x1C, 0x01, 0x09, 0x84, 0x00, 0x41, 0x24, 0x00, 0xFB, 0xC3, 0x87, 0x0F,
    0x1F, 0x10, 0x84, 0x23, 0x94, 0x2A, 0x24, 0x05, 0x4B, 0x21, 0x01, 0xE9, 0xD0, 0xD0, 0xB0, 0xC8, 0xF8, 0xE0,
    0xD8, 0xB0, 0x90, 0x88, 0x50, 0x40, 0x48, 0x38, 0x38, 0x40, 0x50, 0x68, 0xA0, 0x30, 0xF1, 0x3A, 0x04, 0x84,
    0x04, 0x85, 0x66, 0x79, 0x2F, 0xDC, 0x28, 0x2D, 0x15, 0x51, 0x96, 0xCB, 0x43, 0xAC, 0xD8, 0x0B, 0x54, 0x46,
    0x08, 0xC2, 0x25, 0xA2, 0xCA, 0x31, 0x04, 0xD6, 0x69, 0x13, 0x94, 0xCB, 0xCC, 0x12, 0x84, 0x34, 0xC1, 0x4D,
    0xAE, 0x3A, 0xEA, 0x50, 0x8B, 0x28, 0x0C, 0xC7, 0x35, 0x10, 0x8D, 0xD2, 0x4C, 0x1C, 0x88, 0x12, 0x30, 0x04,
    0x48, 0x59, 0x04, 0x09, 0x5B, 0x1C, 0x24, 0x00, 0x00, 0x10, 0x00, 0x60, 0x03, 0x30, 0x00, 0x80, 0x70, 0x00,
    0xF8, 0xF0, 0xE1, 0xC3, 0x07, 0x00, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00, 0x57, 0xC1, 0x7F, 0x65, 0xFC, 0x10,
    0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x80, 0x20, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB,
    0xB6, 0x01, 0x82, 0xB4, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB, 0x06, 0x08, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x40, 0x90, 0xB6, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB, 0x00, 0x41, 0xDA, 0xB6, 0x6D, 0xDB, 0xB6, 0x6D,
    0x03, 0x04, 0x11, 0x04, 0x69, 0xDB, 0xB6, 0x6D, 0xDB, 0xB6, 0x0D, 0x10, 0xA4, 0x6D, 0xDB, 0xB6, 0x6D, 0xDB,
    0x36, 0x00, 0x85, 0xB6, 0xFD, 0xFF, 0x00, 0x14, 0x04, 0x28, 0xDA, 0xB6, 0x6D, 0xFB, 0xFF, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x57, 0xC1, 0x7E, 0x65, 0xEC, 0x00, 0x00, 0x00, 0x00, 0x11, 0x08, 0x00, 0x00, 0x00,
    0x11, 0x00, 0x00, 0x80, 0x20, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
    0x4B, 0xFC, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x1F, 0x58, 0x22, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0xA0, 0xE0, 0xFF, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x57, 0xC1, 0x7E, 0x65, 0xEC, 0x00, 0x00, 0x00,
    0x00, 0x11, 0x08, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x80, 0x20, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x4B, 0xFC, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x1F, 0x58, 0x22, 0x02,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA0, 0xE0, 0xFF, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub const TRANSLATIONS_REPO: &str = "https://github.com/Frodo45127/total_war_translation_hub";
pub const TRANSLATIONS_REMOTE: &str = "origin";
pub const TRANSLATIONS_BRANCH: &str = "master";

pub const VANILLA_LOC_NAME: &str = "vanilla_english.tsv";
pub const VANILLA_FIXES_NAME: &str = "vanilla_fixes_";

const DB_FILE: &str = "sql.db3";

mod attila;
mod empire;
mod napoleon;
mod pharaoh;
mod rome_2;
mod shogun_2;
mod three_kingdoms;
mod thrones;
mod troy;
mod warhammer;
mod warhammer_2;
mod warhammer_3;

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub fn prepare_launch_options(cli: &Cli,
    game: &GameInfo,
    reserved_pack: &mut Pack,
    vanilla_pack: &mut Pack,
    modded_pack: &mut Pack,
    schema: &Schema,
    load_order: &[PathBuf],
    game_path: &Path
) -> Result<()> {

    // Skip videos.
    prepare_skip_intro_videos(cli, game, reserved_pack, vanilla_pack, modded_pack, schema)?;

    // Logging.
    prepare_script_logging(cli, game, reserved_pack)?;

    // Trait limit removal.
    prepare_trait_limit_removal(cli, game, reserved_pack, vanilla_pack, modded_pack, schema)?;

    // Siege Attacker removal.
    prepare_siege_attacker_removal(cli, game, reserved_pack, vanilla_pack, modded_pack, schema)?;

    // Translations.
    prepare_translations(cli, game, reserved_pack, load_order, game_path)?;

    // Unit multiplier.
    prepare_unit_multiplier(cli, game, reserved_pack, vanilla_pack, modded_pack, schema)?;

    // Universal rebalancer.
    prepare_universal_rebalancer(cli, game, reserved_pack, vanilla_pack, modded_pack, schema, load_order)?;

    // SQL Queries.
    prepare_sql_queries(cli, game, reserved_pack, vanilla_pack, modded_pack, schema)?;

    // Enable dev ui in all ui files.
    prepare_dev_ui(cli, game, reserved_pack, vanilla_pack, modded_pack)?;

    Ok(())
}


pub fn prepare_dev_ui(cli: &Cli, game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack) -> Result<()> {
    info!("- Enable Dev UI: {}.", cli.enable_dev_ui);

    if cli.enable_dev_ui {

        let mut files = vanilla_pack.files_by_type_and_paths(&[FileType::Text], &[ContainerPath::Folder("ui/".to_owned())], true)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        files.append(&mut modded_pack.files_by_type_and_paths(&[FileType::Text], &[ContainerPath::Folder("ui/".to_owned())], true)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>());

        files.append(&mut reserved_pack.files_by_type_and_paths(&[FileType::Text], &[ContainerPath::Folder("ui/".to_owned())], true)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>());

        // Sort them so file processing is done in the correct order.
        files.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());

        let enc_extra_data = Some(EncodeableExtraData::new_from_game_info(game));
        let dec_extra_data = Some(DecodeableExtraData::default());

        for file in &mut files {
            if let Ok(Some(RFileDecoded::Text(mut data))) = file.decode(&dec_extra_data, false, true) {
                if data.contents().contains("is_dev_only=\"true\"") {
                    let mut new_data = data.contents().replace("is_dev_only=\"true\"", "is_dev_only=\"false\"");

                    // Make the items visible. The ui files use both, is_visible and visible.
                    let mut pos = 0;
                    while let Some(start_pos) = new_data[pos..].find("is_dev_only") {
                        pos += start_pos;

                        let new_data_pre = new_data[..pos].to_owned();
                        let new_data_post = new_data[pos..].replacen("visible=\"false\"", "visible=\"true\"", 1);
                        new_data = new_data_pre + &new_data_post;

                        // Add one to skip to the next match.
                        pos += 1;
                    }
                    data.set_contents(new_data);

                    file.set_decoded(RFileDecoded::Text(data))?;
                    file.encode(&enc_extra_data, false, true, false)?;
                    reserved_pack.insert(file.clone())?;
                }
            }
        }
    }

    Ok(())
}

pub fn prepare_sql_queries(cli: &Cli, game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema) -> Result<()> {
    info!("- Apply SQL Scripts: {}.", cli.sql_script.is_some());

    if let Some(ref scripts) = cli.sql_script {
        info!("  - SQL Scripts to apply:");
        for (script, params) in scripts {
            info!("    - Path: {}. Params: {}", script.to_string_lossy().to_string().replace("\\", "/"), params.join(","));
        }

        let mut tables = vanilla_pack.files_by_type(&[FileType::DB])
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        // Give the daracores extreme low priority so they don't overwrite other mods tables.
        tables.iter_mut().for_each(rename_file_name_to_low_priority);

        tables.append(&mut modded_pack.files_by_type(&[FileType::DB])
            .into_iter()
            .cloned()
            .collect::<Vec<_>>());

        // Just in case another step of the launch process adds this table.
        tables.append(&mut reserved_pack.files_by_type(&[FileType::DB])
            .into_iter()
            .cloned()
            .collect::<Vec<_>>());

        // Sort them so file processing is done in the correct order.
        tables.sort_by_key(|rfile| rfile.path_in_container_raw().to_string());

        info!("  - Removing old db (if exists) and creating it anew.");

        // Make sure the database is clean before rebuilding it.
        let manager = SqliteConnectionManager::file(DB_FILE);
        let pool = Pool::new(manager)?;
        if let Err(error) = pool.get()?.execute_batch("
            PRAGMA writable_schema = 1;
            delete from sqlite_master where type in ('table', 'index', 'trigger');
            PRAGMA writable_schema = 0;
            VACUUM;
            PRAGMA INTEGRITY_CHECK;
        ") {
            error!("  - Error reseting the database file: {}.", error);
        }

        // Then, back all tables to a sqlite database.
        let enc_extra_data = Some(EncodeableExtraData::new_from_game_info(game));
        let mut dec_extra_data = DecodeableExtraData::default();
        dec_extra_data.set_schema(Some(schema));
        let dec_extra_data = Some(dec_extra_data);

        info!("  - Building SQL database.");

        for table in &mut tables {
            if let Ok(Some(RFileDecoded::DB(data))) = table.decode(&dec_extra_data, true, true) {
                if let Err(error) = data.table().db_to_sql(&pool) {
                    warn!("  - Table {}_v{} failed to be populated in the database, with the following error: {}.", data.table_name(), data.definition().version(), error);
                }
            }
        }

        info!("  - Executing scripts:");

        // Execute all the scripts in order.
        let mut edited_tables = vec![];
        for (script_path, params) in scripts {
            let script = script_path.to_string_lossy().to_string().replace("\\", "/");

            info!("    - Executing script: {}", script);

            let mut file = BufReader::new(File::open(script_path)?);
            let mut data = String::new();
            file.read_to_string(&mut data)?;

            // Get the array of tables to import first. It must always exist at the start of the file, after the random comment.
            let start_pos = data.find("-- Tables to import:");
            let end_pos = data.find("-- End of tables to import.");

            if let Some(start_pos) = start_pos {
                if let Some(end_pos) = end_pos {
                    if start_pos < end_pos {
                        let tables = data[start_pos + 20..end_pos].replace("-- ", "").replace("\r\n", "\n");
                        let tables_split = tables.split("\n")
                            .map(|x| x.trim())
                            .filter(|x| !x.is_empty())
                            .map(|x| format!("db/{}_tables/", x))
                            .collect::<Vec<_>>();
                        edited_tables.extend_from_slice(&tables_split);
                    } else {
                        error!("  - Failed to process sql script {}, due to bad formatting. Make sure the line '-- End of tables to import.' is AFTER '-- Tables to import:', with lines with the format '-- land_units' (the name of the table to import, without the _tables ending, and one table per line) in between.", script);
                    }
                } else {
                    error!("  - Failed to process sql script {}, due to bad formatting. Make sure the line '-- End of tables to import.' (without the commas) it's somewhere in the file.", script);
                }
            } else {
                error!("  - Failed to process sql script {}, due to bad formatting. Make sure the line '-- Tables to import:' (without the commas) it's somewhere in the file.", script);
            }

            // Next, perform the string replacements, if there's any to do.
            let start_pos = data.find("-- Strings to replace:");
            let end_pos = data.find("-- End of strings to replace.");

            if let Some(start_pos) = start_pos {
                if let Some(end_pos) = end_pos {
                    if start_pos < end_pos {
                        let mut subqueries = data[start_pos + 26..end_pos]
                            .replace("\r\n", "\n")
                            .split("------")
                            .map(|x| x.replace("--", ""))
                            .collect::<Vec<_>>();

                        // Reversed so we can do replacements within the queries.
                        subqueries.reverse();
                        for subquery in subqueries {

                            // Ignore empty subqueries.
                            if !subquery.is_empty() && subquery != "\n" {
                                let split = subquery.split(":::").collect::<Vec<_>>();
                                if split.len() == 2 {
                                    let key = split[0].trim();
                                    let subquery = split[1].split('\n').map(|x| x.trim()).join(" ");
                                    data = data.replace(key, &subquery);
                                } else {
                                    error!("  - Failed to process sql script {}, due to bad formatting. The subquery replacement {} is not formatted correctly.", script, subquery);
                                }
                            }
                        }
                    } else {
                        error!("  - Failed to process sql script {}, due to bad formatting. Make sure the line '-- End of subqueries to replace.' is AFTER '-- Subqueries to replace:'.", script);
                    }
                } else {
                    error!("  - Failed to process sql script {}, due to bad formatting. Make sure the line '-- End of subqueries to replace.' (without the commas) it's somewhere in the file.", script);
                }
            }

            // Replace the params in the in-memory script. We have to do it this way because execute batch doesn't allow params.
            for (index, param) in params.iter().enumerate() {
                data = data.replace(&format!("${}", index), param);
            }

            if let Err(error) = pool.get()?.execute_batch(&data) {
                error!("  - SQL script failed to execute with the following error: {}.", error);
                error!("  - Contents of the SQL script that failed (in case the error message doesn't output the full script):\n {}.", &data);
            }
        }

        info!("  - Rebuilding in-memory tables.");

        // Retrieve the data of the tables specified in the scripts and add them to the reserved pack.
        for table in &mut tables {
            for edited_path in &edited_tables {

                if table.path_in_container_raw().starts_with(edited_path) {
                    if let Ok(RFileDecoded::DB(ref mut data)) = table.decoded_mut() {
                        data.sql_to_db(&pool)?;
                    }

                    table.encode(&enc_extra_data, false, true, false)?;
                    reserved_pack.insert(table.clone())?;
                    break;
                }
            }
        }

        info!("  - SQL scripts processed.");
    }

    Ok(())
}

pub fn prepare_script_logging(cli: &Cli, game: &GameInfo, reserved_pack: &mut Pack) -> Result<()> {
    info!("- Enable script logging: {}.", cli.enable_logging);

    if cli.enable_logging {
        match game.key() {
            KEY_PHARAOH | KEY_PHARAOH_DYNASTIES => pharaoh::prepare_script_logging(reserved_pack),
            KEY_WARHAMMER_3 => warhammer_3::prepare_script_logging(reserved_pack),
            KEY_TROY => troy::prepare_script_logging(reserved_pack),
            KEY_THREE_KINGDOMS => Ok(()),
            KEY_WARHAMMER_2 => warhammer_2::prepare_script_logging(reserved_pack),
            KEY_WARHAMMER |
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA |
            KEY_ROME_2 |
            KEY_SHOGUN_2 |
            KEY_NAPOLEON |
            KEY_EMPIRE => Ok(()),
            _ => Ok(())
        }
    } else {
        Ok(())
    }
}

pub fn prepare_skip_intro_videos(cli: &Cli, game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema) -> Result<()> {
    info!("- Skip intro videos: {}.", cli.skip_intro_videos);

    if cli.skip_intro_videos {
        match game.key() {
            KEY_PHARAOH | KEY_PHARAOH_DYNASTIES => pharaoh::prepare_skip_intro_videos(game, reserved_pack, vanilla_pack, modded_pack, schema),
            KEY_WARHAMMER_3 => warhammer_3::prepare_skip_intro_videos(reserved_pack),
            KEY_TROY => troy::prepare_skip_intro_videos(game, reserved_pack, vanilla_pack, modded_pack, schema),
            KEY_THREE_KINGDOMS => three_kingdoms::prepare_skip_intro_videos(reserved_pack),
            KEY_WARHAMMER_2 => warhammer_2::prepare_skip_intro_videos(reserved_pack),
            KEY_WARHAMMER => warhammer::prepare_skip_intro_videos(reserved_pack),
            KEY_THRONES_OF_BRITANNIA => thrones::prepare_skip_intro_videos(reserved_pack),
            KEY_ATTILA => attila::prepare_skip_intro_videos(reserved_pack),
            KEY_ROME_2 => rome_2::prepare_skip_intro_videos(reserved_pack),
            KEY_SHOGUN_2 => shogun_2::prepare_skip_intro_videos(reserved_pack),
            KEY_NAPOLEON => napoleon::prepare_skip_intro_videos(reserved_pack),
            KEY_EMPIRE => empire::prepare_skip_intro_videos(reserved_pack),
            _ => Ok(())
        }
    } else {
        Ok(())
    }
}

pub fn prepare_trait_limit_removal(cli: &Cli, game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema) -> Result<()> {
    info!("- Remove trait limit: {}.", cli.remove_trait_limit);

    if cli.remove_trait_limit {
        match game.key() {
            KEY_PHARAOH | KEY_PHARAOH_DYNASTIES => Ok(()),
            KEY_WARHAMMER_3 => warhammer_3::prepare_trait_limit_removal(game, reserved_pack, vanilla_pack, modded_pack, schema),
            KEY_TROY |
            KEY_THREE_KINGDOMS |
            KEY_WARHAMMER_2 |
            KEY_WARHAMMER |
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA |
            KEY_ROME_2 |
            KEY_SHOGUN_2 |
            KEY_NAPOLEON |
            KEY_EMPIRE => Ok(()),
            _ => Ok(())
        }
    } else {
        Ok(())
    }
}

pub fn prepare_siege_attacker_removal(cli: &Cli, game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema) -> Result<()> {
    info!("- Remove Siege Attacker attribute: {}.", cli.remove_siege_attacker);

    if cli.remove_siege_attacker {
        match game.key() {
            KEY_PHARAOH | KEY_PHARAOH_DYNASTIES => Ok(()),
            KEY_WARHAMMER_3 => warhammer_3::prepare_siege_attacker_removal(game, reserved_pack, vanilla_pack, modded_pack, schema),
            KEY_TROY |
            KEY_THREE_KINGDOMS |
            KEY_WARHAMMER_2 |
            KEY_WARHAMMER |
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA |
            KEY_ROME_2 |
            KEY_SHOGUN_2 |
            KEY_NAPOLEON |
            KEY_EMPIRE => Ok(()),
            _ => Ok(())
        }
    } else {
        Ok(())
    }
}

/// All total war games use the same translation system.
///
/// The only particularity is that all games before warhammer 1 need to merge all translations into a localisation.loc file.
pub fn prepare_translations(cli: &Cli, game: &GameInfo, reserved_pack: &mut Pack, load_order: &[PathBuf], game_path: &Path) -> Result<()> {
    match &cli.translation_language {
        Some(language) => info!("- Apply translations fixes and mod translations for language: {}.", language),
        None => info!("- Do not apply translation fixes and mod translations."),
    }

    // Translation process:
    // - Pull new translations from the repo.
    // - Get language from UI.
    // - Get all the paths for available translations.
    // - Get all the packs we need to translate, in z-a order, so the last one has priority.
    // - Make an empty loc to put the translations into.
    // - Apply the fixes file, if found.
    // - For each Pack:
    //   - Check for translations in the local folder.
    //   - If not found, check for translations in the remote folder.
    //   - If found in any folder, apply them, or use the english value if there's no translation.
    //   - If none are found, just add the loc to the end of the translated loc.
    //
    // - Pass the translated loc through the optimizer to remove lines that didn't need to be there.
    //   - If it's an old game, append the vanilla localisation.loc file to the translated file.
    //   - If it's not an old game, check what lines got optimized and re-add them, but from the vanilla translation, so they overwrite any mod using them.

    // TODO: Troy has a weird translation system. Check that it works, and check pharaoh too.
    if let Some(language) = &cli.translation_language {

        // Download the translations. Ignore failure here, as it may fail due to network issues.
        if let Ok(local_path) = translations_remote_path() {
            info!("Checking and downloading community translations...");

            let git_integration = GitIntegration::new(&local_path, TRANSLATIONS_REPO, TRANSLATIONS_BRANCH, TRANSLATIONS_REMOTE);
            let _ = git_integration.update_repo();

            info!("Checking and downloading community translations done.");
        }

        // Get the paths. Local has priority over remote, so it goes first.
        let mut paths = vec![];
        if let Ok(path) = translations_local_path() {
            paths.push(path);
        }

        if let Ok(path) = translations_remote_path() {
            paths.push(path);
        }

        if !paths.is_empty() {
            let mut pack_paths = load_order.to_vec();

            // Reversed so we just get the higher priority stuff at the end, overwriting the rest.
            pack_paths.reverse();

            // If we need to merge the localisation.loc file if found to the translations.
            let use_old_multilanguage_logic = matches!(game.key(),
                KEY_THRONES_OF_BRITANNIA |
                KEY_ATTILA |
                KEY_ROME_2 |
                KEY_SHOGUN_2 |
                KEY_NAPOLEON |
                KEY_EMPIRE
            );

            let mut loc = Loc::new();
            let mut loc_data = vec![];

            for pack_path in &pack_paths {
                if let Some(ref pack_name) = pack_path.file_name().map(|name| name.to_string_lossy().to_string()) {
                    let mut translation_found = false;

                    if let Ok(tr) = PackTranslation::load(&paths, pack_name, game.key(), language) {
                        for tr in tr.translations().values() {

                            // Only add entries for values we actually have translated and up to date.
                            if !tr.value_translated().is_empty() && !*tr.needs_retranslation() {
                                loc_data.push(vec![
                                    DecodedData::StringU16(tr.key().to_owned()),
                                    DecodedData::StringU16(tr.value_translated().to_owned()),
                                    DecodedData::Boolean(false),
                                ]);
                            }

                            // If we're in a game with the old logic and there is no translation, add the text in english directly.
                            else if use_old_multilanguage_logic && !tr.value_original().is_empty() {
                                loc_data.push(vec![
                                    DecodedData::StringU16(tr.key().to_owned()),
                                    DecodedData::StringU16(tr.value_original().to_owned()),
                                    DecodedData::Boolean(false),
                                ]);
                            }
                        }

                        translation_found = true;
                    }

                    if cli.verbose && translation_found {
                        info!("  - Translation found for Pack: {}.", pack_name);
                    }

                    // If there's no translation data, just merge their locs.
                    if !translation_found {
                        let mut pack = Pack::read_and_merge(&[pack_path.to_path_buf()], true, false, true)?;

                        let mut locs = pack.files_by_type_mut(&[FileType::Loc]);

                        // Some people (SCM Team) decided it was a good idea to put a loc wiping entries outside the text folder.
                        // This filters out anything not in text/.
                        locs.retain(|loc| loc.path_in_container_raw().starts_with("text/"));
                        locs.sort_by(|a, b| a.path_in_container_raw().cmp(b.path_in_container_raw()));

                        let locs_split = locs.iter_mut()
                            .filter_map(|loc| if let Ok(Some(RFileDecoded::Loc(loc))) = loc.decode(&None, false, true) {
                                Some(loc)
                            } else {
                                None
                            })
                            .collect::<Vec<_>>();

                        let locs_split_ref = locs_split.iter().collect::<Vec<_>>();

                        let mut merged_loc = Loc::merge(&locs_split_ref)?;
                        loc_data.append(merged_loc.data_mut());
                    }
                }
            }

            // If we have a fixes file for the vanilla translation, apply it before everything else.
            if let Some(remote_path) = paths.last() {
                let fixes_loc_path = remote_path.join(format!("{}/{}{}.tsv", game.key(), VANILLA_FIXES_NAME, language));
                if let Ok(mut fixes_loc) = RFile::tsv_import_from_path(&fixes_loc_path, &None) {
                    fixes_loc.guess_file_type()?;
                    if let Ok(Some(RFileDecoded::Loc(fixes_loc))) = fixes_loc.decode(&None, false, true) {
                        loc_data.append(&mut fixes_loc.data().to_vec());
                    }
                }
            }

            // Only needed for modern games.
            let keys_pre_opt = if use_old_multilanguage_logic {
                HashSet::new()
            } else {
                loc_data.par_iter()
                    .map(|row| row[0].data_to_string().to_string())
                    .collect::<HashSet<_>>()
            };

            let mut vanilla_english_loc = None;

            // Perform the optimisation BEFORE appending the vanilla loc, if we're appending it. Otherwise we'll lose valid entries.
            if let Some(remote_path) = paths.last() {
                let vanilla_loc_path = remote_path.join(format!("{}/{}", game.key(), VANILLA_LOC_NAME));
                if let Ok(mut vanilla_loc) = RFile::tsv_import_from_path(&vanilla_loc_path, &None) {
                    vanilla_loc.guess_file_type()?;
                    vanilla_loc.decode(&None, true, false)?;

                    // Keep it in memory to reuse it when filling missing translation data.
                    vanilla_english_loc = Some(vanilla_loc.clone());

                    if !loc_data.is_empty() {
                        loc.set_data(&loc_data)?;

                        // Workaround: We do not need a whole dependencies for this, just one file with the entire english loc combined.
                        // So we initialize an empty dependencies, the manually insert that loc.
                        let mut dependencies = Dependencies::default();
                        dependencies.insert_loc_as_vanilla_loc(vanilla_loc);

                        let _ = !loc.optimize(&mut dependencies);
                        loc_data = loc.data().to_vec();
                    }
                }
            }

            // If the game uses the old multilanguage logic, we need to get the most updated version of localisation.loc from the game and append it to our loc.
            if use_old_multilanguage_logic {
                let mut pack = Pack::read_and_merge_ca_packs(game, game_path)?;
                if let Some(vanilla_loc) = pack.file_mut(TRANSLATED_PATH_OLD, false) {
                    if let Ok(Some(RFileDecoded::Loc(mut loc))) = vanilla_loc.decode(&None, false, true) {
                        loc_data.append(loc.data_mut());
                    }
                }
            }


            // If the game is not using the old logic, we need to restore the optimized lines, but from the translated loc, not the english one.
            else {
                let mut pack = Pack::read_and_merge_ca_packs(game, game_path)?;
                let mut vanilla_locs = pack.files_by_type_mut(&[FileType::Loc]);
                let vanilla_loc_data = vanilla_locs.par_iter_mut()
                    .filter_map(|rfile| {
                        if let Ok(Some(RFileDecoded::Loc(loc))) = rfile.decode(&None, false, true) {
                            Some(loc)
                        } else {
                            None
                        }
                    })
                    .map(|loc| loc.data().to_vec())
                    .flatten()
                    .collect::<Vec<_>>();


                let vanilla_loc_data_hash = vanilla_loc_data
                    .par_iter()
                    .rev()
                    .map(|row| (row[0].data_to_string(), row[1].data_to_string()))
                    .collect::<HashMap<_,_>>();

                let keys_post_opt = loc_data.par_iter()
                    .map(|row| row[0].data_to_string().to_string())
                    .collect::<HashSet<_>>();

                let keys_to_fill_from_vanilla = keys_pre_opt.par_iter()
                    .filter(|key| !keys_post_opt.contains(&**key))
                    .map(|key| key)
                    .collect::<HashSet<_>>();

                let mut new_rows = keys_to_fill_from_vanilla.par_iter()
                    .filter_map(|key| {
                        let value = vanilla_loc_data_hash.get(&***key)?;

                        Some(vec![
                            DecodedData::StringU16(key.to_string()),
                            DecodedData::StringU16(value.to_string()),
                            DecodedData::Boolean(false),
                        ])
                    })
                    .collect::<Vec<_>>();
                loc_data.append(&mut new_rows);

                // There's a bug that sometimes surfaces in patches in which the english loc has lines the other locs don't have.
                // We need to grab them from the english loc and added to our own post-optimizations.
                //
                // This is mainly for newer games that still get patched.
                if let Some(mut vanilla_english_loc) = vanilla_english_loc {
                    if let Ok(Some(RFileDecoded::Loc(vanilla_english_loc))) = vanilla_english_loc.decode(&None, false, true) {
                        let mut missing_entries = vanilla_english_loc.data()
                            .par_iter()
                            .rev()
                            .filter_map(|entry| {

                                // Ignore entries already empty in english.
                                if !entry[1].data_to_string().is_empty() {
                                    match vanilla_loc_data_hash.get(&entry[0].data_to_string()) {
                                        Some(vanilla_entry) => {
                                            if vanilla_entry.is_empty() {
                                                Some(entry.clone())
                                            } else {
                                                None
                                            }
                                        }

                                        // Not found means is only in english.
                                        None => Some(entry.clone())
                                    }
                                } else {
                                    None
                                }
                            }).collect::<Vec<_>>();

                        // These need to be on top of the file in order to overwrite empty lines.
                        missing_entries.append(&mut loc_data);
                        loc_data = missing_entries;
                    }
                }
            }

            if !loc_data.is_empty() {
                loc.set_data(&loc_data)?;

                let path = if use_old_multilanguage_logic {
                    TRANSLATED_PATH_OLD.to_string()
                } else {
                    TRANSLATED_PATH.to_string()
                };

                let file = RFile::new_from_decoded(&RFileDecoded::Loc(loc), 0, &path);
                reserved_pack.files_mut().insert(path, file);
            }
        }
    }

    Ok(())
}

pub fn prepare_unit_multiplier(cli: &Cli, game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema) -> Result<()> {
    if let Some(multiplier) = cli.unit_multiplier {

        info!("- Apply unit multiplier (if the game supports it) of: {}.", multiplier);

        match game.key() {
            KEY_PHARAOH_DYNASTIES |
            KEY_PHARAOH => Ok(()),
            KEY_WARHAMMER_3 => warhammer_3::prepare_unit_multiplier(game, reserved_pack, vanilla_pack, modded_pack, schema, multiplier),
            KEY_TROY => Ok(()),
            KEY_THREE_KINGDOMS => three_kingdoms::prepare_unit_multiplier(game, reserved_pack, vanilla_pack, modded_pack, schema, multiplier),
            KEY_WARHAMMER_2 |
            KEY_WARHAMMER |
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA |
            KEY_ROME_2 |
            KEY_SHOGUN_2 |
            KEY_NAPOLEON |
            KEY_EMPIRE => Ok(()),
            _ => Ok(())
        }
    } else {

        info!("- Do not apply unit multiplier.");
        Ok(())
    }
}

pub fn prepare_universal_rebalancer(cli: &Cli, game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema, mod_paths: &[PathBuf]) -> Result<()> {
    if let Some(mod_name) = &cli.universal_rebalancer {
        info!("- Perform a universal rebalancing using the mod {} as base mod.", mod_name);

        if let Some(mod_path) = mod_paths.iter().find(|x| x.ends_with(mod_name)) {
            match game.key() {
                KEY_PHARAOH | KEY_PHARAOH_DYNASTIES => Ok(()),
                KEY_WARHAMMER_3 => warhammer_3::prepare_universal_rebalancer(game, reserved_pack, vanilla_pack, modded_pack, schema, mod_path, mod_paths),
                KEY_TROY |
                KEY_THREE_KINGDOMS |
                KEY_WARHAMMER_2 |
                KEY_WARHAMMER |
                KEY_THRONES_OF_BRITANNIA |
                KEY_ATTILA |
                KEY_ROME_2 |
                KEY_SHOGUN_2 |
                KEY_NAPOLEON |
                KEY_EMPIRE => Ok(()),
                _ => Ok(())
            }
        } else {
            Ok(())
        }
    } else {

        info!("- Do not perform a universal rebalancing pass.");
        Ok(())
    }
}

pub fn rename_file_name_to_low_priority(file: &mut RFile) {
    let mut path = file.path_in_container_raw().split('/').map(|x| x.to_owned()).collect::<Vec<_>>();

    if let Some(name) = path.last_mut() {
        *name = format!("~{}", name);
    }

    file.set_path_in_container_raw(&path.join("/"));
}
