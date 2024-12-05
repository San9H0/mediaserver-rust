#[cfg(test)]
mod tests {
    #![allow(unused_imports, dead_code)]

    use super::*;
    use m3u8_rs::{MediaPlaylist, MediaSegment, Part, PreloadHint, ServerControl};

    fn create_test_playlist() -> MediaPlaylist {
        MediaPlaylist {
            version: Some(10),
            independent_segments: true,
            target_duration: 2,
            media_sequence: 0,
            map: Some(m3u8_rs::Map {
                uri: "init.mp4".to_string(),
                ..Default::default()
            }),
            discontinuity_sequence: 0,
            end_list: false,
            playlist_type: None,
            segments: vec![
                MediaSegment {
                    uri: "output_0_0.mp4".into(),
                    duration: 2.002,
                    title: Some("title".into()),
                    ..Default::default()
                },
                MediaSegment {
                    uri: "output_0_1.mp4".into(),
                    duration: 2.002,
                    title: Some("title".into()),
                    ..Default::default()
                },
                MediaSegment {
                    uri: "output_1_0.mp4".into(),
                    duration: 2.002,
                    title: Some("title".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    fn create_test_playlist_ext_x_part() -> MediaPlaylist {
        MediaPlaylist {
            version: Some(10),
            independent_segments: true,
            target_duration: 2,
            part_inf: Some(1.0),
            media_sequence: 100,
            map: Some(m3u8_rs::Map {
                uri: "init.mp4".to_string(),
                ..Default::default()
            }),
            discontinuity_sequence: 0,
            end_list: false,
            playlist_type: None,
            server_control: Some(ServerControl {
                can_block_reload: true,
                part_hold_back: Some(3.0510),
                can_skip_util: None,
            }),
            segments: vec![
                MediaSegment {
                    uri: "output_97.m4s".into(),
                    duration: 1.98999,
                    title: Some("title".into()),
                    parts: vec![
                        Part {
                            duration: 1.00098,
                            uri: "output_97_0.m4s".into(),
                            independent: true,
                        },
                        Part {
                            duration: 0.9999,
                            uri: "output_97_1.m4s".into(),
                            independent: true,
                        },
                    ],
                    ..Default::default()
                },
                MediaSegment {
                    uri: "output_98.m4s".into(),
                    duration: 2.00098,
                    title: Some("title".into()),
                    parts: vec![
                        Part {
                            duration: 1.00098,
                            uri: "output_98_0.m4s".into(),
                            independent: true,
                        },
                        Part {
                            duration: 1.00012,
                            uri: "output_98_1.m4s".into(),
                            independent: true,
                        },
                    ],
                    ..Default::default()
                },
                MediaSegment {
                    uri: "output_99.m4s".into(),
                    duration: 2.00098,
                    title: Some("title".into()),
                    parts: vec![
                        Part {
                            duration: 1.00098,
                            uri: "output_99_0.m4s".into(),
                            independent: true,
                        },
                        Part {
                            duration: 1.00000,
                            uri: "output_99_1.m4s".into(),
                            independent: true,
                        },
                    ],
                    ..Default::default()
                },
            ],
            parts: vec![Part {
                duration: 1.00098,
                uri: "output_100_0.m4s".into(),
                independent: true,
            }],
            preload_hint: Some(PreloadHint {
                r#type: "PART".to_string(),
                uri: "output_100_1.m4s".to_string(),
            }),
            ..Default::default()
        }
    }
    #[test]
    fn creates_playlist_with_correct_version() {
        let playlist = create_test_playlist();
        assert_eq!(playlist.version, Some(10));
    }

    #[test]
    fn creates_playlist_with_correct_target_duration() {
        let playlist = create_test_playlist();
        assert_eq!(playlist.target_duration, 2);
    }

    #[test]
    fn creates_playlist_with_correct_media_sequence() {
        let playlist = create_test_playlist();
        assert_eq!(playlist.media_sequence, 0);
    }

    #[test]
    fn creates_playlist_with_correct_discontinuity_sequence() {
        let playlist = create_test_playlist();
        assert_eq!(playlist.discontinuity_sequence, 0);
    }

    #[test]
    fn creates_playlist_with_correct_end_list() {
        let playlist = create_test_playlist();
        assert_eq!(playlist.end_list, false);
    }

    #[test]
    fn creates_playlist_with_correct_segments() {
        let playlist = create_test_playlist();
        let segment = &playlist.segments[0];

        let mut buffer = Vec::new();
        if let Err(_) = playlist.write_to(&mut buffer) {
            return;
        };
        let m3u8 = String::from_utf8(buffer).expect("Invalid UTF-8 sequence");
        println!("{}", m3u8);
    }

    #[test]
    fn creates_playlist_with_ext_x_part() {
        let playlist = create_test_playlist_ext_x_part();
        let mut buffer = Vec::new();
        if let Err(_) = playlist.write_to(&mut buffer) {
            return;
        }
        let m3u8 = String::from_utf8(buffer).expect("Invalid UTF-8 sequence");
        println!("{}", m3u8);
    }
}
