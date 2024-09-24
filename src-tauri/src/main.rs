use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayMenuItem,  SystemTrayEvent};
use tauri::Manager;


extern crate winapi;
use tauri::command;
use winapi::um::winuser::{EnumWindows, GetWindowTextW, IsWindowVisible, GetWindow, GWL_EXSTYLE, GW_OWNER, GetWindowLongW, GWL_STYLE, ShowWindow, WS_CAPTION, SW_RESTORE, SW_SHOW, GetWindowThreadProcessId};
use winapi::um::psapi::GetModuleFileNameExW;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
use winapi::um::handleapi::CloseHandle;
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{DWORD, FALSE};
use std::ptr::null_mut;
use std::ffi::{OsString, CString};
use std::os::windows::ffi::{OsStringExt, OsStrExt};
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use std::path::Path;

#[derive(serde::Serialize)]
struct Window {
    title: String,
    application_name: String,
    hwnd: usize, // Store the HWND as a usize to send it to the frontend
}

#[command]
fn get_open_window_apps() -> Vec<Window> {
    let mut windows: HashMap<HWND, (String, String, HWND)> = HashMap::new();

    unsafe {
        EnumWindows(Some(enum_window), &mut windows as *mut _ as isize);
    }

    windows.into_iter().map(|(_, (title, app_name, hwnd))| Window {
        title,
        application_name: app_name, // Ensure this field is populated
        hwnd: hwnd as usize,
    }).collect()
}

unsafe extern "system" fn enum_window(hwnd: HWND, lparam: isize) -> i32 {
    let windows = &mut *(lparam as *mut HashMap<HWND, (String, String, HWND)>);

    // Check if the window is visible and meets other taskbar criteria
    if is_taskbar_window(hwnd) {
        let mut buffer = [0u16; 512];
        let length = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);

        if length > 0 {
            let window_title = OsString::from_wide(&buffer[..length as usize]).to_string_lossy().into_owned();

            if !window_title.is_empty() {
                // Get the application name
                let app_name = get_application_name(hwnd);

                windows.insert(hwnd, (window_title, app_name, hwnd));
            }
        }
    }

    1 // Continue enumeration
}

// Function to check if the window should appear in the taskbar
unsafe fn is_taskbar_window(hwnd: HWND) -> bool {
    // The window must be visible
    if IsWindowVisible(hwnd) == 0 {
        return false;
    }

    // The window should not have an owner
    if GetWindow(hwnd, GW_OWNER) != null_mut() {
        return false;
    }

    // Check the window style
    let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

    // The window should have a caption (title bar) and should not be a tool window
    if (style & WS_CAPTION != 0) {
        return true;
    }

    false
}

#[command]
fn activate_window(hwnd: usize) -> bool {
     unsafe {
        let hwnd = hwnd as HWND; // Convert back to HWND

         // Check if the window is maximized
         let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        
        // Restore the window if it is minimized
        if (style & 0x01000000) == 0 { // WS_MAXIMIZE
            winapi::um::winuser::ShowWindow(hwnd, SW_RESTORE);
        } else {
            winapi::um::winuser::ShowWindow(hwnd, SW_SHOW);
        }
        
        // Attempt to bring the window to the foreground
        if winapi::um::winuser::SetForegroundWindow(hwnd) == 0 {
            // Delay and retry
            sleep(Duration::from_millis(100));
            winapi::um::winuser::SetForegroundWindow(hwnd) != 0 // Try again
        } else {
            true // Successfully brought to foreground
        }
    }
}

unsafe fn get_application_name(hwnd: HWND) -> String {
    let mut process_id: DWORD = 0;
    GetWindowThreadProcessId(hwnd, &mut process_id);

    // Open the process with the required permission
    let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, process_id);
    if process_handle.is_null() {
        return String::new();
    }

    // Get the application name
    let mut name_buffer = [0u16; 512];
    let length = GetModuleFileNameExW(process_handle, null_mut(), name_buffer.as_mut_ptr(), name_buffer.len() as u32);

    // Close the process handle
    CloseHandle(process_handle);

    if length > 0 {
        // Convert wide string to Rust String
        let app_path = OsString::from_wide(&name_buffer[..length as usize]).to_string_lossy().into_owned();
        
        // Extract the file name without the extension
        let path = Path::new(&app_path);
        if let Some(file_name) = path.file_stem() {
            return file_name.to_string_lossy().into_owned();
        }
    }

    String::new()
}


fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");

    let tray_menu = SystemTrayMenu::new()
    .add_item(quit)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(hide);

    tauri::Builder::default()
      .system_tray(SystemTray::new().with_menu(tray_menu))
      .invoke_handler(tauri::generate_handler![get_open_window_apps, activate_window])
      .on_system_tray_event(|app, event| match event {
        SystemTrayEvent::LeftClick {
          position: _,
          size: _,
          ..
        } => {
          println!("system tray received a left click");
        }
        SystemTrayEvent::RightClick {
          position: _,
          size: _,
          ..
        } => {
          println!("system tray received a right click");
        }
        SystemTrayEvent::DoubleClick {
          position: _,
          size: _,
          ..
        } => {
          println!("system tray received a double click");
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
          match id.as_str() {
            "quit" => {
              std::process::exit(0);
            }
            "hide" => {
              let window = app.get_window("main").unwrap();
              window.hide().unwrap();
            }
            _ => {}
          }
        }
        _ => {}
      })
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}
