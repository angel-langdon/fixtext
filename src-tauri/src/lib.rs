use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    mem::{size_of, zeroed},
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    ptr::{NonNull, null, null_mut},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{
    AppHandle, Emitter, Manager, State, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use windows_sys::Win32::{
    Foundation::{ERROR_SUCCESS, GlobalFree, HGLOBAL, HWND, LPARAM, LRESULT, WPARAM},
    Networking::WinHttp::{
        INTERNET_DEFAULT_HTTPS_PORT, WINHTTP_ACCESS_TYPE_DEFAULT_PROXY, WINHTTP_FLAG_SECURE,
        WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest,
        WinHttpQueryDataAvailable, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest,
        WinHttpSetTimeouts,
    },
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
        },
        Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
        Registry::{
            HKEY, HKEY_CURRENT_USER, KEY_READ, REG_SZ, RegCloseKey, RegCreateKeyW, RegDeleteValueW,
            RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
        },
        Threading::GetCurrentThreadId,
    },
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, KEYEVENTF_KEYUP, VK_A, VK_C, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN,
            VK_SHIFT, VK_V, keybd_event,
        },
        WindowsAndMessaging::{
            CallNextHookEx, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId,
            KBDLLHOOKSTRUCT, MSG, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
            WH_KEYBOARD_LL, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
        },
    },
};

const APP_NAME: &str = "FixText";
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const GEMINI_HOST: &str = "generativelanguage.googleapis.com";
const GROQ_HOST: &str = "api.groq.com";
const CF_UNICODETEXT: u32 = 13;
const LLKHF_INJECTED: u32 = 0x10;
const MOD_CTRL: u32 = 0x01;
const MOD_ALT: u32 = 0x02;
const MOD_SHIFT: u32 = 0x04;
const MOD_WIN: u32 = 0x08;

static GLOBAL_APP: OnceLock<AppHandle> = OnceLock::new();
static LOG_PATHS: OnceLock<Vec<PathBuf>> = OnceLock::new();
static KEYBOARD_HOOK: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(null_mut());
static KEYBOARD_HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);
static HEALTH_LOGGER_STARTED: AtomicBool = AtomicBool::new(false);
static SHORTCUT_ENABLED: AtomicBool = AtomicBool::new(true);
static SHORTCUT_VK: AtomicU32 = AtomicU32::new(VK_C as u32);
static SHORTCUT_MODS: AtomicU32 = AtomicU32::new(MOD_CTRL);
static SHORTCUT_LAST_TICK: AtomicU64 = AtomicU64::new(0);
static SHORTCUT_COUNT: AtomicU32 = AtomicU32::new(0);
static SHORTCUT_PRESSES: AtomicU32 = AtomicU32::new(2);
static SHORTCUT_WINDOW_MS: AtomicU64 = AtomicU64::new(650);
static SELECT_ALL_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize, Deserialize)]
struct ModelChoice {
    provider: String,
    id: String,
    label: String,
    note: String,
    free_tier: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct PromptProfile {
    id: String,
    title: String,
    instructions: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Settings {
    #[serde(default = "default_provider")]
    selected_provider: String,
    api_key: String,
    #[serde(default)]
    groq_api_key: String,
    selected_model: String,
    custom_model: String,
    selected_profile: String,
    max_output_tokens: u32,
    thinking_level: String,
    temperature: f32,
    auto_paste: bool,
    show_window_on_start: bool,
    #[serde(default = "default_shortcut_enabled")]
    shortcut_enabled: bool,
    #[serde(default = "default_shortcut_key")]
    shortcut_key: String,
    #[serde(default = "default_shortcut_ctrl")]
    shortcut_ctrl: bool,
    #[serde(default)]
    shortcut_alt: bool,
    #[serde(default)]
    shortcut_shift: bool,
    #[serde(default)]
    shortcut_win: bool,
    #[serde(default = "default_shortcut_presses")]
    shortcut_presses: u32,
    #[serde(default = "default_shortcut_window_ms")]
    shortcut_window_ms: u64,
    #[serde(default = "default_status_overlay_enabled")]
    show_status_overlay: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct PersistedState {
    settings: Settings,
    profiles: Vec<PromptProfile>,
}

#[derive(Clone, Serialize)]
struct ActivityItem {
    id: u64,
    kind: String,
    message: String,
    created_at: u64,
}

#[derive(Serialize)]
struct AppViewState {
    settings: Settings,
    models: Vec<ModelChoice>,
    profiles: Vec<PromptProfile>,
    startup_enabled: bool,
    busy: bool,
    activity: Vec<ActivityItem>,
}

#[derive(Serialize)]
struct TransformResult {
    input: String,
    output: String,
    provider: String,
    model: String,
    profile: String,
}

#[derive(Clone, Serialize)]
struct RustStatusEvent {
    stage: String,
    message: String,
    created_at: u64,
}

struct AppState {
    settings: Mutex<Settings>,
    profiles: Mutex<Vec<PromptProfile>>,
    activity: Mutex<Vec<ActivityItem>>,
    busy: AtomicBool,
}

impl AppState {
    fn push_activity(&self, kind: &str, message: impl Into<String>) {
        let Ok(mut activity) = self.activity.lock() else {
            return;
        };
        let id = activity.last().map_or(1, |item| item.id + 1);
        activity.push(ActivityItem {
            id,
            kind: kind.to_owned(),
            message: message.into(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        });
        if activity.len() > 20 {
            let overflow = activity.len() - 20;
            activity.drain(0..overflow);
        }
    }
}

#[tauri::command]
fn load_app_state(_app: AppHandle, state: State<'_, AppState>) -> Result<AppViewState, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "settings lock poisoned".to_owned())?
        .clone();
    let profiles = state
        .profiles
        .lock()
        .map_err(|_| "profiles lock poisoned".to_owned())?
        .clone();
    let activity = state
        .activity
        .lock()
        .map_err(|_| "activity lock poisoned".to_owned())?
        .clone();

    Ok(AppViewState {
        settings,
        models: default_models(),
        profiles,
        startup_enabled: is_startup_enabled()?,
        busy: state.busy.load(Ordering::Relaxed),
        activity,
    })
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
    profiles: Vec<PromptProfile>,
) -> Result<AppViewState, String> {
    validate_settings(&settings)?;
    validate_profiles(&profiles)?;
    sync_shortcut_settings(&settings);
    save_state_file(
        &app,
        &PersistedState {
            settings: settings.clone(),
            profiles: profiles.clone(),
        },
    )?;
    *state
        .settings
        .lock()
        .map_err(|_| "settings lock poisoned".to_owned())? = settings;
    *state
        .profiles
        .lock()
        .map_err(|_| "profiles lock poisoned".to_owned())? = profiles;
    state.push_activity("saved", "Settings saved");
    load_app_state(app, state)
}

#[tauri::command]
fn export_app_state(state: State<'_, AppState>) -> Result<String, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "settings lock poisoned".to_owned())?
        .clone();
    let profiles = state
        .profiles
        .lock()
        .map_err(|_| "profiles lock poisoned".to_owned())?
        .clone();
    serde_json::to_string_pretty(&PersistedState { settings, profiles })
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn import_app_state(
    app: AppHandle,
    state: State<'_, AppState>,
    json: String,
) -> Result<AppViewState, String> {
    let imported: PersistedState = serde_json::from_str(&json).map_err(|err| err.to_string())?;
    validate_settings(&imported.settings)?;
    validate_profiles(&imported.profiles)?;
    sync_shortcut_settings(&imported.settings);
    save_state_file(&app, &imported)?;
    *state
        .settings
        .lock()
        .map_err(|_| "settings lock poisoned".to_owned())? = imported.settings;
    *state
        .profiles
        .lock()
        .map_err(|_| "profiles lock poisoned".to_owned())? = imported.profiles;
    state.push_activity("saved", "Full app state imported");
    load_app_state(app, state)
}

#[tauri::command]
async fn transform_text(
    app: AppHandle,
    input: String,
    profile: PromptProfile,
    provider_id: Option<String>,
    model_id: Option<String>,
) -> Result<TransformResult, String> {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        run_exclusive(&state, || {
            let mut settings = state
                .settings
                .lock()
                .map_err(|_| "settings lock poisoned".to_owned())?
                .clone();
            if let Some(provider_id) = provider_id {
                validate_provider_id(provider_id.trim())?;
                settings.selected_provider = provider_id.trim().to_owned();
            }
            if let Some(model_id) = model_id {
                let model_id = model_id.trim();
                validate_model_id_for_provider(&selected_provider_id(&settings), model_id)?;
                settings.selected_model = model_id.to_owned();
                settings.custom_model.clear();
            }
            emit_status(Some(&app), "prepared", "Prepared draft text");
            let output = generate_text(Some(&app), &settings, &profile.instructions, &input)?;
            state.push_activity("fixed", format!("Transformed {} characters", input.len()));
            Ok(TransformResult {
                input,
                output,
                provider: selected_provider_id(&settings),
                model: selected_model_id(&settings),
                profile: profile.title,
            })
        })
    })
    .await
    .map_err(|err| err.to_string())?
}

#[tauri::command]
async fn fix_clipboard(app: AppHandle) -> Result<TransformResult, String> {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        run_exclusive(&state, || fix_clipboard_inner(&state, Some(&app)))
    })
    .await
    .map_err(|err| err.to_string())?
}

#[tauri::command]
fn read_clipboard_text() -> Result<String, String> {
    read_clipboard()
}

#[tauri::command]
fn write_clipboard_text(text: String) -> Result<(), String> {
    write_clipboard(&text)
}

#[tauri::command]
fn set_startup_enabled(enabled: bool) -> Result<bool, String> {
    if enabled {
        enable_startup()?;
    } else {
        disable_startup()?;
    }
    is_startup_enabled()
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    show_window(&app)
}

#[tauri::command]
fn hide_main_window(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_owned())?;
    window.hide().map_err(|err| err.to_string())
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    log_event("quit requested from UI");
    uninstall_keyboard_hook();
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    install_panic_logger();
    tauri::Builder::default()
        .setup(|app| {
            init_log_path(app.handle());
            log_event("startup");
            let persisted = load_state_file(app.handle()).unwrap_or_else(|_| {
                let mut settings = default_settings();
                if let Some(api_key) = load_api_key_from_env_or_file() {
                    settings.api_key = api_key;
                }
                PersistedState {
                    settings,
                    profiles: default_profiles(),
                }
            });
            log_event("state loaded");
            sync_shortcut_settings(&persisted.settings);

            app.manage(AppState {
                settings: Mutex::new(persisted.settings.clone()),
                profiles: Mutex::new(persisted.profiles),
                activity: Mutex::new(Vec::new()),
                busy: AtomicBool::new(false),
            });

            install_tray(app.handle())?;
            log_event("tray installed");
            install_keyboard_hook(app.handle().clone());
            start_health_logger();
            if !persisted.settings.show_window_on_start
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.hide();
            }
            log_event("setup complete");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            load_app_state,
            save_settings,
            export_app_state,
            import_app_state,
            transform_text,
            fix_clipboard,
            read_clipboard_text,
            write_clipboard_text,
            set_startup_enabled,
            show_main_window,
            hide_main_window,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn run_exclusive<T>(
    state: &State<'_, AppState>,
    work: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    if state.busy.swap(true, Ordering::Acquire) {
        return Err("FixText is already working".to_owned());
    }
    let result = catch_unwind(AssertUnwindSafe(work));
    state.busy.store(false, Ordering::Release);
    match result {
        Ok(result) => result,
        Err(_) => Err("FixText hit an internal error while working".to_owned()),
    }
}

fn install_panic_logger() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
            .unwrap_or_else(|| "unknown location".to_owned());
        log_event(format!("panic at {location}: {info}"));
    }));
}

fn init_log_path(app: &AppHandle) {
    let mut paths = Vec::new();
    if let Ok(dir) = app.path().app_config_dir() {
        let _ = fs::create_dir_all(&dir);
        paths.push(dir.join("fixtext.log"));
    }
    for path in project_error_log_paths() {
        if !paths.iter().any(|existing| existing == &path) {
            paths.push(path);
        }
    }
    let _ = LOG_PATHS.set(paths);
    log_event("logging initialized");
}

fn log_event(message: impl AsRef<str>) {
    let Some(paths) = LOG_PATHS.get() else {
        return;
    };
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    let process_id = std::process::id();
    let thread_id = format!("{:?}", thread::current().id());
    for path in paths {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(
                file,
                "{timestamp} pid={process_id} tid={thread_id} {}",
                message.as_ref()
            );
        }
    }
}

fn project_error_log_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(current_dir) = std::env::current_dir()
        && let Some(path) = project_error_log_path_from(&current_dir)
    {
        paths.push(path);
    }
    if let Ok(current_exe) = std::env::current_exe() {
        for ancestor in current_exe.ancestors() {
            if let Some(path) = project_error_log_path_from(ancestor)
                && !paths.iter().any(|existing| existing == &path)
            {
                paths.push(path);
            }
        }
    }
    paths
}

fn project_error_log_path_from(path: &Path) -> Option<PathBuf> {
    if path.join("package.json").is_file() && path.join("src-tauri").is_dir() {
        Some(path.join("error.log"))
    } else {
        None
    }
}

fn start_health_logger() {
    if HEALTH_LOGGER_STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    match thread::Builder::new()
        .name("fixtext-health-log".to_owned())
        .spawn(|| {
            loop {
                thread::sleep(Duration::from_secs(60));
                let hook_thread_id = KEYBOARD_HOOK_THREAD_ID.load(Ordering::Relaxed);
                let hook_installed = !KEYBOARD_HOOK.load(Ordering::Relaxed).is_null();
                log_event(format!(
                    "heartbeat hook_thread_id={hook_thread_id} hook_installed={hook_installed}"
                ));
            }
        }) {
        Ok(_) => log_event("health logger started"),
        Err(err) => {
            HEALTH_LOGGER_STARTED.store(false, Ordering::Release);
            log_event(format!("health logger failed: {err}"));
        }
    }
}

fn install_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open FixText", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit FixText", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip(APP_NAME)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                let _ = show_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                let _ = show_window(app);
            }
            "quit" => {
                log_event("quit requested from tray");
                uninstall_keyboard_hook();
                app.exit(0);
            }
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn install_keyboard_hook(app: AppHandle) {
    let _ = GLOBAL_APP.set(app.clone());
    if KEYBOARD_HOOK_THREAD_ID.load(Ordering::Acquire) != 0 {
        log_event("keyboard hook thread already running");
        return;
    }

    let state_app = app.clone();
    let state = state_app.state::<AppState>();
    match thread::Builder::new()
        .name("fixtext-keyboard-hook".to_owned())
        .spawn(move || run_keyboard_hook_thread(app))
    {
        Ok(_) => {
            log_event("keyboard hook thread started");
            state.push_activity("saved", "Keyboard shortcut hook thread started");
        }
        Err(err) => {
            log_event(format!("keyboard hook thread failed: {err}"));
            state.push_activity("error", "Keyboard shortcut hook thread failed");
        }
    }
}

fn uninstall_keyboard_hook() {
    let thread_id = KEYBOARD_HOOK_THREAD_ID.load(Ordering::Acquire);
    if thread_id != 0 {
        unsafe {
            let _ = PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
        }
        log_event("keyboard hook stop requested");
    }
}

fn run_keyboard_hook_thread(app: AppHandle) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let thread_id = unsafe { GetCurrentThreadId() };
        KEYBOARD_HOOK_THREAD_ID.store(thread_id, Ordering::Release);
        log_event(format!(
            "keyboard hook thread message loop ready id={thread_id}"
        ));

        // Low-level hooks are delivered through the installing thread's message queue.
        let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), null_mut(), 0) };
        let state = app.state::<AppState>();
        if hook.is_null() {
            KEYBOARD_HOOK_THREAD_ID.store(0, Ordering::Release);
            log_event("keyboard hook failed");
            state.push_activity("error", "Keyboard shortcut hook failed");
            return;
        }

        KEYBOARD_HOOK.store(hook, Ordering::Release);
        log_event("keyboard hook installed");
        state.push_activity("saved", "Keyboard shortcut hook installed");

        let mut msg: MSG = unsafe { zeroed() };
        loop {
            let status = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
            if status <= 0 {
                break;
            }
        }
    }));

    if result.is_err() {
        log_event("keyboard hook thread panicked");
    }

    let hook = KEYBOARD_HOOK.swap(null_mut(), Ordering::AcqRel);
    if let Some(hook) = NonNull::new(hook) {
        unsafe {
            UnhookWindowsHookEx(hook.as_ptr());
        }
        log_event("keyboard hook uninstalled");
    }
    KEYBOARD_HOOK_THREAD_ID.store(0, Ordering::Release);
    log_event("keyboard hook thread exited");
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && (wparam as u32 == WM_KEYDOWN || wparam as u32 == WM_SYSKEYDOWN) {
        let event = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
        if event.flags & LLKHF_INJECTED == 0 {
            let _ = catch_unwind(AssertUnwindSafe(|| handle_shortcut_key(event.vkCode)));
        }
    }
    unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
}

fn handle_shortcut_key(vk_code: u32) {
    let Some(app) = GLOBAL_APP.get() else {
        return;
    };
    if !SHORTCUT_ENABLED.load(Ordering::Relaxed) {
        return;
    };
    if handle_select_all_shortcut(app, vk_code) {
        return;
    }
    if SHORTCUT_VK.load(Ordering::Relaxed) != vk_code {
        return;
    }
    if !shortcut_modifiers_match(SHORTCUT_MODS.load(Ordering::Relaxed)) {
        return;
    }
    if own_window_is_foreground() {
        return;
    }

    let now = monotonic_millis();
    let last = SHORTCUT_LAST_TICK.load(Ordering::Relaxed);
    let count =
        if last != 0 && now.saturating_sub(last) <= SHORTCUT_WINDOW_MS.load(Ordering::Relaxed) {
            SHORTCUT_COUNT.fetch_add(1, Ordering::Relaxed) + 1
        } else {
            SHORTCUT_COUNT.store(1, Ordering::Relaxed);
            1
        };
    SHORTCUT_LAST_TICK.store(now, Ordering::Relaxed);

    let needed = SHORTCUT_PRESSES.load(Ordering::Relaxed).clamp(1, 4);
    if count == needed {
        SHORTCUT_COUNT.store(0, Ordering::Relaxed);
        log_event("clipboard shortcut triggered");
        let app = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            std::thread::sleep(Duration::from_millis(90));
            let state = app.state::<AppState>();
            match run_exclusive(&state, || fix_clipboard_inner(&state, Some(&app))) {
                Ok(_) => state.push_activity("fixed", "Clipboard fixed from shortcut"),
                Err(err) => state.push_activity("error", err),
            }
        });
    }
}

fn handle_select_all_shortcut(app: &AppHandle, vk_code: u32) -> bool {
    if vk_code != VK_C as u32
        || !key_down(VK_CONTROL as i32)
        || !key_down(VK_MENU as i32)
        || key_down(VK_SHIFT as i32)
        || key_down(VK_LWIN as i32)
        || key_down(VK_RWIN as i32)
    {
        return false;
    }
    if own_window_is_foreground() {
        return true;
    }
    if SELECT_ALL_IN_FLIGHT.swap(true, Ordering::AcqRel) {
        log_event("selection shortcut ignored because one is already running");
        return true;
    }

    let app = app.clone();
    log_event("selection shortcut triggered");
    tauri::async_runtime::spawn_blocking(move || {
        wait_for_shortcut_release();
        let state = app.state::<AppState>();
        match run_exclusive(&state, || fix_selection_inner(&state, Some(&app))) {
            Ok(_) => state.push_activity("fixed", "Selection fixed from Ctrl+Alt+C"),
            Err(err) => state.push_activity("error", err),
        }
        SELECT_ALL_IN_FLIGHT.store(false, Ordering::Release);
    });
    true
}

fn own_window_is_foreground() -> bool {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.is_null() {
            return false;
        }
        let mut process_id = 0;
        GetWindowThreadProcessId(foreground, &mut process_id);
        process_id != 0 && process_id == std::process::id()
    }
}

fn shortcut_modifiers_match(mods: u32) -> bool {
    ((mods & MOD_CTRL) != 0) == key_down(VK_CONTROL as i32)
        && ((mods & MOD_ALT) != 0) == key_down(VK_MENU as i32)
        && ((mods & MOD_SHIFT) != 0) == key_down(VK_SHIFT as i32)
        && ((mods & MOD_WIN) != 0) == (key_down(VK_LWIN as i32) || key_down(VK_RWIN as i32))
}

fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) & i16::MIN) != 0 }
}

fn monotonic_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

fn wait_for_shortcut_release() {
    for _ in 0..35 {
        if !key_down(VK_CONTROL as i32) && !key_down(VK_MENU as i32) && !key_down(VK_C as i32) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn copy_all_text_to_clipboard() {
    key_combo(VK_CONTROL as u8, VK_A as u8);
    std::thread::sleep(Duration::from_millis(90));
    key_combo(VK_CONTROL as u8, VK_C as u8);
    std::thread::sleep(Duration::from_millis(180));
}

fn key_combo(modifier: u8, key: u8) {
    unsafe {
        keybd_event(modifier, 0, 0, 0);
        keybd_event(key, 0, 0, 0);
        keybd_event(key, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(modifier, 0, KEYEVENTF_KEYUP, 0);
    }
}

fn show_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_owned())?;
    window.show().map_err(|err| err.to_string())?;
    window.unminimize().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
    Ok(())
}

fn fix_clipboard_inner(
    state: &State<'_, AppState>,
    app: Option<&AppHandle>,
) -> Result<TransformResult, String> {
    fix_clipboard_inner_with_paste(state, app, false)
}

fn fix_selection_inner(
    state: &State<'_, AppState>,
    app: Option<&AppHandle>,
) -> Result<TransformResult, String> {
    emit_status(app, "selecting", "Selecting text");
    copy_all_text_to_clipboard();
    fix_clipboard_inner_with_paste(state, app, true)
}

fn fix_clipboard_inner_with_paste(
    state: &State<'_, AppState>,
    app: Option<&AppHandle>,
    force_paste: bool,
) -> Result<TransformResult, String> {
    log_event("fix clipboard started");
    let settings = state
        .settings
        .lock()
        .map_err(|_| "settings lock poisoned".to_owned())?
        .clone();
    let profiles = state
        .profiles
        .lock()
        .map_err(|_| "profiles lock poisoned".to_owned())?
        .clone();
    let profile = profiles
        .iter()
        .find(|profile| profile.id == settings.selected_profile)
        .or_else(|| profiles.first())
        .ok_or_else(|| "no prompt profiles available".to_owned())?;
    let input = read_clipboard()?;
    emit_status(app, "copied", "Copied clipboard text");
    if input.trim().is_empty() {
        return Err("Clipboard is empty".to_owned());
    }
    let output = generate_text(app, &settings, &profile.instructions, &input)?;
    write_clipboard(&output)?;
    if settings.auto_paste || force_paste {
        emit_status(app, "pasting", "Pasting result");
        paste_clipboard();
    }
    emit_status(app, "done", "Clipboard updated");
    log_event("fix clipboard finished");
    state.push_activity("fixed", "Clipboard fixed");
    Ok(TransformResult {
        input,
        output,
        provider: selected_provider_id(&settings),
        model: selected_model_id(&settings),
        profile: profile.title.clone(),
    })
}

fn validate_settings(settings: &Settings) -> Result<(), String> {
    if settings.max_output_tokens == 0 || settings.max_output_tokens > 4096 {
        return Err("Max output tokens must be between 1 and 4096".to_owned());
    }
    if !(0.0..=2.0).contains(&settings.temperature) {
        return Err("Temperature must be between 0 and 2".to_owned());
    }
    if shortcut_vk(&settings.shortcut_key).is_none() {
        return Err("Shortcut key must be A-Z, 0-9, or F1-F12".to_owned());
    }
    if settings.shortcut_presses == 0 || settings.shortcut_presses > 4 {
        return Err("Shortcut press count must be between 1 and 4".to_owned());
    }
    if settings.shortcut_window_ms < 150 || settings.shortcut_window_ms > 2000 {
        return Err("Shortcut window must be between 150 and 2000 ms".to_owned());
    }
    let provider = selected_provider_id(settings);
    validate_provider_id(&provider)?;
    validate_model_id_for_provider(&provider, &selected_model_id(settings))
}

fn validate_profiles(profiles: &[PromptProfile]) -> Result<(), String> {
    if profiles.is_empty() {
        return Err("At least one prompt profile is required".to_owned());
    }
    for profile in profiles {
        if profile.id.trim().is_empty() || profile.title.trim().is_empty() {
            return Err("Prompt profiles need an id and title".to_owned());
        }
        if profile.instructions.trim().is_empty() {
            return Err(format!(
                "Prompt profile '{}' has empty instructions",
                profile.title
            ));
        }
    }
    Ok(())
}

fn validate_model_id_for_provider(provider: &str, model: &str) -> Result<(), String> {
    if model.is_empty()
        || model
            .chars()
            .any(|ch| ch.is_whitespace() || matches!(ch, '?' | '&' | '#'))
    {
        return Err(format!("{provider} model id is invalid"));
    }
    if provider == "gemini" && model.contains('/') {
        return Err("Gemini model id must not include a provider prefix or slash".to_owned());
    }
    Ok(())
}

fn validate_provider_id(provider: &str) -> Result<(), String> {
    match provider {
        "gemini" | "groq" => Ok(()),
        _ => Err("Provider must be gemini or groq".to_owned()),
    }
}

fn sync_shortcut_settings(settings: &Settings) {
    let mut mods = 0;
    if settings.shortcut_ctrl {
        mods |= MOD_CTRL;
    }
    if settings.shortcut_alt {
        mods |= MOD_ALT;
    }
    if settings.shortcut_shift {
        mods |= MOD_SHIFT;
    }
    if settings.shortcut_win {
        mods |= MOD_WIN;
    }
    SHORTCUT_ENABLED.store(settings.shortcut_enabled, Ordering::Release);
    SHORTCUT_VK.store(
        shortcut_vk(&settings.shortcut_key).unwrap_or(VK_C as u32),
        Ordering::Release,
    );
    SHORTCUT_MODS.store(mods, Ordering::Release);
    SHORTCUT_PRESSES.store(settings.shortcut_presses.clamp(1, 4), Ordering::Release);
    SHORTCUT_WINDOW_MS.store(
        settings.shortcut_window_ms.clamp(150, 2000),
        Ordering::Release,
    );
    SHORTCUT_LAST_TICK.store(0, Ordering::Release);
    SHORTCUT_COUNT.store(0, Ordering::Release);
}

fn selected_provider_id(settings: &Settings) -> String {
    if settings.selected_provider == "groq" {
        "groq".to_owned()
    } else {
        "gemini".to_owned()
    }
}

fn selected_model_id(settings: &Settings) -> String {
    if settings.selected_model == "custom" {
        settings.custom_model.trim().to_owned()
    } else {
        settings.selected_model.clone()
    }
}

fn groq_reasoning_effort(model: &str) -> Option<&'static str> {
    if model.starts_with("openai/gpt-oss-") {
        Some("low")
    } else if model.starts_with("qwen/qwen3-") {
        Some("none")
    } else {
        None
    }
}

fn shortcut_vk(key: &str) -> Option<u32> {
    let key = key.trim().to_ascii_uppercase();
    let bytes = key.as_bytes();
    if bytes.len() == 1 {
        let ch = bytes[0];
        if ch.is_ascii_uppercase() || ch.is_ascii_digit() {
            return Some(ch as u32);
        }
    }
    if let Some(number) = key
        .strip_prefix('F')
        .and_then(|value| value.parse::<u32>().ok())
        && (1..=12).contains(&number)
    {
        return Some(0x70 + number - 1);
    }
    None
}

fn default_models() -> Vec<ModelChoice> {
    vec![
        ModelChoice {
            provider: "gemini".to_owned(),
            id: "gemini-3.1-flash-lite".to_owned(),
            label: "Gemini 3.1 Flash-Lite".to_owned(),
            note: "Current cost-efficient free-tier default".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "gemini".to_owned(),
            id: "gemini-3.1-flash-lite-preview".to_owned(),
            label: "Gemini 3.1 Flash-Lite Preview".to_owned(),
            note: "Preview free-tier Flash-Lite option".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "gemini".to_owned(),
            id: "gemini-3-flash-preview".to_owned(),
            label: "Gemini 3 Flash Preview".to_owned(),
            note: "Higher-capability free-tier Flash preview".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "gemini".to_owned(),
            id: "gemini-2.5-flash".to_owned(),
            label: "Gemini 2.5 Flash".to_owned(),
            note: "Free-tier low-latency rewrite model".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "gemini".to_owned(),
            id: "gemini-2.5-flash-lite".to_owned(),
            label: "Gemini 2.5 Flash-Lite".to_owned(),
            note: "Stable 2.5 free-tier fallback".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "groq".to_owned(),
            id: "openai/gpt-oss-120b".to_owned(),
            label: "Groq GPT-OSS 120B".to_owned(),
            note: "GroqCloud text model, developer/free-tier rate limits".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "groq".to_owned(),
            id: "openai/gpt-oss-20b".to_owned(),
            label: "Groq GPT-OSS 20B".to_owned(),
            note: "Very fast GroqCloud text model".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "groq".to_owned(),
            id: "llama-3.3-70b-versatile".to_owned(),
            label: "Groq Llama 3.3 70B".to_owned(),
            note: "General-purpose GroqCloud production model".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "groq".to_owned(),
            id: "llama-3.1-8b-instant".to_owned(),
            label: "Groq Llama 3.1 8B Instant".to_owned(),
            note: "Lowest-latency GroqCloud production model".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "groq".to_owned(),
            id: "meta-llama/llama-4-scout-17b-16e-instruct".to_owned(),
            label: "Groq Llama 4 Scout".to_owned(),
            note: "Preview text/vision-capable model; useful for latency testing".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "groq".to_owned(),
            id: "qwen/qwen3-32b".to_owned(),
            label: "Groq Qwen3 32B".to_owned(),
            note: "Preview reasoning-capable text model".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "groq".to_owned(),
            id: "groq/compound-mini".to_owned(),
            label: "Groq Compound Mini".to_owned(),
            note: "Groq system model with tool-capable routing".to_owned(),
            free_tier: true,
        },
        ModelChoice {
            provider: "groq".to_owned(),
            id: "groq/compound".to_owned(),
            label: "Groq Compound".to_owned(),
            note: "Groq system model with web/code tooling support".to_owned(),
            free_tier: true,
        },
    ]
}

fn default_profiles() -> Vec<PromptProfile> {
    let common = "Corrige el texto de forma minima.\nArregla solo errores claros de ortografia, gramatica, puntuacion, concordancia u orden.\nConserva las palabras, estructura, tono e intencion originales siempre que sea posible.\nSi ya esta bien, devuelve practicamente lo mismo.\nMismo idioma. Espanol: Castellano de Espana.\nSin explicaciones, comentarios ni comillas.";
    vec![
        PromptProfile {
            id: "fix".to_owned(),
            title: "Fix".to_owned(),
            instructions: common.to_owned(),
        },
        PromptProfile {
            id: "formal".to_owned(),
            title: "Formal".to_owned(),
            instructions: format!("{common}\n- Haz que el mensaje sea formal, manteniendo el estilo original."),
        },
        PromptProfile {
            id: "markdown".to_owned(),
            title: "Markdown".to_owned(),
            instructions: "Fix any errors in the Markdown formatting of the text.\nAdd proper headings, lists, and formatting to make it more readable and structured.\nOnly return the modified Markdown text without explanations.".to_owned(),
        },
        PromptProfile {
            id: "english".to_owned(),
            title: "Formal English".to_owned(),
            instructions: format!("{common}\n- Answer in English with a formal but natural tone."),
        },
    ]
}

fn default_settings() -> Settings {
    Settings {
        selected_provider: default_provider(),
        api_key: String::new(),
        groq_api_key: String::new(),
        selected_model: "gemini-3.1-flash-lite".to_owned(),
        custom_model: String::new(),
        selected_profile: "fix".to_owned(),
        max_output_tokens: 512,
        thinking_level: "minimal".to_owned(),
        temperature: 0.2,
        auto_paste: true,
        show_window_on_start: true,
        shortcut_enabled: default_shortcut_enabled(),
        shortcut_key: default_shortcut_key(),
        shortcut_ctrl: default_shortcut_ctrl(),
        shortcut_alt: false,
        shortcut_shift: false,
        shortcut_win: false,
        shortcut_presses: default_shortcut_presses(),
        shortcut_window_ms: default_shortcut_window_ms(),
        show_status_overlay: default_status_overlay_enabled(),
    }
}

fn default_provider() -> String {
    "gemini".to_owned()
}

fn default_status_overlay_enabled() -> bool {
    true
}

fn default_shortcut_enabled() -> bool {
    true
}

fn default_shortcut_key() -> String {
    "C".to_owned()
}

fn default_shortcut_ctrl() -> bool {
    true
}

fn default_shortcut_presses() -> u32 {
    2
}

fn default_shortcut_window_ms() -> u64 {
    650
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|err| err.to_string())?;
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    Ok(dir.join("settings.json"))
}

fn load_state_file(app: &AppHandle) -> Result<PersistedState, String> {
    let path = settings_path(app)?;
    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    serde_json::from_slice(&bytes).map_err(|err| err.to_string())
}

fn save_state_file(app: &AppHandle, persisted: &PersistedState) -> Result<(), String> {
    let path = settings_path(app)?;
    let bytes = serde_json::to_vec_pretty(persisted).map_err(|err| err.to_string())?;
    fs::write(path, bytes).map_err(|err| err.to_string())
}

fn load_api_key_from_env_or_file() -> Option<String> {
    if let Ok(value) = std::env::var("GEMINI_API_KEY")
        && !value.trim().is_empty()
    {
        return Some(value);
    }
    let env_paths = [
        std::env::current_dir().ok().map(|path| path.join(".env")),
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|dir| dir.join(".env"))),
    ];
    for path in env_paths.into_iter().flatten() {
        let Ok(file) = fs::read_to_string(path) else {
            continue;
        };
        for line in file.lines() {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("GEMINI_API_KEY=") {
                let value = value.trim_matches(['"', '\'']).trim();
                if !value.is_empty() {
                    return Some(value.to_owned());
                }
            }
        }
    }
    None
}

fn generate_text(
    app: Option<&AppHandle>,
    settings: &Settings,
    instructions: &str,
    input: &str,
) -> Result<String, String> {
    let provider = selected_provider_id(settings);
    log_event(format!(
        "generate_text provider={provider} model={} input_chars={}",
        selected_model_id(settings),
        input.len()
    ));
    if provider == "groq" {
        if settings.groq_api_key.trim().is_empty() {
            return Err("Groq API key is missing".to_owned());
        }
    } else if settings.api_key.trim().is_empty() {
        return Err("Gemini API key is missing".to_owned());
    }
    emit_status(app, "api-key", format!("{provider} API key loaded"));
    let model = selected_model_id(settings);
    validate_model_id_for_provider(&provider, &model)?;
    if provider == "groq" {
        let mut body = serde_json::json!({
            "model": model,
            "messages": [
                { "role": "system", "content": instructions },
                { "role": "user", "content": input }
            ],
            "temperature": settings.temperature,
            "max_completion_tokens": settings.max_output_tokens,
            "stream": false
        });
        if let Some(reasoning_effort) = groq_reasoning_effort(&model)
            && let Some(body) = body.as_object_mut()
        {
            body.insert("include_reasoning".to_owned(), serde_json::json!(false));
            body.insert(
                "reasoning_effort".to_owned(),
                serde_json::json!(reasoning_effort),
            );
        }
        emit_status(app, "sent", "Sent request to Groq");
        let response = post_groq(&settings.groq_api_key, &body.to_string())?;
        emit_status(app, "received", "Received Groq response");
        return extract_groq_text(&response);
    }
    let mut generation_config = serde_json::json!({
        "responseMimeType": "text/plain",
        "maxOutputTokens": settings.max_output_tokens,
        "temperature": settings.temperature,
    });
    if let Some(config) = generation_config.as_object_mut() {
        if model.starts_with("gemini-3") {
            config.insert(
                "thinkingConfig".to_owned(),
                serde_json::json!({ "thinkingLevel": settings.thinking_level }),
            );
        } else if model.starts_with("gemini-2.5-flash") {
            config.insert(
                "thinkingConfig".to_owned(),
                serde_json::json!({ "thinkingBudget": 0 }),
            );
        }
    }
    let body = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{ "text": input }]
        }],
        "systemInstruction": {
            "parts": [{ "text": instructions }]
        },
        "generationConfig": generation_config
    });
    emit_status(app, "sent", "Sent request to Gemini");
    let response = post_gemini(&settings.api_key, &model, &body.to_string())?;
    emit_status(app, "received", "Received Gemini response");
    extract_gemini_text(&response)
}

fn emit_status(app: Option<&AppHandle>, stage: &str, message: impl Into<String>) {
    if let Some(app) = app {
        let _ = app.emit(
            "rust-status",
            RustStatusEvent {
                stage: stage.to_owned(),
                message: message.into(),
                created_at: monotonic_millis(),
            },
        );
    }
}

fn extract_gemini_text(response: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(response).map_err(|err| err.to_string())?;
    if let Some(message) = value
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
    {
        return Err(message.to_owned());
    }
    let mut out = String::new();
    if let Some(parts) = value
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|candidate| candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(Value::as_array)
    {
        for part in parts {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                out.push_str(text);
            }
        }
    }
    if out.trim().is_empty() {
        Err("Gemini returned no text".to_owned())
    } else {
        Ok(out)
    }
}

fn extract_groq_text(response: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(response).map_err(|err| err.to_string())?;
    if let Some(message) = value
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
    {
        return Err(message.to_owned());
    }
    let Some(text) = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
    else {
        return Err("Groq returned no text".to_owned());
    };
    if text.trim().is_empty() {
        Err("Groq returned empty text".to_owned())
    } else {
        Ok(text.to_owned())
    }
}

fn post_gemini(api_key: &str, model: &str, body: &str) -> Result<String, String> {
    post_json(
        GEMINI_HOST,
        &format!("/v1beta/models/{model}:generateContent?key={api_key}"),
        "Content-Type: application/json\r\nAccept: application/json\r\n",
        body,
        "Gemini",
    )
}

fn post_groq(api_key: &str, body: &str) -> Result<String, String> {
    post_json(
        GROQ_HOST,
        "/openai/v1/chat/completions",
        &format!(
            "Content-Type: application/json\r\nAccept: application/json\r\nAuthorization: Bearer {}\r\n",
            api_key.trim()
        ),
        body,
        "Groq",
    )
}

fn post_json(
    host: &str,
    path: &str,
    headers: &str,
    body: &str,
    service: &str,
) -> Result<String, String> {
    let agent = wide_null("FixText/0.1");
    let host = wide_null(host);
    let verb = wide_null("POST");
    let path = wide_null(path);
    let headers = wide_null(headers);
    let mut response = Vec::<u8>::new();

    // WinHTTP is a C API. All handles are closed on every early return below.
    unsafe {
        let session = WinHttpOpen(
            agent.as_ptr(),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            null(),
            null(),
            0,
        );
        if session.is_null() {
            return Err("WinHttpOpen failed".to_owned());
        }
        WinHttpSetTimeouts(session, 15_000, 15_000, 30_000, 60_000);
        let connect = WinHttpConnect(session, host.as_ptr(), INTERNET_DEFAULT_HTTPS_PORT, 0);
        if connect.is_null() {
            WinHttpCloseHandle(session);
            return Err("WinHttpConnect failed".to_owned());
        }
        let request = WinHttpOpenRequest(
            connect,
            verb.as_ptr(),
            path.as_ptr(),
            null(),
            null(),
            null(),
            WINHTTP_FLAG_SECURE,
        );
        if request.is_null() {
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return Err("WinHttpOpenRequest failed".to_owned());
        }
        let ok = WinHttpSendRequest(
            request,
            headers.as_ptr(),
            u32::MAX,
            body.as_ptr() as _,
            body.len() as u32,
            body.len() as u32,
            0,
        );
        if ok == 0 || WinHttpReceiveResponse(request, null_mut()) == 0 {
            WinHttpCloseHandle(request);
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            log_event(format!("{service} request failed"));
            return Err(format!("{service} request failed"));
        }
        loop {
            let mut available = 0u32;
            if WinHttpQueryDataAvailable(request, &mut available) == 0 {
                WinHttpCloseHandle(request);
                WinHttpCloseHandle(connect);
                WinHttpCloseHandle(session);
                log_event("WinHttpQueryDataAvailable failed");
                return Err("WinHttpQueryDataAvailable failed".to_owned());
            }
            if available == 0 {
                break;
            }
            let start = response.len();
            response.resize(start + available as usize, 0);
            let mut read = 0u32;
            if WinHttpReadData(
                request,
                response[start..].as_mut_ptr() as _,
                available,
                &mut read,
            ) == 0
            {
                WinHttpCloseHandle(request);
                WinHttpCloseHandle(connect);
                WinHttpCloseHandle(session);
                log_event("WinHttpReadData failed");
                return Err("WinHttpReadData failed".to_owned());
            }
            response.truncate(start + read as usize);
        }
        WinHttpCloseHandle(request);
        WinHttpCloseHandle(connect);
        WinHttpCloseHandle(session);
    }

    String::from_utf8(response).map_err(|err| err.to_string())
}

fn read_clipboard() -> Result<String, String> {
    // Clipboard ownership and global-memory access are OS contracts; this block copies the text out.
    let text = unsafe {
        open_clipboard_with_retry()?;
        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            CloseClipboard();
            return Err("Clipboard does not contain Unicode text".to_owned());
        }
        let byte_len = GlobalSize(handle as HGLOBAL);
        if byte_len < size_of::<u16>() {
            CloseClipboard();
            return Err("Clipboard text size is unavailable".to_owned());
        }
        let locked = GlobalLock(handle as HGLOBAL) as *const u16;
        if locked.is_null() {
            CloseClipboard();
            return Err("Could not lock clipboard memory".to_owned());
        }
        let max_units = byte_len / size_of::<u16>();
        let slice = std::slice::from_raw_parts(locked, max_units);
        let len = slice
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(max_units);
        let copy = slice[..len].to_vec();
        GlobalUnlock(handle as HGLOBAL);
        CloseClipboard();
        copy
    };
    String::from_utf16(&text).map_err(|err| err.to_string())
}

fn write_clipboard(text: &str) -> Result<(), String> {
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);
    let byte_len = wide.len() * size_of::<u16>();

    // Windows takes ownership of the HGLOBAL after SetClipboardData succeeds.
    unsafe {
        let mem = GlobalAlloc(GMEM_MOVEABLE, byte_len);
        if mem.is_null() {
            return Err("GlobalAlloc failed".to_owned());
        }
        let locked = GlobalLock(mem) as *mut u16;
        if locked.is_null() {
            GlobalFree(mem);
            return Err("GlobalLock failed".to_owned());
        }
        locked.copy_from_nonoverlapping(wide.as_ptr(), wide.len());
        GlobalUnlock(mem);

        if let Err(err) = open_clipboard_with_retry() {
            GlobalFree(mem);
            return Err(err);
        }
        if EmptyClipboard() == 0 {
            CloseClipboard();
            GlobalFree(mem);
            return Err("Could not empty clipboard".to_owned());
        }
        if SetClipboardData(CF_UNICODETEXT, mem as _).is_null() {
            CloseClipboard();
            GlobalFree(mem);
            return Err("SetClipboardData failed".to_owned());
        }
        CloseClipboard();
    }
    Ok(())
}

unsafe fn open_clipboard_with_retry() -> Result<(), String> {
    for attempt in 0..12 {
        if unsafe { OpenClipboard(null_mut::<std::ffi::c_void>() as HWND) } != 0 {
            return Ok(());
        }
        if attempt < 11 {
            thread::sleep(Duration::from_millis(25));
        }
    }
    Err("Could not open clipboard".to_owned())
}

fn paste_clipboard() {
    unsafe {
        keybd_event(VK_CONTROL as u8, 0, 0, 0);
        keybd_event(VK_V as u8, 0, 0, 0);
        keybd_event(VK_V as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}

fn is_startup_enabled() -> Result<bool, String> {
    unsafe {
        let mut key: HKEY = null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide_null(RUN_KEY).as_ptr(),
            0,
            KEY_READ,
            &mut key,
        ) != ERROR_SUCCESS
        {
            return Ok(false);
        }
        let mut size = 0u32;
        let status = RegQueryValueExW(
            key,
            wide_null(APP_NAME).as_ptr(),
            null_mut(),
            null_mut(),
            null_mut(),
            &mut size,
        );
        RegCloseKey(key);
        Ok(status == ERROR_SUCCESS)
    }
}

fn enable_startup() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let command = format!("\"{}\"", exe.display());
    let value = wide_null(&command);
    unsafe {
        let mut key: HKEY = null_mut();
        let status = RegCreateKeyW(HKEY_CURRENT_USER, wide_null(RUN_KEY).as_ptr(), &mut key);
        if status != ERROR_SUCCESS {
            return Err("Could not open Windows startup registry key".to_owned());
        }
        let result = RegSetValueExW(
            key,
            wide_null(APP_NAME).as_ptr(),
            0,
            REG_SZ,
            value.as_ptr() as *const u8,
            (value.len() * size_of::<u16>()) as u32,
        );
        RegCloseKey(key);
        if result == ERROR_SUCCESS {
            Ok(())
        } else {
            Err("Could not write startup registry value".to_owned())
        }
    }
}

fn disable_startup() -> Result<(), String> {
    unsafe {
        let mut key: HKEY = null_mut();
        let status = RegCreateKeyW(HKEY_CURRENT_USER, wide_null(RUN_KEY).as_ptr(), &mut key);
        if status != ERROR_SUCCESS {
            return Err("Could not open Windows startup registry key".to_owned());
        }
        let _result = RegDeleteValueW(key, wide_null(APP_NAME).as_ptr());
        RegCloseKey(key);
        Ok(())
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
