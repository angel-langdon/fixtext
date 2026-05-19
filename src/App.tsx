import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

type ModelChoice = {
  provider: string;
  id: string;
  label: string;
  note: string;
  free_tier: boolean;
};

type PromptProfile = {
  id: string;
  title: string;
  instructions: string;
};

type Settings = {
  selected_provider: string;
  api_key: string;
  groq_api_key: string;
  selected_model: string;
  custom_model: string;
  selected_profile: string;
  max_output_tokens: number;
  thinking_level: string;
  temperature: number;
  auto_paste: boolean;
  show_window_on_start: boolean;
  show_status_overlay: boolean;
  shortcut_enabled: boolean;
  shortcut_key: string;
  shortcut_ctrl: boolean;
  shortcut_alt: boolean;
  shortcut_shift: boolean;
  shortcut_win: boolean;
  shortcut_presses: number;
  shortcut_window_ms: number;
};

type ActivityItem = {
  id: number;
  kind: string;
  message: string;
  created_at: number;
};

type AppViewState = {
  settings: Settings;
  models: ModelChoice[];
  profiles: PromptProfile[];
  startup_enabled: boolean;
  busy: boolean;
  activity: ActivityItem[];
};

type TransformResult = {
  input: string;
  output: string;
  provider: string;
  model: string;
  profile: string;
};

type RustStatusEvent = {
  stage: string;
  message: string;
  created_at: number;
};

type StatusToast = RustStatusEvent & {
  id: number;
};

type ModelTestResult = {
  provider: string;
  modelId: string;
  label: string;
  elapsedMs: number;
  output: string;
  error: string;
};

const emptySettings: Settings = {
  selected_provider: "gemini",
  api_key: "",
  groq_api_key: "",
  selected_model: "gemini-3.1-flash-lite",
  custom_model: "",
  selected_profile: "fix",
  max_output_tokens: 512,
  thinking_level: "minimal",
  temperature: 0.2,
  auto_paste: true,
  show_window_on_start: true,
  show_status_overlay: true,
  shortcut_enabled: true,
  shortcut_key: "C",
  shortcut_ctrl: true,
  shortcut_alt: false,
  shortcut_shift: false,
  shortcut_win: false,
  shortcut_presses: 2,
  shortcut_window_ms: 650,
};

const shortcutKeys = [
  ..."ABCDEFGHIJKLMNOPQRSTUVWXYZ".split(""),
  ..."0123456789".split(""),
  ...Array.from({ length: 12 }, (_, index) => `F${index + 1}`),
];

const providerLabels: Record<string, string> = {
  gemini: "Gemini",
  groq: "Groq",
};

function App() {
  const [settings, setSettings] = useState<Settings>(emptySettings);
  const [models, setModels] = useState<ModelChoice[]>([]);
  const [profiles, setProfiles] = useState<PromptProfile[]>([]);
  const [selectedProfileId, setSelectedProfileId] = useState("fix");
  const [startupEnabled, setStartupEnabledState] = useState(false);
  const [activity, setActivity] = useState<ActivityItem[]>([]);
  const [busy, setBusy] = useState(false);
  const [draftText, setDraftText] = useState("");
  const [resultText, setResultText] = useState("");
  const [modelTestResults, setModelTestResults] = useState<ModelTestResult[]>([]);
  const [stateJson, setStateJson] = useState("");
  const [status, setStatus] = useState("Loading");
  const [error, setError] = useState("");
  const [statusToast, setStatusToast] = useState<StatusToast | null>(null);
  const loadedState = useRef(false);
  const savedSnapshot = useRef("");
  const autosaveTimer = useRef<number | undefined>(undefined);
  const autosaveGeneration = useRef(0);

  const selectedProfile = useMemo(
    () => profiles.find((profile) => profile.id === selectedProfileId) ?? profiles[0],
    [profiles, selectedProfileId],
  );
  const providerModels = useMemo(
    () => models.filter((model) => model.provider === settings.selected_provider),
    [models, settings.selected_provider],
  );

  useEffect(() => {
    refreshState();
    let unlisten: (() => void) | undefined;
    listen<RustStatusEvent>("rust-status", (event) => {
      const toast = { ...event.payload, id: Date.now() + Math.random() };
      setStatusToast(toast);
      window.setTimeout(() => {
        setStatusToast((current) => (current?.id === toast.id ? null : current));
      }, 3200);
    }).then((cleanup) => {
      unlisten = cleanup;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    if (!loadedState.current) return;
    const payload = persistentPayload(settings, profiles, selectedProfileId);
    const snapshot = JSON.stringify(payload);
    if (snapshot === savedSnapshot.current) return;

    const generation = autosaveGeneration.current + 1;
    autosaveGeneration.current = generation;
    setStatus("Autosaving");
    if (autosaveTimer.current) {
      window.clearTimeout(autosaveTimer.current);
    }
    autosaveTimer.current = window.setTimeout(async () => {
      try {
        const state = await invoke<AppViewState>("save_settings", payload);
        if (generation !== autosaveGeneration.current) return;
        rememberSavedState(state.settings, state.profiles);
        setStartupEnabledState(state.startup_enabled);
        setActivity(state.activity);
        setBusy(state.busy);
        setError("");
        setStatus("Autosaved");
      } catch (err) {
        if (generation !== autosaveGeneration.current) return;
        setError(String(err));
        setStatus("Autosave failed");
      }
    }, 450);

    return () => {
      if (autosaveTimer.current) {
        window.clearTimeout(autosaveTimer.current);
      }
    };
  }, [settings, profiles, selectedProfileId]);

  async function refreshState() {
    try {
      const state = await invoke<AppViewState>("load_app_state");
      applyState(state);
      setStatus("Ready");
    } catch (err) {
      setError(String(err));
      setStatus("Could not load state");
    }
  }

  function applyState(state: AppViewState) {
    rememberSavedState(state.settings, state.profiles);
    setSettings(state.settings);
    setModels(state.models);
    setProfiles(state.profiles);
    setSelectedProfileId(state.settings.selected_profile);
    setStartupEnabledState(state.startup_enabled);
    setActivity(state.activity);
    setBusy(state.busy);
    loadedState.current = true;
  }

  function persistentPayload(
    nextSettings: Settings,
    nextProfiles: PromptProfile[],
    nextSelectedProfileId: string,
  ) {
    return {
      settings: { ...nextSettings, selected_profile: nextSelectedProfileId },
      profiles: nextProfiles,
    };
  }

  function rememberSavedState(nextSettings: Settings, nextProfiles: PromptProfile[]) {
    savedSnapshot.current = JSON.stringify({
      settings: nextSettings,
      profiles: nextProfiles,
    });
  }

  function updateSettings(patch: Partial<Settings>) {
    setSettings((current) => ({ ...current, ...patch }));
  }

  function updateSelectedProfile(patch: Partial<PromptProfile>) {
    if (!selectedProfile) return;
    setProfiles((current) =>
      current.map((profile) =>
        profile.id === selectedProfile.id ? { ...profile, ...patch } : profile,
      ),
    );
  }

  function yieldToUi() {
    return new Promise<void>((resolve) => {
      window.requestAnimationFrame(() => resolve());
    });
  }

  async function runOnDraft() {
    if (!selectedProfile || draftText.trim().length === 0) return;
    setBusy(true);
    setError("");
    setStatus("Fixing draft");
    try {
      const result = await invoke<TransformResult>("transform_text", {
        input: draftText,
        profile: selectedProfile,
        providerId: null,
        modelId: null,
      });
      setResultText(result.output);
      setStatus(`${result.profile} with ${providerLabels[result.provider] ?? result.provider} ${result.model}`);
      await refreshState();
    } catch (err) {
      setError(String(err));
      setStatus("Fix failed");
    } finally {
      setBusy(false);
    }
  }

  async function testAllModels() {
    if (!selectedProfile || draftText.trim().length === 0 || models.length === 0) return;
    setBusy(true);
    setError("");
    setModelTestResults([]);
    setResultText("");
    const nextResults: ModelTestResult[] = [];
    let firstOutputSet = false;

    try {
      for (const [index, model] of models.entries()) {
        setStatus(`Testing ${index + 1}/${models.length}: ${model.label}`);
        await yieldToUi();
        const started = performance.now();
        try {
          const result = await invoke<TransformResult>("transform_text", {
            input: draftText,
            profile: selectedProfile,
            providerId: model.provider,
            modelId: model.id,
          });
          const item = {
            provider: model.provider,
            modelId: model.id,
            label: model.label,
            elapsedMs: Math.round(performance.now() - started),
            output: result.output,
            error: "",
          };
          nextResults.push(item);
          setModelTestResults([...nextResults]);
          if (!firstOutputSet) {
            setResultText(result.output);
            firstOutputSet = true;
          }
          await yieldToUi();
        } catch (err) {
          nextResults.push({
            provider: model.provider,
            modelId: model.id,
            label: model.label,
            elapsedMs: Math.round(performance.now() - started),
            output: "",
            error: String(err),
          });
          setModelTestResults([...nextResults]);
          await yieldToUi();
        }
      }
      const failures = nextResults.filter((item) => item.error).length;
      setStatus(failures ? `Model test finished with ${failures} errors` : "Model test finished");
    } finally {
      setBusy(false);
    }
  }

  async function fixClipboardNow() {
    setBusy(true);
    setError("");
    setStatus("Fixing clipboard");
    try {
      const result = await invoke<TransformResult>("fix_clipboard");
      setDraftText(result.input);
      setResultText(result.output);
      setStatus("Clipboard updated");
      await refreshState();
    } catch (err) {
      setError(String(err));
      setStatus("Clipboard fix failed");
    } finally {
      setBusy(false);
    }
  }

  async function readClipboard() {
    try {
      setDraftText(await invoke<string>("read_clipboard_text"));
      setStatus("Clipboard loaded");
    } catch (err) {
      setError(String(err));
    }
  }

  async function copyResult() {
    if (!resultText) return;
    await invoke("write_clipboard_text", { text: resultText });
    setStatus("Result copied");
  }

  async function toggleStartup() {
    try {
      const enabled = await invoke<boolean>("set_startup_enabled", {
        enabled: !startupEnabled,
      });
      setStartupEnabledState(enabled);
      setStatus(enabled ? "Startup enabled" : "Startup disabled");
    } catch (err) {
      setError(String(err));
    }
  }

  async function quitApp() {
    await invoke("quit_app");
  }

  async function exportState() {
    try {
      const exported = await invoke<string>("export_app_state");
      setStateJson(exported);
      await invoke("write_clipboard_text", { text: exported });
      setStatus("Full state copied");
    } catch (err) {
      setError(String(err));
    }
  }

  async function importState() {
    if (stateJson.trim().length === 0) return;
    try {
      const state = await invoke<AppViewState>("import_app_state", { json: stateJson });
      applyState(state);
      setStatus("Full state imported");
      setError("");
    } catch (err) {
      setError(String(err));
      setStatus("Import failed");
    }
  }

  function addProfile() {
    const id = `profile-${Date.now()}`;
    const profile = {
      id,
      title: "New profile",
      instructions: "Rewrite the selected text. Return only the final text.",
    };
    setProfiles((current) => [...current, profile]);
    setSelectedProfileId(id);
  }

  function duplicateProfile() {
    if (!selectedProfile) return;
    const id = `${selectedProfile.id}-${Date.now()}`;
    setProfiles((current) => [
      ...current,
      { ...selectedProfile, id, title: `${selectedProfile.title} copy` },
    ]);
    setSelectedProfileId(id);
  }

  function deleteProfile() {
    if (!selectedProfile || profiles.length < 2) return;
    const next = profiles.filter((profile) => profile.id !== selectedProfile.id);
    setProfiles(next);
    setSelectedProfileId(next[0].id);
  }

  const activeModel =
    settings.selected_model === "custom" ? settings.custom_model : settings.selected_model;
  const activeProvider = providerLabels[settings.selected_provider] ?? settings.selected_provider;
  const shortcutLabel = [
    settings.shortcut_ctrl && "Ctrl",
    settings.shortcut_alt && "Alt",
    settings.shortcut_shift && "Shift",
    settings.shortcut_win && "Win",
    settings.shortcut_key,
  ]
    .filter(Boolean)
    .join("+");

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div>
          <p className="eyebrow">FixText</p>
          <h1>Clipboard rewrite console</h1>
        </div>

        <nav className="nav-stack" aria-label="Primary actions">
          <button onClick={fixClipboardNow} disabled={busy}>
            Fix clipboard
          </button>
          <button onClick={quitApp}>Quit</button>
        </nav>

        <div className="status-block">
          <span className={busy ? "status-dot busy" : "status-dot"} />
          <div>
            <strong>{status}</strong>
            <small>{activeProvider} {activeModel || "No model selected"}</small>
          </div>
        </div>

        {error && <p className="error-text">{error}</p>}
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <span>Windows tray active</span>
            <strong>{startupEnabled ? "Starts with Windows" : "Manual start"}</strong>
          </div>
          <div className="topbar-actions">
            <button onClick={toggleStartup}>
              {startupEnabled ? "Disable startup" : "Enable startup"}
            </button>
            <button onClick={refreshState}>Refresh</button>
          </div>
        </header>

        <section className="panel grid-two">
          <div className="field">
            <label htmlFor="api-key">Gemini API key</label>
            <input
              id="api-key"
              type="password"
              value={settings.api_key}
              onChange={(event) => updateSettings({ api_key: event.currentTarget.value })}
              placeholder="Stored in the local app config"
            />
          </div>

          <div className="field">
            <label htmlFor="groq-api-key">Groq API key</label>
            <input
              id="groq-api-key"
              type="password"
              value={settings.groq_api_key}
              onChange={(event) => updateSettings({ groq_api_key: event.currentTarget.value })}
              placeholder="Stored in the local app config"
            />
          </div>

          <div className="field">
            <label htmlFor="provider">Provider</label>
            <select
              id="provider"
              value={settings.selected_provider}
              onChange={(event) => {
                const provider = event.currentTarget.value;
                const firstModel = models.find((model) => model.provider === provider);
                updateSettings({
                  selected_provider: provider,
                  selected_model: firstModel?.id ?? "custom",
                });
              }}
            >
              <option value="gemini">Gemini</option>
              <option value="groq">Groq</option>
            </select>
          </div>

          <div className="field">
            <label htmlFor="model">Free-tier model</label>
            <select
              id="model"
              value={settings.selected_model}
              onChange={(event) => updateSettings({ selected_model: event.currentTarget.value })}
            >
              {providerModels.map((model) => (
                <option key={`${model.provider}:${model.id}`} value={model.id}>
                  {model.label}
                </option>
              ))}
              <option value="custom">Custom model id</option>
            </select>
          </div>

          {settings.selected_model === "custom" && (
            <div className="field full">
              <label htmlFor="custom-model">Custom model id</label>
              <input
                id="custom-model"
                value={settings.custom_model}
                onChange={(event) => updateSettings({ custom_model: event.currentTarget.value })}
                placeholder={settings.selected_provider === "groq" ? "llama-3.3-70b-versatile" : "gemini-3.1-flash-lite-preview"}
              />
            </div>
          )}

          <div className="model-list full">
            {providerModels.map((model) => (
              <button
                key={`${model.provider}:${model.id}`}
                className={settings.selected_model === model.id ? "model-row active" : "model-row"}
                onClick={() => updateSettings({ selected_model: model.id })}
              >
                <span>{model.label}</span>
                <small>{model.note}</small>
              </button>
            ))}
          </div>

          <label className="check-row">
            <input
              type="checkbox"
              checked={settings.auto_paste}
              onChange={(event) => updateSettings({ auto_paste: event.currentTarget.checked })}
            />
            Paste automatically after fixing clipboard
          </label>

          <label className="check-row">
            <input
              type="checkbox"
              checked={settings.show_window_on_start}
              onChange={(event) =>
                updateSettings({ show_window_on_start: event.currentTarget.checked })
              }
            />
            Show window when the app starts
          </label>

          <label className="check-row">
            <input
              type="checkbox"
              checked={settings.show_status_overlay}
              onChange={(event) =>
                updateSettings({ show_status_overlay: event.currentTarget.checked })
              }
            />
            Show Rust status overlay
          </label>

          <div className="shortcut-box full">
            <div className="section-head compact">
              <div>
                <h2>Global shortcut</h2>
                <p>
                  Active shortcut: {settings.shortcut_enabled ? shortcutLabel : "Disabled"}
                  {settings.shortcut_presses > 1 ? ` x${settings.shortcut_presses}` : ""}
                </p>
              </div>
            </div>

            <label className="check-row">
              <input
                type="checkbox"
                checked={settings.shortcut_enabled}
                onChange={(event) =>
                  updateSettings({ shortcut_enabled: event.currentTarget.checked })
                }
              />
              Enable global shortcut
            </label>

            <div className="shortcut-grid">
              <label className="check-row">
                <input
                  type="checkbox"
                  checked={settings.shortcut_ctrl}
                  onChange={(event) =>
                    updateSettings({ shortcut_ctrl: event.currentTarget.checked })
                  }
                />
                Ctrl
              </label>
              <label className="check-row">
                <input
                  type="checkbox"
                  checked={settings.shortcut_alt}
                  onChange={(event) =>
                    updateSettings({ shortcut_alt: event.currentTarget.checked })
                  }
                />
                Alt
              </label>
              <label className="check-row">
                <input
                  type="checkbox"
                  checked={settings.shortcut_shift}
                  onChange={(event) =>
                    updateSettings({ shortcut_shift: event.currentTarget.checked })
                  }
                />
                Shift
              </label>
              <label className="check-row">
                <input
                  type="checkbox"
                  checked={settings.shortcut_win}
                  onChange={(event) =>
                    updateSettings({ shortcut_win: event.currentTarget.checked })
                  }
                />
                Win
              </label>
            </div>

            <div className="shortcut-grid">
              <div className="field">
                <label htmlFor="shortcut-key">Key</label>
                <select
                  id="shortcut-key"
                  value={settings.shortcut_key}
                  onChange={(event) => updateSettings({ shortcut_key: event.currentTarget.value })}
                >
                  {shortcutKeys.map((key) => (
                    <option key={key} value={key}>
                      {key}
                    </option>
                  ))}
                </select>
              </div>
              <div className="field">
                <label htmlFor="shortcut-presses">Presses</label>
                <input
                  id="shortcut-presses"
                  type="number"
                  min={1}
                  max={4}
                  value={settings.shortcut_presses}
                  onChange={(event) =>
                    updateSettings({ shortcut_presses: Number(event.currentTarget.value) })
                  }
                />
              </div>
              <div className="field">
                <label htmlFor="shortcut-window">Window ms</label>
                <input
                  id="shortcut-window"
                  type="number"
                  min={150}
                  max={2000}
                  step={50}
                  value={settings.shortcut_window_ms}
                  onChange={(event) =>
                    updateSettings({ shortcut_window_ms: Number(event.currentTarget.value) })
                  }
                />
              </div>
            </div>
          </div>

          <div className="field">
            <label htmlFor="tokens">Max output tokens</label>
            <input
              id="tokens"
              type="number"
              min={1}
              max={4096}
              value={settings.max_output_tokens}
              onChange={(event) =>
                updateSettings({ max_output_tokens: Number(event.currentTarget.value) })
              }
            />
          </div>

          <div className="field">
            <label htmlFor="temperature">Temperature</label>
            <input
              id="temperature"
              type="number"
              min={0}
              max={2}
              step={0.1}
              value={settings.temperature}
              onChange={(event) =>
                updateSettings({ temperature: Number(event.currentTarget.value) })
              }
            />
          </div>
        </section>

        <section className="panel prompt-panel">
          <div className="section-head">
            <div>
              <h2>Prompt profiles</h2>
              <p>Edit safely here; FixText saves the structured state as JSON.</p>
            </div>
            <div className="button-row">
              <button onClick={addProfile}>Add</button>
              <button onClick={duplicateProfile}>Duplicate</button>
              <button onClick={deleteProfile} disabled={profiles.length < 2}>
                Delete
              </button>
            </div>
          </div>

          <div className="prompt-layout">
            <div className="profile-list">
              {profiles.map((profile) => (
                <button
                  key={profile.id}
                  className={profile.id === selectedProfileId ? "profile-tab active" : "profile-tab"}
                  onClick={() => setSelectedProfileId(profile.id)}
                >
                  {profile.title}
                </button>
              ))}
            </div>

            {selectedProfile && (
              <div className="prompt-editor">
                <div className="field">
                  <label htmlFor="profile-title">Profile name</label>
                  <input
                    id="profile-title"
                    value={selectedProfile.title}
                    onChange={(event) =>
                      updateSelectedProfile({ title: event.currentTarget.value })
                    }
                  />
                </div>
                <div className="field tall">
                  <label htmlFor="profile-prompt">System prompt</label>
                  <textarea
                    id="profile-prompt"
                    value={selectedProfile.instructions}
                    onChange={(event) =>
                      updateSelectedProfile({ instructions: event.currentTarget.value })
                    }
                  />
                </div>
              </div>
            )}
          </div>
        </section>

        <section className="panel text-panel">
          <div className="section-head">
            <div>
              <h2>Try a rewrite</h2>
              <p>Test the selected prompt before using it from the clipboard or tray.</p>
            </div>
            <div className="button-row">
              <button onClick={readClipboard}>Load clipboard</button>
              <button onClick={runOnDraft} disabled={busy || draftText.trim().length === 0}>
                Run
              </button>
              <button onClick={testAllModels} disabled={busy || draftText.trim().length === 0}>
                Test all models
              </button>
              <button onClick={copyResult} disabled={!resultText}>
                Copy result
              </button>
            </div>
          </div>
          <div className="text-grid">
            <textarea
              value={draftText}
              onChange={(event) => setDraftText(event.currentTarget.value)}
              placeholder="Paste text to test the active profile"
            />
            <textarea
              value={resultText}
              onChange={(event) => setResultText(event.currentTarget.value)}
              placeholder="Output appears here"
            />
          </div>

          {modelTestResults.length > 0 && (
            <div className="model-results">
              {modelTestResults.map((item) => (
                <article
                  key={`${item.provider}:${item.modelId}`}
                  className={item.error ? "model-result error" : "model-result"}
                >
                  <header>
                    <div>
                      <strong>{item.label}</strong>
                      <span>{providerLabels[item.provider] ?? item.provider} / {item.modelId}</span>
                    </div>
                    <small>{(item.elapsedMs / 1000).toFixed(2)}s</small>
                  </header>
                  <pre>{item.error || item.output}</pre>
                </article>
              ))}
            </div>
          )}
        </section>

        <section className="panel state-panel">
          <div className="section-head">
            <div>
              <h2>Full app state</h2>
              <p>Export or import settings, prompts, model choice, and API key as JSON.</p>
            </div>
            <div className="button-row">
              <button onClick={exportState}>Copy state</button>
              <button onClick={importState}>Import state</button>
            </div>
          </div>
          <textarea
            value={stateJson}
            onChange={(event) => setStateJson(event.currentTarget.value)}
            placeholder="Use Copy state, or paste a previous FixText state JSON here"
          />
        </section>

        <section className="activity-strip">
          {activity.length === 0 ? (
            <span>No activity yet</span>
          ) : (
            activity
              .slice()
              .reverse()
              .map((item) => (
                <span key={item.id} className={`activity ${item.kind}`}>
                  {item.message}
                </span>
              ))
          )}
        </section>
      </section>
      {settings.show_status_overlay && statusToast && (
        <div className="rust-overlay" aria-live="polite">
          <div key={statusToast.id} className={`rust-toast ${statusToast.stage}`}>
            <span>{statusToast.stage}</span>
            <strong>{statusToast.message}</strong>
          </div>
        </div>
      )}
    </main>
  );
}

export default App;
