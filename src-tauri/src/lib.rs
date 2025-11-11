// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::env;
use std::path::Path;
use std::time::Duration;
use tauri::{Manager, Position, PhysicalPosition, Emitter, AppHandle};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri::menu::{Menu, MenuItem};
use tokio::time::sleep;
use serde_json;

#[tauri::command]
fn greet(name: &str) -> String {
    log::error!("something bad happened!");
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn is_directory(path: &str) -> bool {
    Path::new(&path).is_dir()
}

#[tauri::command]
async fn show_alert_window(app: tauri::AppHandle, message: String) -> Result<(), String> {
    // 알람창이 이미 열려있는지 확인
    if let Some(alert_window) = app.get_webview_window("alert") {
        // 이미 열려있다면 메시지만 업데이트하고 표시
        let size = tauri::LogicalSize::new(380.0, 220.0); // 실제 컨텐츠 크기
        alert_window.set_size(size).map_err(|e| e.to_string())?;
        alert_window.emit("show-alert", message).map_err(|e| e.to_string())?;
        alert_window.show().map_err(|e| e.to_string())?;
        alert_window.set_focus().map_err(|e| e.to_string())?;

        #[cfg(target_os = "windows")]
        {
            alert_window.set_ignore_cursor_events(true)
                .map_err(|e| e.to_string())?;
        }
    } else {
        // 알람창이 없다면 새로 생성
        let alert_window = tauri::WebviewWindowBuilder::new(
            &app,
            "alert",
            tauri::WebviewUrl::App("alert".into())
        )
        .title("알림")
        .inner_size(480.0, 300.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;

        // 창을 우측 상단으로 위치 조정
        position_alert_window_internal(&alert_window).await?;
        
        // 메시지 전송 후 표시
        alert_window.emit("show-alert", message).map_err(|e| e.to_string())?;
        alert_window.show().map_err(|e| e.to_string())?;
        alert_window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn enable_alert_interaction(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("alert") {
        #[cfg(target_os = "windows")]
        {
            window.set_ignore_cursor_events(false)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn disable_alert_interaction(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("alert") {
        #[cfg(target_os = "windows")]
        {
            window.set_ignore_cursor_events(true)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

async fn position_alert_window_internal(window: &tauri::WebviewWindow) -> Result<(), String> {
    // 화면 크기 가져오기
    let monitor = window.primary_monitor().map_err(|e| e.to_string())?;
    let (width, _height) = if let Some(m) = monitor {
        let size = m.size();
        (size.width, size.height)
    } else {
        (1920, 1080)
    };
    
    // 우측 상단 위치 계산
    let x = width - 480 - 20; // 창 너비 + 여백
    let y = 20; // 상단 여백
    
    window.set_position(Position::Physical(PhysicalPosition::new(x as i32, y as i32)))
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn close_alert_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(alert_window) = app.get_webview_window("alert") {
        alert_window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn alert_closed(app: tauri::AppHandle) -> Result<(), String> {
    // 알람창이 닫혔을 때 메인 창에 알림
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.emit("alert-closed", ()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn auto_close_alert_window(app: tauri::AppHandle, delay_seconds: u64) -> Result<(), String> {
    // 지정된 시간 후 자동으로 alert 창 닫기
    let app_handle = app.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(delay_seconds)).await;
        if let Some(alert_window) = app_handle.get_webview_window("alert") {
            let _ = alert_window.close();
        }
    });
    Ok(())
}

#[tauri::command]
async fn position_alert_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(alert_window) = app.get_webview_window("alert") {
        position_alert_window_internal(&alert_window).await?;
    }
    Ok(())
}

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Keyword {
    id: u64,
    #[serde(rename = "group_id")]
    group_id: u64,
    #[serde(rename = "type")]
    type_: String,
    content: String,
    editable: bool,
}

#[tauri::command]
async fn search_advanced(search_id: String, keywords: HashMap<String, Vec<Keyword>>) {
    println!("search_id : {:?}", search_id);
    
    // 모든 키워드를 하나의 벡터로 평탄화
    let all_keywords: Vec<&Keyword> = keywords.values().flatten().collect();
    
    println!("Total keywords count: {}", all_keywords.len());
    
    // 그룹별로 출력
    for (group_id, keyword_list) in &keywords {
        println!("Group {} has {} keywords:", group_id, keyword_list.len());
        for keyword in keyword_list {
            println!("  - id: {}, type: {}, content: {}", keyword.id, keyword.type_, keyword.content);
        }
    }
    
    // 또는 모든 키워드를 하나의 Vec로 추출
    let flattened_keywords: Vec<Keyword> = keywords
        .into_values()
        .flatten()
        .collect();
    
    println!("Flattened keywords: {:?}", flattened_keywords);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let platform = tauri_plugin_os::platform();
    println!("Platform: {}", platform);

    println!(
        "current directory: {}",
        env::current_dir().unwrap().display()
    );
    log::error!("something bad happened!");
    log::info!("Tauri is awesome!");

    tauri::Builder::default()
        .setup(|app| {
            // 트레이 아이콘 메뉴 생성
            let quit = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "창 보이기", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "창 숨기기", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

            // 트레이 아이콘 생성
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        // 트레이 아이콘 좌클릭 시 메인 창 토글
                        let app = tray.app_handle();
                        if let Some(main_window) = app.get_webview_window("main") {
                            if main_window.is_visible().unwrap_or(false) {
                                let _ = main_window.hide();
                            } else {
                                let _ = main_window.show();
                                let _ = main_window.set_focus();
                            }
                        }
                    }
                    _ => {}
                })
                .on_menu_event(|tray, event| match event.id.as_ref() {
                    "quit" => {
                        // 모든 창 닫기
                        let app = tray.app_handle();
                        if let Some(main_window) = app.get_webview_window("main") {
                            let _ = main_window.close();
                        }
                        if let Some(alert_window) = app.get_webview_window("alert") {
                            let _ = alert_window.close();
                        }
                        app.exit(0);
                    }
                    "show" => {
                        let app = tray.app_handle();
                        if let Some(main_window) = app.get_webview_window("main") {
                            let _ = main_window.show();
                            let _ = main_window.set_focus();
                        }
                    }
                    "hide" => {
                        let app = tray.app_handle();
                        if let Some(main_window) = app.get_webview_window("main") {
                            let _ = main_window.hide();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                match event {
                    tauri::WindowEvent::CloseRequested { .. } => {
                        // 메인 창이 닫힐 때 알람창도 닫기
                        if let Some(alert_window) = window.app_handle().get_webview_window("alert") {
                            let _ = alert_window.close();
                        }
                    }
                    _ => {}
                }
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_upload::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            is_directory, 
            show_alert_window, 
            close_alert_window, 
            alert_closed,
            auto_close_alert_window,
            position_alert_window,
            enable_alert_interaction,
            disable_alert_interaction,
            search_advanced
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
