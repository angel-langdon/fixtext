# FixText

Native Windows tray utility that rewrites the current clipboard text with Gemini.

## Features

- Always-on double-copy listener: press `Ctrl+C` twice quickly.
- Windows tray menu for manual fixing, format selection, notification toggle, startup toggle, and exit.
- Clipboard in, clipboard out.
- Automatically pastes the rewritten text after updating the clipboard.
- Tiny diagnostic log at `zig-out/bin/fixtext.log`.
- Gemini API model: `gemini-3.1-flash-lite` with `thinkingLevel: "minimal"`.
- API key loaded from `GEMINI_API_KEY` or a local `.env` file.

## Build

```powershell
zig build -Doptimize=ReleaseSafe
```

The executable is created at `zig-out/bin/fixtext.exe`.

## Usage

1. Set `GEMINI_API_KEY` in your environment or create `.env`:

   ```text
   GEMINI_API_KEY=your-key-here
   ```

2. Start `zig-out/bin/fixtext.exe`.
3. Pick the active format from the tray menu.
4. Select text, press `Ctrl+C` twice quickly, and FixText will rewrite the clipboard and paste it back.

Use the tray menu's `Show notifications` option to enable or disable FixText balloon notifications.

Use the tray menu's `Start with Windows` option to add or remove FixText from `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
