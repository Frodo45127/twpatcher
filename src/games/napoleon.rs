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

use rpfm_lib::files::{FileType, pack::Pack, RFile};

use super::EMPTY_BIK;

const INTRO_MOVIE_PATHS_BY_GAME: [&str; 3] = [
    "movies/corei7_intro.bik",
    "movies/ntw_intro.bik",
    "movies/sega_logo_sting_hd.bik",
];

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub fn prepare_skip_intro_videos(reserved_pack: &mut Pack) -> Result<()> {
    for path in INTRO_MOVIE_PATHS_BY_GAME {
        let file = RFile::new_from_vec(&EMPTY_BIK, FileType::Video, 0, path);
        reserved_pack.files_mut().insert(path.to_string(), file);
    }

    Ok(())
}
