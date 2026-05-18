const std = @import("std");

const Allocator = std.mem.Allocator;
const L = std.unicode.utf8ToUtf16LeStringLiteral;

const HWND = ?*anyopaque;
const HINSTANCE = ?*anyopaque;
const HICON = ?*anyopaque;
const HCURSOR = ?*anyopaque;
const HBRUSH = ?*anyopaque;
const HMENU = ?*anyopaque;
const HGLOBAL = ?*anyopaque;
const HKEY = ?*anyopaque;
const HINTERNET = ?*anyopaque;
const HANDLE = ?*anyopaque;
const HHOOK = ?*anyopaque;
const LPCWSTR = [*:0]const u16;
const LPWSTR = [*:0]u16;
const DWORD = u32;
const UINT = u32;
const BOOL = i32;
const LONG = i32;
const LRESULT = isize;
const WPARAM = usize;
const LPARAM = isize;

const app_name = "FixText";
const model = "gemini-3.1-flash-lite";

const WM_DESTROY = 0x0002;
const WM_COMMAND = 0x0111;
const WM_KEYDOWN = 0x0100;
const WM_SYSKEYDOWN = 0x0104;
const WM_APP = 0x8000;
const WM_TRAY = WM_APP + 1;
const WM_RBUTTONUP = 0x0205;
const WM_LBUTTONUP = 0x0202;

const VK_CONTROL = 0x11;
const VK_C = 0x43;
const VK_V = 0x56;
const WH_KEYBOARD_LL = 13;
const HC_ACTION = 0;

const NIM_ADD = 0x00000000;
const NIM_MODIFY = 0x00000001;
const NIM_DELETE = 0x00000002;
const NIF_MESSAGE = 0x00000001;
const NIF_ICON = 0x00000002;
const NIF_TIP = 0x00000004;
const NIF_INFO = 0x00000010;

const MF_STRING = 0x00000000;
const MF_SEPARATOR = 0x00000800;
const MF_CHECKED = 0x00000008;
const MF_UNCHECKED = 0x00000000;
const TPM_RIGHTBUTTON = 0x0002;
const TPM_RETURNCMD = 0x0100;
const TPM_NONOTIFY = 0x0080;

const CF_UNICODETEXT = 13;
const GMEM_MOVEABLE = 0x0002;
const REG_SZ = 1;
const REG_DWORD = 4;
const KEY_READ = 0x20019;
const KEY_WRITE = 0x20006;
const ERROR_SUCCESS = 0;

const WINHTTP_ACCESS_TYPE_DEFAULT_PROXY = 0;
const WINHTTP_FLAG_SECURE = 0x00800000;
const WINHTTP_NO_REFERER: ?LPCWSTR = null;
const WINHTTP_DEFAULT_ACCEPT_TYPES: ?*anyopaque = null;

const FILE_APPEND_DATA = 0x0004;
const FILE_SHARE_READ = 0x00000001;
const FILE_SHARE_WRITE = 0x00000002;
const OPEN_ALWAYS = 4;
const FILE_ATTRIBUTE_NORMAL = 0x00000080;
const INVALID_HANDLE_VALUE: HANDLE = @ptrFromInt(std.math.maxInt(usize));
const KEYEVENTF_KEYUP = 0x0002;
const TRIGGER_DEBOUNCE_MS = 700;
const DOUBLE_COPY_WINDOW_MS = 650;
const COPY_SEQUENCE_POLL_MS = 25;
const INTERNET_DEFAULT_HTTPS_PORT = 443;

const ID_FORMAT_BASE = 1000;
const ID_FIX_NOW = 2000;
const ID_STARTUP = 2001;
const ID_NOTIFICATIONS = 2002;
const ID_EXIT = 2003;

const HKEY_CURRENT_USER: HKEY = @ptrFromInt(0x80000001);

const Format = struct {
    title: []const u8,
    instructions: []const u8,
};

const common =
    \\Corrige el texto de forma minima.
    \\Arregla solo errores claros de ortografia, gramatica, puntuacion, concordancia u orden.
    \\Conserva las palabras, estructura, tono e intencion originales siempre que sea posible.
    \\Si ya esta bien, devuelve practicamente lo mismo.
    \\Mismo idioma. Espanol: Castellano de Espana.
    \\Sin explicaciones, comentarios ni comillas.
;

const formal = common ++
    \\
    \\- Haz que el mensaje sea formal pero sigue el estilo del mensaje original, no trates de usted si no lo hace el mensaje original.
;

const cult = common ++
    \\
    \\- Utiliza un lenguaje antiguo, culto, con lexico elevado y con ciertos insultos complejos y elaborados.
;

const valle_inclan = cult ++
    \\
    \\- Utiliza un lenguaje parecido al de Valle Inclan en Luces de Bohemia y utiliza insultos y palabras en desuso de la epoca.
;

const non_sense = cult ++
    \\
    \\- Haz un juego de palabras y cambia el significado de la frase para que no tenga ningun sentido logico pero si que este bien escrito.
;

const formal_english = formal ++
    \\
    \\- Answer in English.
;

const fix_markdown =
    \\Fix any errors in the Markdown formatting of the text.
    \\Add proper headings, lists, and formatting (bold, italic, etc.) to make the text more readable and structured.
    \\Only return the modified Markdown text without any additional explanations or comments.
;

const formats = [_]Format{
    .{ .title = "Formal English", .instructions = formal_english },
    .{ .title = "Fix", .instructions = common },
    .{ .title = "Fix Markdown and format better", .instructions = fix_markdown },
    .{ .title = "Formal", .instructions = formal },
    .{ .title = "Cult", .instructions = cult },
    .{ .title = "Valle Inclan", .instructions = valle_inclan },
    .{ .title = "Non sense", .instructions = non_sense },
};

const POINT = extern struct {
    x: LONG,
    y: LONG,
};

const MSG = extern struct {
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    time: DWORD,
    pt: POINT,
    lPrivate: DWORD,
};

const WNDCLASSEXW = extern struct {
    cbSize: UINT,
    style: UINT,
    lpfnWndProc: *const fn (HWND, UINT, WPARAM, LPARAM) callconv(.winapi) LRESULT,
    cbClsExtra: c_int,
    cbWndExtra: c_int,
    hInstance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    lpszMenuName: ?LPCWSTR,
    lpszClassName: LPCWSTR,
    hIconSm: HICON,
};

const GUID = extern struct {
    Data1: u32,
    Data2: u16,
    Data3: u16,
    Data4: [8]u8,
};

const NOTIFYICONDATAW = extern struct {
    cbSize: DWORD,
    hWnd: HWND,
    uID: UINT,
    uFlags: UINT,
    uCallbackMessage: UINT,
    hIcon: HICON,
    szTip: [128]u16,
    dwState: DWORD,
    dwStateMask: DWORD,
    szInfo: [256]u16,
    uTimeoutOrVersion: UINT,
    szInfoTitle: [64]u16,
    dwInfoFlags: DWORD,
    guidItem: GUID,
    hBalloonIcon: HICON,
};

const KBDLLHOOKSTRUCT = extern struct {
    vkCode: DWORD,
    scanCode: DWORD,
    flags: DWORD,
    time: DWORD,
    dwExtraInfo: usize,
};

extern "kernel32" fn GetModuleHandleW(lpModuleName: ?LPCWSTR) callconv(.winapi) HINSTANCE;
extern "kernel32" fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) callconv(.winapi) DWORD;
extern "kernel32" fn GlobalAlloc(uFlags: UINT, dwBytes: usize) callconv(.winapi) HGLOBAL;
extern "kernel32" fn GlobalLock(hMem: HGLOBAL) callconv(.winapi) ?*anyopaque;
extern "kernel32" fn GlobalUnlock(hMem: HGLOBAL) callconv(.winapi) BOOL;
extern "kernel32" fn GlobalSize(hMem: HGLOBAL) callconv(.winapi) usize;
extern "kernel32" fn GlobalFree(hMem: HGLOBAL) callconv(.winapi) HGLOBAL;
extern "kernel32" fn lstrlenW(lpString: LPCWSTR) callconv(.winapi) c_int;
extern "kernel32" fn CreateFileW(LPCWSTR, DWORD, DWORD, ?*anyopaque, DWORD, DWORD, HANDLE) callconv(.winapi) HANDLE;
extern "kernel32" fn WriteFile(HANDLE, [*]const u8, DWORD, ?*DWORD, ?*anyopaque) callconv(.winapi) BOOL;
extern "kernel32" fn CloseHandle(HANDLE) callconv(.winapi) BOOL;
extern "kernel32" fn Sleep(DWORD) callconv(.winapi) void;
extern "kernel32" fn GetTickCount64() callconv(.winapi) u64;

extern "user32" fn RegisterClassExW(*const WNDCLASSEXW) callconv(.winapi) u16;
extern "user32" fn CreateWindowExW(DWORD, LPCWSTR, LPCWSTR, DWORD, c_int, c_int, c_int, c_int, HWND, HMENU, HINSTANCE, ?*anyopaque) callconv(.winapi) HWND;
extern "user32" fn DefWindowProcW(HWND, UINT, WPARAM, LPARAM) callconv(.winapi) LRESULT;
extern "user32" fn DestroyWindow(HWND) callconv(.winapi) BOOL;
extern "user32" fn PostQuitMessage(c_int) callconv(.winapi) void;
extern "user32" fn GetMessageW(*MSG, HWND, UINT, UINT) callconv(.winapi) BOOL;
extern "user32" fn TranslateMessage(*const MSG) callconv(.winapi) BOOL;
extern "user32" fn DispatchMessageW(*const MSG) callconv(.winapi) LRESULT;
extern "user32" fn SetWindowsHookExW(c_int, *const fn (c_int, WPARAM, LPARAM) callconv(.winapi) LRESULT, HINSTANCE, DWORD) callconv(.winapi) HHOOK;
extern "user32" fn UnhookWindowsHookEx(HHOOK) callconv(.winapi) BOOL;
extern "user32" fn CallNextHookEx(HHOOK, c_int, WPARAM, LPARAM) callconv(.winapi) LRESULT;
extern "user32" fn LoadIconW(HINSTANCE, LPCWSTR) callconv(.winapi) HICON;
extern "user32" fn CreatePopupMenu() callconv(.winapi) HMENU;
extern "user32" fn AppendMenuW(HMENU, UINT, usize, ?LPCWSTR) callconv(.winapi) BOOL;
extern "user32" fn DestroyMenu(HMENU) callconv(.winapi) BOOL;
extern "user32" fn TrackPopupMenu(HMENU, UINT, c_int, c_int, c_int, HWND, ?*const anyopaque) callconv(.winapi) UINT;
extern "user32" fn SetForegroundWindow(HWND) callconv(.winapi) BOOL;
extern "user32" fn GetCursorPos(*POINT) callconv(.winapi) BOOL;
extern "user32" fn OpenClipboard(HWND) callconv(.winapi) BOOL;
extern "user32" fn CloseClipboard() callconv(.winapi) BOOL;
extern "user32" fn EmptyClipboard() callconv(.winapi) BOOL;
extern "user32" fn GetClipboardData(UINT) callconv(.winapi) HANDLE;
extern "user32" fn SetClipboardData(UINT, HANDLE) callconv(.winapi) HANDLE;
extern "user32" fn GetAsyncKeyState(c_int) callconv(.winapi) i16;
extern "user32" fn keybd_event(u8, u8, DWORD, usize) callconv(.winapi) void;

extern "shell32" fn Shell_NotifyIconW(DWORD, *NOTIFYICONDATAW) callconv(.winapi) BOOL;

extern "advapi32" fn RegCreateKeyExW(HKEY, LPCWSTR, DWORD, ?LPWSTR, DWORD, DWORD, ?*anyopaque, *HKEY, ?*DWORD) callconv(.winapi) LONG;
extern "advapi32" fn RegOpenKeyExW(HKEY, LPCWSTR, DWORD, DWORD, *HKEY) callconv(.winapi) LONG;
extern "advapi32" fn RegSetValueExW(HKEY, LPCWSTR, DWORD, DWORD, [*]const u8, DWORD) callconv(.winapi) LONG;
extern "advapi32" fn RegQueryValueExW(HKEY, LPCWSTR, ?*DWORD, ?*DWORD, ?[*]u8, ?*DWORD) callconv(.winapi) LONG;
extern "advapi32" fn RegDeleteValueW(HKEY, LPCWSTR) callconv(.winapi) LONG;
extern "advapi32" fn RegCloseKey(HKEY) callconv(.winapi) LONG;

extern "winhttp" fn WinHttpOpen(LPCWSTR, DWORD, ?LPCWSTR, ?LPCWSTR, DWORD) callconv(.winapi) HINTERNET;
extern "winhttp" fn WinHttpConnect(HINTERNET, LPCWSTR, u16, DWORD) callconv(.winapi) HINTERNET;
extern "winhttp" fn WinHttpOpenRequest(HINTERNET, LPCWSTR, LPCWSTR, ?LPCWSTR, ?LPCWSTR, ?*anyopaque, DWORD) callconv(.winapi) HINTERNET;
extern "winhttp" fn WinHttpSendRequest(HINTERNET, ?LPCWSTR, DWORD, ?*anyopaque, DWORD, DWORD, usize) callconv(.winapi) BOOL;
extern "winhttp" fn WinHttpReceiveResponse(HINTERNET, ?*anyopaque) callconv(.winapi) BOOL;
extern "winhttp" fn WinHttpQueryDataAvailable(HINTERNET, *DWORD) callconv(.winapi) BOOL;
extern "winhttp" fn WinHttpReadData(HINTERNET, [*]u8, DWORD, *DWORD) callconv(.winapi) BOOL;
extern "winhttp" fn WinHttpCloseHandle(HINTERNET) callconv(.winapi) BOOL;

const App = struct {
    allocator: Allocator,
    hwnd: HWND = null,
    instance: HINSTANCE = null,
    selected_format: usize = 1,
    processing: std.atomic.Value(bool) = .init(false),
    notifications_enabled: std.atomic.Value(bool) = .init(true),
    keyboard_hook: HHOOK = null,
    last_trigger_tick: std.atomic.Value(u64) = .init(0),
    last_copy_tick: std.atomic.Value(u64) = .init(0),
    copy_sequence_count: std.atomic.Value(u32) = .init(0),
    copy_sequence_generation: std.atomic.Value(u64) = .init(0),
    http_session: HINTERNET = null,
    http_connect: HINTERNET = null,
    tray: NOTIFYICONDATAW = undefined,
};

var g_app: ?*App = null;

pub fn main() !void {
    var debug_allocator: std.heap.DebugAllocator(.{}) = .init;
    defer _ = debug_allocator.deinit();
    const allocator = debug_allocator.allocator();

    var app = App{ .allocator = allocator };
    g_app = &app;
    app.selected_format = readSelectedFormat();
    app.notifications_enabled.store(readNotificationsEnabled(), .monotonic);

    try initWindow(&app);
    try initTray(&app, "FixText ready - Ctrl+C, C");
    initHttp(&app) catch |err| logEvent(app.allocator, "WinHTTP preconnect failed: {s}", .{@errorName(err)});
    logEvent(app.allocator, "starting FixText; model={s}; selected_format={s}", .{ model, formats[app.selected_format].title });
    app.keyboard_hook = SetWindowsHookExW(WH_KEYBOARD_LL, keyboardProc, app.instance, 0);
    if (app.keyboard_hook == null) {
        logEvent(app.allocator, "failed to install low-level Ctrl+C,C keyboard hook", .{});
        showBalloon(&app, "FixText", "Low-level keyboard listener failed");
    } else {
        logEvent(app.allocator, "installed low-level Ctrl+C,C keyboard hook", .{});
    }

    var msg: MSG = undefined;
    while (GetMessageW(&msg, null, 0, 0) != 0) {
        _ = TranslateMessage(&msg);
        _ = DispatchMessageW(&msg);
    }
}

fn initWindow(app: *App) !void {
    app.instance = GetModuleHandleW(null);
    const class_name = L("FixTextHiddenWindow");
    const icon = LoadIconW(null, @ptrFromInt(32512));
    const wc = WNDCLASSEXW{
        .cbSize = @sizeOf(WNDCLASSEXW),
        .style = 0,
        .lpfnWndProc = wndProc,
        .cbClsExtra = 0,
        .cbWndExtra = 0,
        .hInstance = app.instance,
        .hIcon = icon,
        .hCursor = null,
        .hbrBackground = null,
        .lpszMenuName = null,
        .lpszClassName = class_name,
        .hIconSm = icon,
    };
    if (RegisterClassExW(&wc) == 0) return error.RegisterClassFailed;
    app.hwnd = CreateWindowExW(0, class_name, L("FixText"), 0, 0, 0, 0, 0, null, null, app.instance, null);
    if (app.hwnd == null) return error.CreateWindowFailed;
}

fn initTray(app: *App, tip: []const u8) !void {
    app.tray = std.mem.zeroes(NOTIFYICONDATAW);
    app.tray.cbSize = @sizeOf(NOTIFYICONDATAW);
    app.tray.hWnd = app.hwnd;
    app.tray.uID = 1;
    app.tray.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    app.tray.uCallbackMessage = WM_TRAY;
    app.tray.hIcon = LoadIconW(null, @ptrFromInt(32512));
    fillWideBuf(&app.tray.szTip, tip);
    if (Shell_NotifyIconW(NIM_ADD, &app.tray) == 0) return error.TrayAddFailed;
}

fn setTrayTip(app: *App, tip: []const u8) void {
    app.tray.uFlags = NIF_TIP;
    fillWideBuf(&app.tray.szTip, tip);
    _ = Shell_NotifyIconW(NIM_MODIFY, &app.tray);
}

fn triggerReceived(app: *App, source: []const u8) void {
    const now = GetTickCount64();
    const last = app.last_trigger_tick.load(.monotonic);
    if (now >= last and now - last < TRIGGER_DEBOUNCE_MS) {
        logEvent(app.allocator, "ignored duplicate trigger from {s}", .{source});
        return;
    }
    app.last_trigger_tick.store(now, .monotonic);
    logEvent(app.allocator, "Ctrl+C,C received from {s}", .{source});
    showBalloon(app, "FixText", "Ctrl+C, C received");
    startTransform();
}

fn copyPressed(app: *App) void {
    const now = GetTickCount64();
    const last = app.last_copy_tick.load(.monotonic);
    if (last != 0 and now >= last and now - last <= DOUBLE_COPY_WINDOW_MS) {
        app.last_copy_tick.store(now, .monotonic);
        const count = app.copy_sequence_count.fetchAdd(1, .monotonic) + 1;
        logEvent(app.allocator, "Ctrl+C sequence count={d}", .{count});
        return;
    }

    app.last_copy_tick.store(now, .monotonic);
    app.copy_sequence_count.store(1, .monotonic);
    const generation = app.copy_sequence_generation.fetchAdd(1, .monotonic) + 1;
    logEvent(app.allocator, "started Ctrl+C sequence", .{});

    const thread = std.Thread.spawn(.{}, finishCopySequenceThread, .{ app, generation }) catch {
        logEvent(app.allocator, "failed to spawn Ctrl+C sequence finalizer", .{});
        return;
    };
    thread.detach();
}

fn finishCopySequenceThread(app: *App, generation: u64) void {
    while (true) {
        Sleep(COPY_SEQUENCE_POLL_MS);
        if (app.copy_sequence_generation.load(.monotonic) != generation) return;

        const last = app.last_copy_tick.load(.monotonic);
        const now = GetTickCount64();
        if (last != 0 and now >= last and now - last > DOUBLE_COPY_WINDOW_MS) break;
    }

    const count = app.copy_sequence_count.load(.monotonic);
    if (app.copy_sequence_generation.cmpxchgStrong(generation, generation + 1, .acquire, .monotonic) != null) return;

    if (count == 2) {
        triggerReceived(app, "keyboard hook");
    } else if (count > 2) {
        setTrayTip(app, "FixText shortcut aborted");
        showBalloon(app, "FixText", "Shortcut aborted: too many Ctrl+C presses");
        logEvent(app.allocator, "aborted Ctrl+C sequence because count={d}", .{count});
    } else {
        logEvent(app.allocator, "ignored Ctrl+C sequence because count={d}", .{count});
    }
}

fn keyboardProc(code: c_int, w_param: WPARAM, l_param: LPARAM) callconv(.winapi) LRESULT {
    if (code == HC_ACTION and (w_param == WM_KEYDOWN or w_param == WM_SYSKEYDOWN)) {
        const event: *const KBDLLHOOKSTRUCT = @ptrFromInt(@as(usize, @bitCast(l_param)));
        if (event.vkCode == VK_C and isCtrlDown()) {
            if (g_app) |app| copyPressed(app);
        }
    }
    return CallNextHookEx(null, code, w_param, l_param);
}

fn showBalloon(app: *App, title: []const u8, message: []const u8) void {
    if (!app.notifications_enabled.load(.monotonic)) return;
    app.tray.uFlags = NIF_INFO;
    fillWideBuf(&app.tray.szInfoTitle, title);
    fillWideBuf(&app.tray.szInfo, message);
    app.tray.dwInfoFlags = 1;
    _ = Shell_NotifyIconW(NIM_MODIFY, &app.tray);
}

fn isCtrlDown() bool {
    return (GetAsyncKeyState(VK_CONTROL) & @as(i16, @bitCast(@as(u16, 0x8000)))) != 0;
}

fn wndProc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) callconv(.winapi) LRESULT {
    switch (msg) {
        WM_TRAY => {
            if (l_param == WM_RBUTTONUP or l_param == WM_LBUTTONUP) showTrayMenu(hwnd);
            return 0;
        },
        WM_COMMAND => {
            handleCommand(@intCast(w_param & 0xffff));
            return 0;
        },
        WM_DESTROY => {
            if (g_app) |app| {
                logEvent(app.allocator, "exiting FixText", .{});
                if (app.keyboard_hook) |hook| _ = UnhookWindowsHookEx(hook);
                if (app.http_connect) |handle| _ = WinHttpCloseHandle(handle);
                if (app.http_session) |handle| _ = WinHttpCloseHandle(handle);
                _ = Shell_NotifyIconW(NIM_DELETE, &app.tray);
            }
            PostQuitMessage(0);
            return 0;
        },
        else => return DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

fn showTrayMenu(hwnd: HWND) void {
    const app = g_app orelse return;
    const menu = CreatePopupMenu() orelse return;
    defer _ = DestroyMenu(menu);

    appendMenuText(menu, ID_FIX_NOW, "Fix clipboard now (Ctrl+C, C)", false);
    _ = AppendMenuW(menu, MF_SEPARATOR, 0, null);
    for (formats, 0..) |format, i| {
        appendMenuText(menu, ID_FORMAT_BASE + i, format.title, i == app.selected_format);
    }
    _ = AppendMenuW(menu, MF_SEPARATOR, 0, null);
    appendMenuText(menu, ID_NOTIFICATIONS, "Show notifications", app.notifications_enabled.load(.monotonic));
    appendMenuText(menu, ID_STARTUP, "Start with Windows", isStartupEnabled());
    appendMenuText(menu, ID_EXIT, "Exit", false);

    var pt: POINT = undefined;
    if (GetCursorPos(&pt) == 0) return;
    _ = SetForegroundWindow(hwnd);
    const cmd = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY, pt.x, pt.y, 0, hwnd, null);
    if (cmd != 0) handleCommand(cmd);
}

fn appendMenuText(menu: HMENU, id: usize, text: []const u8, checked: bool) void {
    const app = g_app orelse return;
    const wide = std.unicode.utf8ToUtf16LeAllocZ(app.allocator, text) catch return;
    defer app.allocator.free(wide);
    const check_flag: UINT = if (checked) MF_CHECKED else MF_UNCHECKED;
    const flags: UINT = MF_STRING | check_flag;
    _ = AppendMenuW(menu, flags, id, wide.ptr);
}

fn handleCommand(cmd: UINT) void {
    const app = g_app orelse return;
    if (cmd >= ID_FORMAT_BASE and cmd < ID_FORMAT_BASE + formats.len) {
        app.selected_format = cmd - ID_FORMAT_BASE;
        writeSelectedFormat(app.selected_format);
        setTrayTip(app, formats[app.selected_format].title);
        logEvent(app.allocator, "selected format={s}", .{formats[app.selected_format].title});
        return;
    }

    switch (cmd) {
        ID_FIX_NOW => startTransform(),
        ID_NOTIFICATIONS => toggleNotifications(),
        ID_STARTUP => toggleStartup(),
        ID_EXIT => _ = DestroyWindow(app.hwnd),
        else => {},
    }
}

fn startTransform() void {
    const app = g_app orelse return;
    if (app.processing.cmpxchgStrong(false, true, .acquire, .monotonic) != null) {
        logEvent(app.allocator, "ignored transform request because one is already running", .{});
        return;
    }
    setTrayTip(app, "FixText working...");
    logEvent(app.allocator, "starting clipboard transform with format={s}", .{formats[app.selected_format].title});
    const thread = std.Thread.spawn(.{}, transformThread, .{app}) catch {
        app.processing.store(false, .release);
        setTrayTip(app, "FixText could not start worker");
        showBalloon(app, "FixText", "Could not start worker");
        logEvent(app.allocator, "failed to spawn transform worker", .{});
        return;
    };
    thread.detach();
}

fn transformThread(app: *App) void {
    defer app.processing.store(false, .release);
    transformClipboard(app) catch |err| {
        var buf: [128]u8 = undefined;
        const msg = std.fmt.bufPrint(&buf, "FixText error: {s}", .{@errorName(err)}) catch "FixText error";
        setTrayTip(app, msg);
        showBalloon(app, "FixText error", msg);
        logEvent(app.allocator, "transform failed: {s}", .{@errorName(err)});
        return;
    };
    setTrayTip(app, "FixText done - clipboard updated");
    showBalloon(app, "FixText", "Clipboard updated");
    logEvent(app.allocator, "transform completed; clipboard updated", .{});
}

fn transformClipboard(app: *App) !void {
    const input = try getClipboardText(app.allocator, app.hwnd);
    defer app.allocator.free(input);
    if (std.mem.trim(u8, input, " \t\r\n").len == 0) return error.EmptyClipboard;
    logEvent(app.allocator, "clipboard text loaded; bytes={d}", .{input.len});

    const key = try loadGeminiKey(app.allocator);
    defer app.allocator.free(key);
    logEvent(app.allocator, "Gemini API key loaded", .{});

    const api_start = GetTickCount64();
    const output = try geminiGenerate(app, key, input, formats[app.selected_format].instructions);
    const api_elapsed = GetTickCount64() - api_start;
    defer app.allocator.free(output);
    logEvent(app.allocator, "Gemini response received; bytes={d}; elapsed_ms={d}", .{ output.len, api_elapsed });

    try setClipboardText(app.allocator, app.hwnd, output);
    pasteClipboard();
    logEvent(app.allocator, "paste shortcut sent", .{});
}

fn getClipboardText(allocator: Allocator, hwnd: HWND) ![]u8 {
    if (OpenClipboard(hwnd) == 0) return error.OpenClipboardFailed;
    defer _ = CloseClipboard();

    const handle = GetClipboardData(CF_UNICODETEXT) orelse return error.NoUnicodeClipboardText;
    const locked = GlobalLock(handle) orelse return error.GlobalLockFailed;
    defer _ = GlobalUnlock(handle);

    const wide_ptr: [*:0]const u16 = @ptrCast(@alignCast(locked));
    const len: usize = @intCast(lstrlenW(wide_ptr));
    return std.unicode.utf16LeToUtf8Alloc(allocator, wide_ptr[0..len]);
}

fn setClipboardText(allocator: Allocator, hwnd: HWND, text: []const u8) !void {
    const wide = try std.unicode.utf8ToUtf16LeAllocZ(allocator, text);
    defer allocator.free(wide);
    const bytes = (wide.len + 1) * @sizeOf(u16);
    const mem = GlobalAlloc(GMEM_MOVEABLE, bytes) orelse return error.GlobalAllocFailed;
    errdefer _ = GlobalFree(mem);

    const locked = GlobalLock(mem) orelse return error.GlobalLockFailed;
    const dest: [*]u16 = @ptrCast(@alignCast(locked));
    @memcpy(dest[0 .. wide.len + 1], wide.ptr[0 .. wide.len + 1]);
    _ = GlobalUnlock(mem);

    if (OpenClipboard(hwnd) == 0) return error.OpenClipboardFailed;
    defer _ = CloseClipboard();
    if (EmptyClipboard() == 0) return error.EmptyClipboardFailed;
    if (SetClipboardData(CF_UNICODETEXT, mem) == null) return error.SetClipboardFailed;
}

fn pasteClipboard() void {
    Sleep(80);
    keybd_event(VK_C, 0, KEYEVENTF_KEYUP, 0);
    keybd_event(VK_CONTROL, 0, 0, 0);
    keybd_event(VK_V, 0, 0, 0);
    keybd_event(VK_V, 0, KEYEVENTF_KEYUP, 0);
    keybd_event(VK_CONTROL, 0, KEYEVENTF_KEYUP, 0);
}

fn loadGeminiKey(allocator: Allocator) ![]u8 {
    if (std.process.Environ.getAlloc(.{ .block = .global }, allocator, "GEMINI_API_KEY")) |value| {
        return value;
    } else |_| {}

    const file = readEnvFile(allocator) catch return error.MissingGeminiApiKey;
    defer allocator.free(file);
    var lines = std.mem.splitScalar(u8, file, '\n');
    while (lines.next()) |line_raw| {
        const line = std.mem.trim(u8, line_raw, " \t\r\n");
        if (std.mem.startsWith(u8, line, "GEMINI_API_KEY=")) {
            const value = std.mem.trim(u8, line["GEMINI_API_KEY=".len..], " \t\r\n\"'");
            if (value.len == 0) return error.MissingGeminiApiKey;
            return allocator.dupe(u8, value);
        }
    }
    return error.MissingGeminiApiKey;
}

fn readEnvFile(allocator: Allocator) ![]u8 {
    if (readFileAllocPath(allocator, ".env")) |file| return file else |_| {}

    const exe_path = try getExePathUtf8(allocator);
    defer allocator.free(exe_path);
    const exe_dir = dirName(exe_path) orelse return error.MissingGeminiApiKey;

    const beside_exe = try std.fmt.allocPrint(allocator, "{s}\\.env", .{exe_dir});
    defer allocator.free(beside_exe);
    if (readFileAllocPath(allocator, beside_exe)) |file| return file else |_| {}

    if (dirName(exe_dir)) |zig_out_dir| {
        if (dirName(zig_out_dir)) |project_dir| {
            const project_env = try std.fmt.allocPrint(allocator, "{s}\\.env", .{project_dir});
            defer allocator.free(project_env);
            if (readFileAllocPath(allocator, project_env)) |file| return file else |_| {}
        }
    }

    return error.MissingGeminiApiKey;
}

fn readFileAllocPath(allocator: Allocator, path: []const u8) ![]u8 {
    var io_threaded: std.Io.Threaded = .init(allocator, .{});
    defer io_threaded.deinit();
    return std.Io.Dir.cwd().readFileAlloc(io_threaded.io(), path, allocator, .limited(64 * 1024));
}

fn getExePathUtf8(allocator: Allocator) ![]u8 {
    var exe_buf: [32768]u16 = undefined;
    const len = GetModuleFileNameW(null, @ptrCast(&exe_buf), exe_buf.len);
    if (len == 0) return error.GetModuleFileNameFailed;
    return std.unicode.utf16LeToUtf8Alloc(allocator, exe_buf[0..len]);
}

fn dirName(path: []const u8) ?[]const u8 {
    var i = path.len;
    while (i > 0) {
        i -= 1;
        if (path[i] == '\\' or path[i] == '/') return path[0..i];
    }
    return null;
}

fn geminiGenerate(app: *App, api_key: []const u8, text: []const u8, instructions: []const u8) ![]u8 {
    const allocator = app.allocator;
    var body_out: std.Io.Writer.Allocating = .init(allocator);
    defer body_out.deinit();
    var jw: std.json.Stringify = .{ .writer = &body_out.writer, .options = .{} };
    try jw.beginObject();
    try jw.objectField("contents");
    try jw.beginArray();
    try jw.beginObject();
    try jw.objectField("role");
    try jw.write("user");
    try jw.objectField("parts");
    try jw.beginArray();
    try jw.beginObject();
    try jw.objectField("text");
    try jw.write(text);
    try jw.endObject();
    try jw.endArray();
    try jw.endObject();
    try jw.endArray();
    try jw.objectField("systemInstruction");
    try jw.beginObject();
    try jw.objectField("parts");
    try jw.beginArray();
    try jw.beginObject();
    try jw.objectField("text");
    try jw.write(instructions);
    try jw.endObject();
    try jw.endArray();
    try jw.endObject();
    try jw.objectField("generationConfig");
    try jw.beginObject();
    try jw.objectField("responseMimeType");
    try jw.write("text/plain");
    try jw.objectField("thinkingConfig");
    try jw.beginObject();
    try jw.objectField("thinkingLevel");
    try jw.write("minimal");
    try jw.endObject();
    try jw.objectField("maxOutputTokens");
    try jw.write(512);
    try jw.endObject();
    try jw.endObject();

    const body = body_out.written();
    const response = try postGemini(app, api_key, body);
    defer allocator.free(response);
    return extractGeminiText(allocator, response);
}

fn initHttp(app: *App) !void {
    const start = GetTickCount64();
    if (app.http_session == null) {
        app.http_session = WinHttpOpen(L("FixText/1.0"), WINHTTP_ACCESS_TYPE_DEFAULT_PROXY, null, null, 0) orelse return error.WinHttpOpenFailed;
        logEvent(app.allocator, "WinHTTP session opened; elapsed_ms={d}", .{GetTickCount64() - start});
    }
    if (app.http_connect == null) {
        const connect_start = GetTickCount64();
        app.http_connect = WinHttpConnect(app.http_session.?, L("generativelanguage.googleapis.com"), INTERNET_DEFAULT_HTTPS_PORT, 0) orelse return error.WinHttpConnectFailed;
        logEvent(app.allocator, "WinHTTP connect handle opened; elapsed_ms={d}", .{GetTickCount64() - connect_start});
    }
}

fn postGemini(app: *App, api_key: []const u8, body: []const u8) ![]u8 {
    const allocator = app.allocator;
    const host = L("generativelanguage.googleapis.com");
    const path = try std.fmt.allocPrint(allocator, "/v1beta/models/{s}:generateContent?key={s}", .{ model, api_key });
    defer allocator.free(path);
    const path_w = try std.unicode.utf8ToUtf16LeAllocZ(allocator, path);
    defer allocator.free(path_w);

    _ = host;
    try initHttp(app);
    const open_start = GetTickCount64();
    const request = WinHttpOpenRequest(app.http_connect.?, L("POST"), path_w.ptr, null, WINHTTP_NO_REFERER, WINHTTP_DEFAULT_ACCEPT_TYPES, WINHTTP_FLAG_SECURE) orelse return error.WinHttpOpenRequestFailed;
    defer _ = WinHttpCloseHandle(request);
    logEvent(allocator, "WinHTTP request opened; elapsed_ms={d}", .{GetTickCount64() - open_start});

    const send_start = GetTickCount64();
    const headers = L("Content-Type: application/json\r\nAccept: application/json\r\nConnection: keep-alive\r\n");
    if (WinHttpSendRequest(request, headers, 0xffffffff, @constCast(body.ptr), @intCast(body.len), @intCast(body.len), 0) == 0) return error.WinHttpSendRequestFailed;
    logEvent(allocator, "Gemini request sent; body_bytes={d}; elapsed_ms={d}", .{ body.len, GetTickCount64() - send_start });

    const wait_start = GetTickCount64();
    if (WinHttpReceiveResponse(request, null) == 0) return error.WinHttpReceiveResponseFailed;
    logEvent(allocator, "Gemini response headers received; elapsed_ms={d}", .{GetTickCount64() - wait_start});

    const read_start = GetTickCount64();
    var out: std.Io.Writer.Allocating = .init(allocator);
    errdefer out.deinit();
    while (true) {
        var available: DWORD = 0;
        if (WinHttpQueryDataAvailable(request, &available) == 0) return error.WinHttpQueryDataAvailableFailed;
        if (available == 0) break;
        const before = out.written().len;
        try out.writer.splatByteAll(0, available);
        var read: DWORD = 0;
        if (WinHttpReadData(request, out.written()[before..].ptr, available, &read) == 0) return error.WinHttpReadDataFailed;
        out.shrinkRetainingCapacity(before + read);
    }
    logEvent(allocator, "Gemini response body read; bytes={d}; elapsed_ms={d}", .{ out.written().len, GetTickCount64() - read_start });
    return out.toOwnedSlice();
}

fn extractGeminiText(allocator: Allocator, response: []const u8) ![]u8 {
    var parsed = try std.json.parseFromSlice(std.json.Value, allocator, response, .{});
    defer parsed.deinit();
    const root = parsed.value;
    if (root != .object) return error.InvalidGeminiResponse;
    if (root.object.get("error")) |err_value| {
        if (err_value == .object) {
            if (err_value.object.get("message")) |message| {
                if (message == .string) return error.GeminiApiError;
            }
        }
        return error.GeminiApiError;
    }

    const candidates = root.object.get("candidates") orelse return error.InvalidGeminiResponse;
    if (candidates != .array or candidates.array.items.len == 0) return error.InvalidGeminiResponse;
    const content = candidates.array.items[0].object.get("content") orelse return error.InvalidGeminiResponse;
    const parts = content.object.get("parts") orelse return error.InvalidGeminiResponse;
    if (parts != .array) return error.InvalidGeminiResponse;

    var out: std.Io.Writer.Allocating = .init(allocator);
    errdefer out.deinit();
    for (parts.array.items) |part| {
        if (part == .object) {
            if (part.object.get("text")) |value| {
                if (value == .string) try out.writer.writeAll(value.string);
            }
        }
    }
    if (out.written().len == 0) return error.EmptyGeminiResponse;
    return out.toOwnedSlice();
}

fn readSelectedFormat() usize {
    var key: HKEY = null;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, L("Software\\FixText"), 0, KEY_READ, &key) != ERROR_SUCCESS) return 1;
    defer _ = RegCloseKey(key);
    var typ: DWORD = 0;
    var data: DWORD = 0;
    var size: DWORD = @sizeOf(DWORD);
    if (RegQueryValueExW(key, L("SelectedFormat"), null, &typ, @ptrCast(&data), &size) != ERROR_SUCCESS) return 1;
    if (typ != REG_DWORD or data >= formats.len) return 1;
    return data;
}

fn writeSelectedFormat(index: usize) void {
    var key: HKEY = null;
    var disposition: DWORD = 0;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, L("Software\\FixText"), 0, null, 0, KEY_WRITE, null, &key, &disposition) != ERROR_SUCCESS) return;
    defer _ = RegCloseKey(key);
    var data: DWORD = @intCast(index);
    _ = RegSetValueExW(key, L("SelectedFormat"), 0, REG_DWORD, @ptrCast(&data), @sizeOf(DWORD));
}

fn readNotificationsEnabled() bool {
    var key: HKEY = null;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, L("Software\\FixText"), 0, KEY_READ, &key) != ERROR_SUCCESS) return true;
    defer _ = RegCloseKey(key);
    var typ: DWORD = 0;
    var data: DWORD = 1;
    var size: DWORD = @sizeOf(DWORD);
    if (RegQueryValueExW(key, L("NotificationsEnabled"), null, &typ, @ptrCast(&data), &size) != ERROR_SUCCESS) return true;
    if (typ != REG_DWORD) return true;
    return data != 0;
}

fn writeNotificationsEnabled(enabled: bool) void {
    var key: HKEY = null;
    var disposition: DWORD = 0;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, L("Software\\FixText"), 0, null, 0, KEY_WRITE, null, &key, &disposition) != ERROR_SUCCESS) return;
    defer _ = RegCloseKey(key);
    var data: DWORD = if (enabled) 1 else 0;
    _ = RegSetValueExW(key, L("NotificationsEnabled"), 0, REG_DWORD, @ptrCast(&data), @sizeOf(DWORD));
}

fn toggleNotifications() void {
    const app = g_app orelse return;
    const enabled = !app.notifications_enabled.load(.monotonic);
    app.notifications_enabled.store(enabled, .monotonic);
    writeNotificationsEnabled(enabled);
    if (enabled) {
        setTrayTip(app, "FixText notifications enabled");
        showBalloon(app, "FixText", "Notifications enabled");
        logEvent(app.allocator, "notifications enabled", .{});
    } else {
        setTrayTip(app, "FixText notifications disabled");
        logEvent(app.allocator, "notifications disabled", .{});
    }
}

fn isStartupEnabled() bool {
    var key: HKEY = null;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, L("Software\\Microsoft\\Windows\\CurrentVersion\\Run"), 0, KEY_READ, &key) != ERROR_SUCCESS) return false;
    defer _ = RegCloseKey(key);
    var typ: DWORD = 0;
    var size: DWORD = 0;
    return RegQueryValueExW(key, L("FixText"), null, &typ, null, &size) == ERROR_SUCCESS;
}

fn toggleStartup() void {
    const app = g_app orelse return;
    var key: HKEY = null;
    var disposition: DWORD = 0;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, L("Software\\Microsoft\\Windows\\CurrentVersion\\Run"), 0, null, 0, KEY_WRITE, null, &key, &disposition) != ERROR_SUCCESS) return;
    defer _ = RegCloseKey(key);

    if (isStartupEnabled()) {
        _ = RegDeleteValueW(key, L("FixText"));
        setTrayTip(app, "FixText startup disabled");
        logEvent(app.allocator, "startup disabled", .{});
        return;
    }

    var exe_buf: [32768]u16 = undefined;
    const len = GetModuleFileNameW(null, @ptrCast(&exe_buf), exe_buf.len);
    if (len == 0) return;

    var cmd_buf: [32772]u16 = undefined;
    cmd_buf[0] = '"';
    @memcpy(cmd_buf[1 .. len + 1], exe_buf[0..len]);
    cmd_buf[len + 1] = '"';
    cmd_buf[len + 2] = 0;
    const bytes: DWORD = @intCast((len + 3) * @sizeOf(u16));
    _ = RegSetValueExW(key, L("FixText"), 0, REG_SZ, @ptrCast(&cmd_buf), bytes);
    setTrayTip(app, "FixText startup enabled");
    logEvent(app.allocator, "startup enabled", .{});
}

fn fillWideBuf(dest: []u16, text: []const u8) void {
    @memset(dest, 0);
    const max = if (dest.len == 0) 0 else dest.len - 1;
    const written = std.unicode.utf8ToUtf16Le(dest[0..max], text) catch 0;
    dest[written] = 0;
}

fn logEvent(allocator: Allocator, comptime fmt: []const u8, args: anytype) void {
    const msg = std.fmt.allocPrint(allocator, fmt, args) catch return;
    defer allocator.free(msg);
    const line = std.fmt.allocPrint(allocator, "{s}\r\n", .{msg}) catch return;
    defer allocator.free(line);

    const path = logPath(allocator) catch return;
    defer allocator.free(path);
    const path_w = std.unicode.utf8ToUtf16LeAllocZ(allocator, path) catch return;
    defer allocator.free(path_w);

    const file = CreateFileW(path_w.ptr, FILE_APPEND_DATA, FILE_SHARE_READ | FILE_SHARE_WRITE, null, OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, null);
    if (file == INVALID_HANDLE_VALUE) return;
    defer _ = CloseHandle(file);

    var written: DWORD = 0;
    _ = WriteFile(file, line.ptr, @intCast(line.len), &written, null);
}

fn logPath(allocator: Allocator) ![]u8 {
    const exe_path = try getExePathUtf8(allocator);
    defer allocator.free(exe_path);
    const exe_dir = dirName(exe_path) orelse ".";
    return std.fmt.allocPrint(allocator, "{s}\\fixtext.log", .{exe_dir});
}
