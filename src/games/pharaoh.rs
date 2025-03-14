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

use rpfm_lib::schema::Schema;
use rpfm_lib::files::{Container, ContainerPath, DecodeableExtraData, EncodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded, table::DecodedData};
use rpfm_lib::games::GameInfo;

use super::rename_file_name_to_low_priority;

const SCRIPT_DEBUG_ACTIVATOR_PATH: &str = "script/enable_console_logging";

const INTRO_MOVIE_KEYS: [&str; 3] = [
    "startup_movie_01",
    "startup_movie_02",
    "startup_movie_03",
];

// These are videos that cannot be replaced with empty ones, or the game will crash.
const NON_REPLACEABLE_VIDEOS: [&str; 13] = [
    "movies/epilepsy_warning/epilepsy_warning_br.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_cn.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_cz.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_de.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_en.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_es.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_fr.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_it.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_kr.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_pl.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_ru.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_tr.ca_vp8",
    "movies/epilepsy_warning/epilepsy_warning_zh.ca_vp8",
];

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

pub fn prepare_script_logging(reserved_pack: &mut Pack) -> Result<()> {
    let file = RFile::new_from_vec("why not working?!!".as_bytes(), FileType::Text, 0, SCRIPT_DEBUG_ACTIVATOR_PATH);
    reserved_pack.files_mut().insert(SCRIPT_DEBUG_ACTIVATOR_PATH.to_string(), file);

    Ok(())
}

pub fn prepare_skip_intro_videos(game: &GameInfo, reserved_pack: &mut Pack, vanilla_pack: &mut Pack, modded_pack: &mut Pack, schema: &Schema) -> Result<()> {
    let mut videos = vanilla_pack.files_by_path(&ContainerPath::Folder("db/videos_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    // This one is to fix the multimedia player in the main menu.
    let mut campaign_videos = vanilla_pack.files_by_path(&ContainerPath::Folder("db/campaign_videos_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    //let mut locs = vanilla_pack.files_by_type(&[FileType::Loc])
    //    .into_iter()
    //    .cloned()
    //    .collect::<Vec<_>>();

    let non_replaceable_videos_paths = NON_REPLACEABLE_VIDEOS.iter().map(|path| ContainerPath::File(path.to_string())).collect::<Vec<_>>();
    let mut non_replaceable_videos = vanilla_pack.files_by_paths(&non_replaceable_videos_paths, true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    // Give the daracores extreme low priority so they don't overwrite other mods tables.
    videos.iter_mut().for_each(rename_file_name_to_low_priority);
    campaign_videos.iter_mut().for_each(rename_file_name_to_low_priority);

    videos.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/videos_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    campaign_videos.append(&mut modded_pack.files_by_path(&ContainerPath::Folder("db/campaign_videos_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    //locs.append(&mut modded_pack.files_by_type(&[FileType::Loc])
    //    .into_iter()
    //    .cloned()
    //    .collect::<Vec<_>>());

    non_replaceable_videos.append(&mut modded_pack.files_by_paths(&non_replaceable_videos_paths, true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());


    videos.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/videos_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    campaign_videos.append(&mut reserved_pack.files_by_path(&ContainerPath::Folder("db/campaign_videos_tables/".to_string()), true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    //locs.append(&mut reserved_pack.files_by_type(&[FileType::Loc])
    //    .into_iter()
    //    .cloned()
    //    .collect::<Vec<_>>());

    non_replaceable_videos.append(&mut reserved_pack.files_by_paths(&non_replaceable_videos_paths, true)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>());

    // Decode each table, modify it, then re-encode it and add it.
    let enc_extra_data = Some(EncodeableExtraData::new_from_game_info(game));
    let mut dec_extra_data = DecodeableExtraData::default();
    dec_extra_data.set_schema(Some(schema));
    let dec_extra_data = Some(dec_extra_data);

    for table in &mut videos {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            let definition = data.definition();
            let video_name = definition.column_position_by_name("video_name");

            for row in data.data_mut() {
                if let Some(video_name_column) = video_name {

                    if let Some(DecodedData::StringU8(key)) = row.get(video_name_column).cloned() {
                        if INTRO_MOVIE_KEYS.contains(&&*key) {
                            if let Some(DecodedData::StringU8(value)) = row.get_mut(video_name_column) {
                                value.push_str("dummy");
                            }
                        }
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    // NOTE: This breaks the video title in the multimedia player, and the playback.
    for table in &mut campaign_videos {
        if let Some(RFileDecoded::DB(mut data)) = table.decode(&dec_extra_data, false, true)? {
            let definition = data.definition();
            let video_name = definition.column_position_by_name("video_name");

            for row in data.data_mut() {
                if let Some(video_name_column) = video_name {

                    if let Some(DecodedData::StringU8(key)) = row.get(video_name_column).cloned() {
                        if INTRO_MOVIE_KEYS.contains(&&*key) {

                            if let Some(DecodedData::StringU8(value)) = row.get_mut(video_name_column) {
                                value.push_str("dummy");
                            }
                        }
                    }
                }
            }

            table.set_decoded(RFileDecoded::DB(data))?;
            table.encode(&enc_extra_data, false, true, false)?;
            reserved_pack.insert(table.clone())?;
        }
    }

    for file in &mut non_replaceable_videos {
        if let Some(RFileDecoded::Video(mut video)) = file.decode(&dec_extra_data, false, true)? {
            video.set_num_frames(2);
            video.set_framerate(30.0);

            file.set_decoded(RFileDecoded::Video(video))?;
            reserved_pack.insert(file.clone())?;
        }
    }

    Ok(())
}
