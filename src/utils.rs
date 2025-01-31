//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted Launcher (Runcher) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::{Path, PathBuf};

use rpfm_lib::binary::ReadBytes;
use rpfm_lib::files::{EncodeableExtraData, pack::Pack};
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType, supported_games::*};
use rpfm_lib::integrations::log::info;
use rpfm_lib::utils::{files_from_subdir, path_to_absolute_path};

// Default generated pack names. These are tested and work on their respective games.
pub const RESERVED_PACK_NAME: &str = "zzzzzzzzzzzzzzzzzzzzrun_you_fool_thron.pack";
pub const RESERVED_PACK_NAME_ALTERNATIVE: &str = "!!!!!!!!!!!!!!!!!!!!!run_you_fool_thron.pack";

const SCHEMAS_FOLDER: &str = "schemas";
const TRANSLATIONS_LOCAL_FOLDER: &str = "translations_local";
const TRANSLATIONS_REMOTE_FOLDER: &str = "translations_remote";

const USER_SCRIPT_FILE_NAME: &str = "user.script.txt";
const USER_SCRIPT_EMPIRE_FILE_NAME: &str = "user.empire_script.txt";

//-------------------------------------------------------------------------------//
//                             Util functions.
//-------------------------------------------------------------------------------//

pub fn translations_local_path() -> Result<PathBuf> {
    rpfm_config_path().map(|path| path.join(TRANSLATIONS_LOCAL_FOLDER))
}

pub fn translations_remote_path() -> Result<PathBuf> {
    config_path().map(|path| path.join(TRANSLATIONS_REMOTE_FOLDER))
}

pub fn schemas_path() -> Result<PathBuf> {
    config_path().map(|path| path.join(SCHEMAS_FOLDER))
}

/// This function returns the current config path, or an error if said path is not available.
///
/// Note: On `Debug´ mode this project is the project from where you execute one of RPFM's programs, which should be the root of the repo.
pub fn config_path() -> Result<PathBuf> {
    if cfg!(debug_assertions) { std::env::current_dir().map_err(From::from) } else {
        {
            match ProjectDirs::from("com", "FrodoWazEre", "twpatcher") {
                Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
                None => Err(anyhow!("Failed to get the config path."))
            }
        }
    }
}

pub fn rpfm_config_path() -> Result<PathBuf> {
    if cfg!(debug_assertions) { std::env::current_dir().map_err(From::from) } else {
        match ProjectDirs::from("com", "FrodoWazEre", "rpfm") {
            Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
            None => Err(anyhow!("Failed to get RPFM's config path."))
        }
    }
}

/// This function returns the paths of all the modded packs, in the order they're loaded.
pub fn load_order_from_file(load_order_path: &Path, game: &GameInfo, game_path: &Path, data_path: &Path) -> Result<Vec<PathBuf>> {

    // Note: Shogun 2 can be utf_16, but we assume people has the last version, where the file is utf_8.
    let (load_order_path, is_utf_16) = if *game.raw_db_version() >= 1 {
        (load_order_path.to_path_buf(), false)
    } else {
        let config_path = game.config_path(&game_path).ok_or(anyhow!("Error getting the game's config path."))?;
        let scripts_path = config_path.join("scripts");

        // Empire has its own user script.
        if game.key() == KEY_EMPIRE {
            (scripts_path.join(USER_SCRIPT_EMPIRE_FILE_NAME), true)
        } else {
            (scripts_path.join(USER_SCRIPT_FILE_NAME), true)
        }
    };

    // TODO: test this with utf16 userscript files.
    let mut file = BufReader::new(File::open(load_order_path)?);
    let string = if is_utf_16 {
        let mut data = vec![];
        file.read_to_end(&mut data)?;
        let mut cursor = Cursor::new(&data);
        cursor.read_string_u16(data.len())?
    } else {
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        string
    };

    // First, get all working paths.
    let mut working_paths = vec![data_path.to_path_buf()];
    working_paths.append(&mut string.lines()
        .filter(|x| x.starts_with("add_working_directory \""))
        .map(|x| path_to_absolute_path(&PathBuf::from(x[23..x.len() - 2].trim().to_owned()), true))
        .collect::<Vec<_>>());

    let mut mod_paths = string.lines()
        .filter(|x| x.starts_with("mod \""))
        .map(|x| x[5..x.len() - 2].trim().to_owned())
        .filter_map(|pack_name| working_paths.iter()
            .position(|path| path.join(&pack_name).is_file())
            .map(|x| working_paths[x].join(&pack_name))
        )
        .collect::<Vec<_>>();

    // We need to get the movie packs. Instead of checking every pack, we check the ones not already in the mod list, and not known as CA paths.
    let vanilla_paths = game.ca_packs_paths(game_path)?
        .iter()
        .map(|x| path_to_absolute_path(x, true))
        .collect::<Vec<_>>();

    // /data is already included here.
    for working_path in &working_paths {
        if let Ok(mut paths) = files_from_subdir(working_path, false) {
            paths.retain(|x| x.ends_with(".pack"));
            paths.iter_mut().for_each(|x| *x = path_to_absolute_path(x, true));

            for path in &paths {
                if !mod_paths.contains(path) && !vanilla_paths.contains(path) {
                    if let Ok(pack) = Pack::read_and_merge(&[path.to_path_buf()], true, false, false) {
                        if pack.pfh_file_type() == PFHFileType::Movie {
                            mod_paths.push(path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    Ok(mod_paths)
}

pub fn init_reserved_pack(game: &GameInfo) -> Result<Pack> {

    // Generate the reserved pack.
    //
    // Note: It has to be a movie pack because otherwise we cannot overwrite the intro files in older games.
    let pack_version = game.pfh_version_by_file_type(PFHFileType::Movie);
    let mut reserved_pack = Pack::new_with_version(pack_version);
    reserved_pack.set_pfh_file_type(PFHFileType::Movie);

    Ok(reserved_pack)
}

pub fn init_vanilla_pack(game: &GameInfo, game_path: &Path) -> Result<Pack> {
    Pack::read_and_merge_ca_packs(game, game_path).map_err(From::from)
}

pub fn init_modded_pack(paths: &[PathBuf]) -> Result<Pack> {
    if !paths.is_empty() {
        Pack::read_and_merge(&paths, true, false, true).map_err(From::from)
    } else {
        Ok(Pack::default())
    }
}

pub fn save_reserved_pack(game: &GameInfo, pack: &mut Pack, mod_paths: &[PathBuf], data_path: &Path, custom_path: Option<PathBuf>) -> Result<()> {

    // We need to use an alternative name for Shogun 2, Rome 2, Attila and Thrones because their load order logic for movie packs seems... either different or broken.
    let reserved_pack_name = if game.key() == KEY_SHOGUN_2 || game.key() == KEY_ROME_2 || game.key() == KEY_ATTILA || game.key() == KEY_THRONES_OF_BRITANNIA {
        RESERVED_PACK_NAME_ALTERNATIVE
    } else {
        RESERVED_PACK_NAME
    };

    let temp_path = match custom_path {
        Some(custom_path) => custom_path.to_path_buf(),
        None => data_path.join(reserved_pack_name),
    };

    info!("Saving Pack to: {}", temp_path.display());

    let mut encode_data = EncodeableExtraData::default();
    encode_data.set_nullify_dates(true);

    // Set the dependencies to be the entire load order. Fake for older games because it seems to crash for them.
    //
    // Real for newer games, as they crash if the dependencies are not set correctly.
    //
    // NOTE: Warhammer 1 may need to be here too.
    if game.key() != KEY_EMPIRE &&
        game.key() != KEY_NAPOLEON &&
        game.key() != KEY_SHOGUN_2 &&
        game.key() != KEY_ROME_2 &&
        game.key() != KEY_ATTILA &&
        game.key() != KEY_THRONES_OF_BRITANNIA {
        let pack_names = mod_paths.iter().map(|path| (true, path.file_name().unwrap().to_string_lossy().to_string())).collect::<Vec<_>>();
        pack.set_dependencies(pack_names);
    } else {
        let pack_names = mod_paths.iter().map(|path| (false, path.file_name().unwrap().to_string_lossy().to_string())).collect::<Vec<_>>();
        pack.set_dependencies(pack_names);
    }

    pack.save(Some(&temp_path), &game, &Some(encode_data)).map_err(From::from)
}
