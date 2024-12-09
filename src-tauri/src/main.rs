// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Duration;
use tauri::menu::{MenuBuilder, MenuId};
use tauri::tray::TrayIconBuilder;
use tokio::{fs, time};
async fn check_directory() {
    let start = time::Instant::now();
    println!("Directory check started at: {:?}", start);

    // ホームディレクトリのパスを取得
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let path = home_dir.join("test");

    // フォルダの存在確認と作成
    if !path.exists() {
        match fs::create_dir(&path).await {
            Ok(_) => println!("Created directory: {:?}", path),
            Err(e) => {
                println!("Error creating directory: {:?}", e);
                return;
            }
        }
    }

    // 10秒感覚での実行を確認するためのテスト実装
    loop {
        time::sleep(Duration::from_secs(10)).await;
        let elapsed = start.elapsed();
        println!("Check performed after: {:?}", elapsed);

        // ディレクトリ内の権限チェック
        match fs::metadata(&path).await {
            Ok(metadata) => {
                if !metadata.permissions().readonly() {
                    // ディレクトリ内のファイルを読み取り
                    match fs::read_dir(&path).await {
                        Ok(mut entries) => {
                            // ファイル情報を収集
                            let mut files = Vec::new();
                            while let Ok(Some(entry)) = entries.next_entry().await {
                                // ファイルの削除権限をここでチェック
                                if let Ok(metadata) = entry.metadata().await {
                                    if metadata.is_file() {
                                        let permissions = metadata.permissions();
                                        let path = entry.path();
                                        if permissions.readonly() {
                                            println!("Skipping read-only file: {:?}", path);
                                            continue;
                                        }
                                        files.push((
                                            entry.path(),
                                            metadata
                                                .modified()
                                                .unwrap_or_else(|_| std::time::SystemTime::now()),
                                        ));
                                    }
                                }
                            }
                            println!("Found {} files in directory", files.len());

                            // ファイル数が３を超える場合の処理
                            if files.len() > 3 {
                                println!("Too many files, removing oldest...");
                                // 更新日時でソート
                                files.sort_by_key(|k| k.1);
                                // 最も古いファイルを削除
                                if let Some((oldest_file, _)) = files.first() {
                                    match fs::remove_file(oldest_file).await {
                                        Ok(_) => println!("Removed file: {:?}", oldest_file),
                                        Err(e) => println!("Error removing file: {}", e),
                                    }
                                }
                            }
                        }
                        Err(e) => println!("Error reaading directory: {}", e),
                    }
                } else {
                    println!("No write permission for directory: {:?}", path);
                }
            }
            Err(e) => println!("Error checking directory permission: {}", e),
        }
    }
}
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            println!("Application is startging up...");

            // メニューの作成
            let menu = MenuBuilder::new(app)
                .text("status", "監視状況を確認")
                .separator()
                .text("set_limit", "ファイル制限数を設定")
                .separator()
                .text("quit", "終了")
                .build()?;

            // システムトレイにメニューを設定
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(true)
                .build(app)?;

            // 監視タスクの開始
            tauri::async_runtime::spawn(async { check_directory().await });
            Ok(())
        })
        .on_menu_event(move |app, event| match event.id() {
            id if *id == MenuId::from("status") => {
                println!("Status clicked");
            }
            id if *id == MenuId::from("set_limit") => {
                println!("Set limit clicked");
            }
            id if *id == MenuId::from("quit") => {
                println!("Quit clicked");
                app.exit(0);
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
