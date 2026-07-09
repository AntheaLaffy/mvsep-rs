//! Database integration tests.
//!
//! Tests for schema migrations, output format storage with bit-depth info,
//! algorithm-format associations, and config management.
//!
//! Run with:
//! ```bash
//! cargo test --test db_integration -- --nocapture
//! ```

/// Helper to create a temporary database for testing
fn temp_db() -> (tempfile::TempDir, mvsep_api_tester::db::Database) {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let db = mvsep_api_tester::db::Database::new(Some(&db_path.to_string_lossy()))
        .expect("Failed to create test database");
    (dir, db)
}

// ── Schema & Migration Tests ──

#[test]
fn test_migration_creates_tables() {
    let (_dir, db) = temp_db();

    db.with_conn(|conn| {
        // Verify all expected tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(
            tables.contains(&"output_formats".to_string()),
            "output_formats table missing"
        );
        assert!(
            tables.contains(&"algorithm_output_formats".to_string()),
            "algorithm_output_formats table missing"
        );
        assert!(
            tables.contains(&"algorithms".to_string()),
            "algorithms table missing"
        );
        assert!(
            tables.contains(&"algorithm_fields".to_string()),
            "algorithm_fields table missing"
        );
        assert!(
            tables.contains(&"algorithm_groups".to_string()),
            "algorithm_groups table missing"
        );
        assert!(
            tables.contains(&"config".to_string()),
            "config table missing"
        );
        assert!(tables.contains(&"tasks".to_string()), "tasks table missing");
        assert!(
            tables.contains(&"task_history".to_string()),
            "task_history table missing"
        );
        assert!(
            tables.contains(&"presets".to_string()),
            "presets table missing"
        );
        assert!(
            tables.contains(&"log_entries".to_string()),
            "log_entries table missing"
        );

        // Verify schema version
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 3, "Expected schema v3");
        Ok(())
    })
    .expect("DB query failed");
}

#[test]
fn test_output_formats_have_new_columns() {
    let (_dir, db) = temp_db();

    db.with_conn(|conn| {
        // Insert a test format
        conn.execute(
            "INSERT INTO output_formats (id, name, bits_per_sample, extension, is_premium) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![99, "Test Format", 24, "test", 1],
        ).unwrap();

        // Read it back
        let mut stmt = conn.prepare("SELECT id, name, bits_per_sample, extension, is_premium FROM output_formats WHERE id = 99").unwrap();
        let row = stmt.query_row([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i32>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i32>(4)?,
            ))
        }).unwrap();

        assert_eq!(row.0, 99);
        assert_eq!(row.1, "Test Format");
        assert_eq!(row.2, Some(24));
        assert_eq!(row.3, "test");
        assert_eq!(row.4, 1);
        Ok(())
    })
    .expect("DB query failed");
}

// ── Output Format Repository Tests ──

#[test]
fn test_init_default_output_formats() {
    let (_dir, db) = temp_db();

    db.with_conn(|conn| {
        let count = mvsep_api_tester::db::repositories::init_default_output_formats(conn)
            .expect("init_default_output_formats failed");
        assert_eq!(count, 6, "Expected 6 default formats");
        Ok(())
    })
    .expect("DB init failed");

    db.with_conn(|conn| {
        let formats = mvsep_api_tester::db::repositories::get_all_output_formats(conn)
            .expect("get_all_output_formats failed");
        assert_eq!(formats.len(), 6);

        // Check each format has correct fields
        let mp3 = formats.iter().find(|f| f.id == 0).unwrap();
        assert_eq!(mp3.name, "MP3 (320 kbps)");
        assert_eq!(mp3.bits_per_sample, None);
        assert_eq!(mp3.extension, "mp3");
        assert!(!mp3.is_premium);

        let wav16 = formats.iter().find(|f| f.id == 1).unwrap();
        assert_eq!(wav16.name, "WAV (16 bit)");
        assert_eq!(wav16.bits_per_sample, Some(16));
        assert_eq!(wav16.extension, "wav");
        assert!(!wav16.is_premium);

        let flac16 = formats.iter().find(|f| f.id == 2).unwrap();
        assert_eq!(flac16.name, "FLAC (16 bit)");
        assert_eq!(flac16.bits_per_sample, Some(16));
        assert_eq!(flac16.extension, "flac");
        assert!(!flac16.is_premium);

        let m4a = formats.iter().find(|f| f.id == 3).unwrap();
        assert_eq!(m4a.name, "M4A (lossy)");
        assert_eq!(m4a.bits_per_sample, None);
        assert_eq!(m4a.extension, "m4a");
        assert!(!m4a.is_premium);

        let wav32 = formats.iter().find(|f| f.id == 4).unwrap();
        assert_eq!(wav32.name, "WAV (32 bit)");
        assert_eq!(wav32.bits_per_sample, Some(32));
        assert_eq!(wav32.extension, "wav");
        assert!(wav32.is_premium, "WAV32 should be premium");

        let flac24 = formats.iter().find(|f| f.id == 5).unwrap();
        assert_eq!(flac24.name, "FLAC (24 bit)");
        assert_eq!(flac24.bits_per_sample, Some(24));
        assert_eq!(flac24.extension, "flac");
        assert!(flac24.is_premium, "FLAC24 should be premium");
        Ok(())
    })
    .expect("DB query failed");
}

#[test]
fn test_upsert_output_format() {
    let (_dir, db) = temp_db();

    db.with_conn(|conn| {
        let fmt = mvsep_api_tester::db::repositories::OutputFormatRow {
            id: 42,
            name: "Custom Format".into(),
            bits_per_sample: Some(16),
            extension: "custom".into(),
            is_premium: true,
        };
        mvsep_api_tester::db::repositories::upsert_output_format(conn, &fmt)
            .expect("upsert_output_format failed");

        let formats = mvsep_api_tester::db::repositories::get_all_output_formats(conn).unwrap();
        let found = formats.iter().find(|f| f.id == 42).unwrap();
        assert_eq!(found.name, "Custom Format");
        assert_eq!(found.bits_per_sample, Some(16));
        assert_eq!(found.extension, "custom");
        assert!(found.is_premium);
        Ok(())
    })
    .expect("DB query failed");
}

#[test]
fn test_upsert_output_format_preserves_algorithm_associations() {
    let (_dir, db) = temp_db();

    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO algorithm_groups (id, name) VALUES (1, 'Test Group')",
            [],
        )?;
        conn.execute(
            "INSERT INTO algorithms (id, name, group_id) VALUES (1, 'Test Algo', 1)",
            [],
        )?;
        mvsep_api_tester::db::repositories::init_default_output_formats(conn)?;
        mvsep_api_tester::db::repositories::set_algorithm_output_formats(conn, 1, &[1])?;

        let updated = mvsep_api_tester::db::repositories::OutputFormatRow {
            id: 1,
            name: "WAV (16 bit updated)".into(),
            bits_per_sample: Some(16),
            extension: "wav".into(),
            is_premium: false,
        };
        mvsep_api_tester::db::repositories::upsert_output_format(conn, &updated)?;

        let formats = mvsep_api_tester::db::repositories::get_formats_for_algorithm(conn, 1)?;
        assert_eq!(formats.len(), 1);
        assert_eq!(formats[0].id, 1);
        assert_eq!(formats[0].name, "WAV (16 bit updated)");
        Ok(())
    })
    .expect("DB query failed");
}

// ── Algorithm-Format Association Tests ──

#[test]
fn test_algorithm_output_formats() {
    let (_dir, db) = temp_db();

    // Insert test data
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO algorithm_groups (id, name) VALUES (1, 'Test Group')",
            [],
        )?;
        conn.execute(
            "INSERT INTO algorithms (id, name, group_id) VALUES (1, 'Test Algo 1', 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO algorithms (id, name, group_id) VALUES (2, 'Test Algo 2', 1)",
            [],
        )?;
        mvsep_api_tester::db::repositories::init_default_output_formats(conn)?;
        Ok(())
    })
    .expect("Setup failed");

    // Set formats for algorithm 1 (only IDs 0, 1, 2)
    db.with_conn(|conn| {
        mvsep_api_tester::db::repositories::set_algorithm_output_formats(conn, 1, &[0, 1, 2])
            .expect("set_algorithm_output_formats failed");
        Ok(())
    })
    .expect("Set formats failed");

    // Verify algorithm 1: should have 3 formats
    db.with_conn(|conn| {
        let formats = mvsep_api_tester::db::repositories::get_formats_for_algorithm(conn, 1)
            .expect("get_formats_for_algorithm failed");
        assert_eq!(formats.len(), 3, "Algo 1 should have 3 formats");
        assert_eq!(formats[0].id, 0);
        assert_eq!(formats[1].id, 1);
        assert_eq!(formats[2].id, 2);
        Ok(())
    })
    .expect("Query algo 1 failed");

    // Verify algorithm 2: should have no formats (none set yet)
    db.with_conn(|conn| {
        let formats = mvsep_api_tester::db::repositories::get_formats_for_algorithm(conn, 2)
            .expect("get_formats_for_algorithm failed");
        assert_eq!(formats.len(), 0, "Algo 2 should have 0 formats");
        Ok(())
    })
    .expect("Query algo 2 failed");
}

#[test]
fn test_init_default_algorithm_format_associations() {
    let (_dir, db) = temp_db();

    // Insert test data
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO algorithm_groups (id, name) VALUES (1, 'Test Group')",
            [],
        )?;
        conn.execute(
            "INSERT INTO algorithms (id, name, group_id) VALUES (10, 'Algo A', 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO algorithms (id, name, group_id) VALUES (20, 'Algo B', 1)",
            [],
        )?;
        mvsep_api_tester::db::repositories::init_default_output_formats(conn)?;
        Ok(())
    })
    .expect("Setup failed");

    // Init default associations (all algorithms × all formats)
    db.with_conn(|conn| {
        let count =
            mvsep_api_tester::db::repositories::init_default_algorithm_format_associations(conn)
                .expect("init_default_algorithm_format_associations failed");
        // 2 algorithms × 6 formats = 12 associations
        assert!(
            count >= 12,
            "Expected at least 12 associations, got {}",
            count
        );
        Ok(())
    })
    .expect("Init associations failed");

    // Verify algorithm 10 has all 6 formats
    db.with_conn(|conn| {
        let formats = mvsep_api_tester::db::repositories::get_formats_for_algorithm(conn, 10)
            .expect("get_formats_for_algorithm failed");
        assert_eq!(formats.len(), 6, "Algo A should have all 6 formats");
        Ok(())
    })
    .expect("Query failed");

    // Verify algorithm 20 has all 6 formats
    db.with_conn(|conn| {
        let formats = mvsep_api_tester::db::repositories::get_formats_for_algorithm(conn, 20)
            .expect("get_formats_for_algorithm failed");
        assert_eq!(formats.len(), 6, "Algo B should have all 6 formats");
        Ok(())
    })
    .expect("Query failed");
}

#[test]
fn test_remove_algorithm_output_formats() {
    let (_dir, db) = temp_db();

    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO algorithm_groups (id, name) VALUES (1, 'Test Group')",
            [],
        )?;
        conn.execute(
            "INSERT INTO algorithms (id, name, group_id) VALUES (99, 'Algo', 1)",
            [],
        )?;
        mvsep_api_tester::db::repositories::init_default_output_formats(conn)?;
        Ok(())
    })
    .expect("Setup failed");

    // Set and verify formats
    db.with_conn(|conn| {
        mvsep_api_tester::db::repositories::set_algorithm_output_formats(conn, 99, &[0, 1, 2])?;
        let before = mvsep_api_tester::db::repositories::get_formats_for_algorithm(conn, 99)?;
        assert_eq!(before.len(), 3);
        Ok(())
    })
    .expect("Set failed");

    // Remove and verify
    db.with_conn(|conn| {
        mvsep_api_tester::db::repositories::remove_algorithm_output_formats(conn, 99)?;
        let after = mvsep_api_tester::db::repositories::get_formats_for_algorithm(conn, 99)?;
        assert_eq!(after.len(), 0, "Formats should be empty after removal");
        Ok(())
    })
    .expect("Remove failed");
}

#[test]
fn test_get_all_algorithm_format_associations() {
    let (_dir, db) = temp_db();

    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO algorithm_groups (id, name) VALUES (1, 'Test Group')",
            [],
        )?;
        conn.execute(
            "INSERT INTO algorithms (id, name, group_id) VALUES (1, 'A', 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO algorithms (id, name, group_id) VALUES (2, 'B', 1)",
            [],
        )?;
        mvsep_api_tester::db::repositories::init_default_output_formats(conn)?;
        Ok(())
    })
    .expect("Setup failed");

    db.with_conn(|conn| {
        mvsep_api_tester::db::repositories::set_algorithm_output_formats(conn, 1, &[0, 1])?;
        mvsep_api_tester::db::repositories::set_algorithm_output_formats(conn, 2, &[2, 3])?;
        Ok(())
    })
    .expect("Set formats failed");

    db.with_conn(|conn| {
        let all = mvsep_api_tester::db::repositories::get_all_algorithm_format_associations(conn)
            .expect("get_all_algorithm_format_associations failed");
        assert_eq!(
            all.len(),
            4,
            "Expected 4 associations (2 algos × 2 formats each)"
        );
        Ok(())
    })
    .expect("Query failed");
}

#[test]
fn test_replace_algorithm_cache_rebuilds_rows_and_default_formats() {
    let (_dir, db) = temp_db();

    let mut conn = db.conn.lock().expect("DB lock failed");
    mvsep_api_tester::db::repositories::replace_algorithm_cache(
        &mut conn,
        &[mvsep_api_tester::db::repositories::AlgorithmGroupRow {
            id: 7,
            name: "Vocals".into(),
        }],
        &[mvsep_api_tester::db::repositories::AlgorithmRow {
            id: 26,
            name: "Old Name".into(),
            group_id: 7,
            price_coefficient: 1.5,
            orientation: 0,
        }],
        &[mvsep_api_tester::db::repositories::AlgorithmFieldRow {
            id: 2601,
            algorithm_id: 26,
            name: "add_opt1".into(),
            text: Some("Model".into()),
            options: Some(r#"{"0":"Default"}"#.into()),
            default_key: Some("0".into()),
        }],
    )
    .expect("first replace_algorithm_cache failed");

    let (algo, fields) =
        mvsep_api_tester::db::repositories::get_algorithm_details_with_fields(&conn, 26)
            .expect("details query failed")
            .expect("algorithm 26 missing");
    assert_eq!(algo.name, "Old Name");
    assert_eq!(fields.len(), 1);
    let formats = mvsep_api_tester::db::repositories::get_formats_for_algorithm(&conn, 26)
        .expect("formats query failed");
    assert_eq!(formats.len(), 6);

    mvsep_api_tester::db::repositories::replace_algorithm_cache(
        &mut conn,
        &[mvsep_api_tester::db::repositories::AlgorithmGroupRow {
            id: 8,
            name: "Drums".into(),
        }],
        &[mvsep_api_tester::db::repositories::AlgorithmRow {
            id: 99,
            name: "New Algo".into(),
            group_id: 8,
            price_coefficient: 1.0,
            orientation: 1,
        }],
        &[],
    )
    .expect("second replace_algorithm_cache failed");

    assert!(
        mvsep_api_tester::db::repositories::get_algorithm_by_id(&conn, 26)
            .expect("stale query failed")
            .is_none(),
        "stale algorithm should be removed during cache replacement"
    );
    assert_eq!(
        mvsep_api_tester::db::repositories::count_algorithms(&conn)
            .expect("count_algorithms failed"),
        1
    );
    let formats = mvsep_api_tester::db::repositories::get_formats_for_algorithm(&conn, 99)
        .expect("new formats query failed");
    assert_eq!(formats.len(), 6);
}

#[test]
fn test_replace_algorithm_cache_preserves_referenced_stale_algorithms() {
    let (_dir, db) = temp_db();

    let mut conn = db.conn.lock().expect("DB lock failed");
    mvsep_api_tester::db::repositories::replace_algorithm_cache(
        &mut conn,
        &[mvsep_api_tester::db::repositories::AlgorithmGroupRow {
            id: 1,
            name: "Old Group".into(),
        }],
        &[mvsep_api_tester::db::repositories::AlgorithmRow {
            id: 1,
            name: "Referenced Old Algo".into(),
            group_id: 1,
            price_coefficient: 1.0,
            orientation: 0,
        }],
        &[],
    )
    .expect("initial replace_algorithm_cache failed");

    conn.execute(
        "INSERT INTO tasks (hash, file_name, algorithm_id, algorithm_name, format, status, created_at)
         VALUES ('hash-1', 'song.wav', 1, 'Referenced Old Algo', 1, 'waiting', 1)",
        [],
    )
    .expect("task insert failed");
    conn.execute(
        "INSERT INTO presets (id, name, algorithm_id, format_id)
         VALUES ('preset-1', 'Preset', 1, 1)",
        [],
    )
    .expect("preset insert failed");

    mvsep_api_tester::db::repositories::replace_algorithm_cache(
        &mut conn,
        &[mvsep_api_tester::db::repositories::AlgorithmGroupRow {
            id: 2,
            name: "New Group".into(),
        }],
        &[mvsep_api_tester::db::repositories::AlgorithmRow {
            id: 2,
            name: "Current Algo".into(),
            group_id: 2,
            price_coefficient: 1.0,
            orientation: 0,
        }],
        &[],
    )
    .expect("replace should not fail while task/preset reference stale algorithm");

    assert!(
        mvsep_api_tester::db::repositories::get_algorithm_by_id(&conn, 1)
            .expect("old algorithm query failed")
            .is_none(),
        "stale referenced algorithm should be hidden from the current cache view"
    );
    assert!(
        mvsep_api_tester::db::repositories::get_algorithm_by_id(&conn, 2)
            .expect("new algorithm query failed")
            .is_some(),
        "new algorithm should be visible"
    );
    let stale_cached: i32 = conn
        .query_row("SELECT is_cached FROM algorithms WHERE id = 1", [], |row| {
            row.get(0)
        })
        .expect("stale algorithm row should remain for foreign keys");
    assert_eq!(stale_cached, 0);

    let current = mvsep_api_tester::db::repositories::get_all_algorithms(&conn)
        .expect("current algorithm query failed");
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].id, 2);
    let old_formats = mvsep_api_tester::db::repositories::get_formats_for_algorithm(&conn, 1)
        .expect("old formats query failed");
    assert!(old_formats.is_empty());
    let new_formats = mvsep_api_tester::db::repositories::get_formats_for_algorithm(&conn, 2)
        .expect("new formats query failed");
    assert_eq!(new_formats.len(), 6);
}

// ── Config Tests ──

#[test]
fn test_config_save_and_load() {
    let (_dir, db) = temp_db();

    let config = mvsep_api_tester::db::repositories::ConfigRow {
        token: Some("test_token_123".into()),
        api_url: Some("https://example.com".into()),
        mirror: Some("main".into()),
        proxy_mode: Some("manual".into()),
        proxy_host: Some("10.0.0.1".into()),
        proxy_port: Some("9999".into()),
        output_dir: Some("/tmp/output".into()),
        output_format: Some(5),
        poll_interval: Some(10),
        algorithm_auto_refresh_days: Some(30),
    };

    db.with_conn(|conn| {
        mvsep_api_tester::db::repositories::save_config(conn, &config).expect("save_config failed");
        Ok(())
    })
    .expect("Save failed");

    db.with_conn(|conn| {
        let loaded = mvsep_api_tester::db::repositories::get_config(conn)
            .expect("get_config failed")
            .expect("Config should exist");
        assert_eq!(loaded.token, Some("test_token_123".into()));
        assert_eq!(loaded.api_url, Some("https://example.com".into()));
        assert_eq!(loaded.proxy_mode, Some("manual".into()));
        assert_eq!(loaded.proxy_host, Some("10.0.0.1".into()));
        assert_eq!(loaded.proxy_port, Some("9999".into()));
        assert_eq!(loaded.output_format, Some(5));
        assert_eq!(loaded.poll_interval, Some(10));
        assert_eq!(loaded.algorithm_auto_refresh_days, Some(30));
        Ok(())
    })
    .expect("Load failed");
}

#[test]
fn test_config_partial_update() {
    let (_dir, db) = temp_db();

    // Save initial config
    db.with_conn(|conn| {
        let cfg = mvsep_api_tester::db::repositories::ConfigRow {
            token: Some("initial".into()),
            api_url: Some("https://initial.com".into()),
            mirror: Some("main".into()),
            proxy_mode: Some("system".into()),
            proxy_host: None,
            proxy_port: None,
            output_dir: None,
            output_format: Some(1),
            poll_interval: None,
            algorithm_auto_refresh_days: None,
        };
        mvsep_api_tester::db::repositories::save_config(conn, &cfg)?;
        Ok(())
    })
    .expect("Initial save failed");

    // Update only token
    db.with_conn(|conn| {
        let current = mvsep_api_tester::db::repositories::get_config(conn)
            .expect("get_config failed")
            .unwrap();
        let updated = mvsep_api_tester::db::repositories::ConfigRow {
            token: Some("updated_token".into()),
            ..current
        };
        mvsep_api_tester::db::repositories::save_config(conn, &updated)?;
        Ok(())
    })
    .expect("Update failed");

    // Verify token changed but other fields preserved
    db.with_conn(|conn| {
        let loaded = mvsep_api_tester::db::repositories::get_config(conn)
            .expect("get_config failed")
            .unwrap();
        assert_eq!(loaded.token, Some("updated_token".into()));
        assert_eq!(
            loaded.api_url,
            Some("https://initial.com".into()),
            "api_url should be preserved"
        );
        assert_eq!(
            loaded.output_format,
            Some(1),
            "output_format should be preserved"
        );
        Ok(())
    })
    .expect("Verification failed");
}

// ── Preset Tests ──

#[test]
fn test_preset_crud() {
    let (_dir, db) = temp_db();

    // Setup: algorithm must exist
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO algorithm_groups (id, name) VALUES (1, 'Test Group')",
            [],
        )?;
        conn.execute(
            "INSERT INTO algorithms (id, name, group_id) VALUES (1, 'Test Algo', 1)",
            [],
        )?;
        Ok(())
    })
    .expect("Setup failed");

    let preset = mvsep_api_tester::db::repositories::PresetRow {
        id: "preset_1".into(),
        name: "My Preset".into(),
        algorithm_id: 1,
        opt1: Some(2),
        opt2: None,
        opt3: None,
        format_id: 1,
        demo: false,
    };

    db.with_conn(|conn| {
        mvsep_api_tester::db::repositories::upsert_preset(conn, &preset)?;
        let presets = mvsep_api_tester::db::repositories::get_all_presets(conn)?;
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].name, "My Preset");
        assert_eq!(presets[0].format_id, 1);
        Ok(())
    })
    .expect("Preset test failed");

    // Delete preset
    db.with_conn(|conn| {
        let deleted = mvsep_api_tester::db::repositories::delete_preset(conn, "preset_1")?;
        assert!(deleted);
        let presets = mvsep_api_tester::db::repositories::get_all_presets(conn)?;
        assert!(presets.is_empty());
        Ok(())
    })
    .expect("Preset delete failed");
}
