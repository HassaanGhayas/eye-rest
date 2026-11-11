// EyeRest Settings Controller
const workIntervalInput = document.getElementById('workInterval');
const breakDurationInput = document.getElementById('breakDuration');
const soundEnabledInput = document.getElementById('soundEnabled');
const strictModeInput = document.getElementById('strictMode');
const launchOnStartupInput = document.getElementById('launchOnStartup');
const themeSelect = document.getElementById('themeSelect');
const saveStatus = document.getElementById('saveStatus');
const btnClose = document.getElementById('btnClose');

function invokeTauri(cmd, payload = {}) {
  if (window.__TAURI__ && window.__TAURI__.core) {
    return window.__TAURI__.core.invoke(cmd, payload);
  } else if (window.__TAURI__ && window.__TAURI__.invoke) {
    return window.__TAURI__.invoke(cmd, payload);
  }
  return Promise.resolve();
}

function applyTheme(theme) {
  if (theme === 'light') {
    document.body.classList.add('theme-light');
    document.body.classList.remove('theme-dark');
  } else if (theme === 'dark') {
    document.body.classList.add('theme-dark');
    document.body.classList.remove('theme-light');
  } else {
    // System preference
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (prefersDark) {
      document.body.classList.add('theme-dark');
      document.body.classList.remove('theme-light');
    } else {
      document.body.classList.add('theme-light');
      document.body.classList.remove('theme-dark');
    }
  }
}

async function loadSettings() {
  try {
    const settings = await invokeTauri('get_settings');
    if (settings) {
      workIntervalInput.value = settings.work_interval_mins ?? 20;
      breakDurationInput.value = settings.break_duration_secs ?? 20;
      soundEnabledInput.checked = settings.sound_enabled ?? true;
      strictModeInput.checked = settings.strict_mode ?? false;
      launchOnStartupInput.checked = settings.launch_on_startup ?? false;
      themeSelect.value = settings.theme ?? 'system';
      applyTheme(settings.theme ?? 'system');
    }
  } catch (err) {
    console.error("Failed to load settings:", err);
  }
}

let saveTimeout = null;
async function persistSettings() {
  const settings = {
    work_interval_mins: Math.max(1, parseInt(workIntervalInput.value, 10) || 20),
    break_duration_secs: Math.max(5, parseInt(breakDurationInput.value, 10) || 20),
    sound_enabled: soundEnabledInput.checked,
    strict_mode: strictModeInput.checked,
    launch_on_startup: launchOnStartupInput.checked,
    theme: themeSelect.value
  };

  applyTheme(settings.theme);

  try {
    await invokeTauri('save_settings', { settings });
    saveStatus.textContent = "Saved";
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      saveStatus.textContent = "Settings saved automatically";
    }, 2000);
  } catch (err) {
    console.error("Failed to save settings:", err);
    saveStatus.textContent = "Error saving settings";
  }
}

// Bind event listeners
[workIntervalInput, breakDurationInput].forEach(el => {
  el.addEventListener('change', persistSettings);
  el.addEventListener('input', () => {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(persistSettings, 600);
  });
});

[soundEnabledInput, strictModeInput, launchOnStartupInput, themeSelect].forEach(el => {
  el.addEventListener('change', persistSettings);
});

btnClose.addEventListener('click', () => {
  invokeTauri('close_settings_window');
});

if (window.__TAURI__ && window.__TAURI__.event) {
  window.__TAURI__.event.listen('refresh-settings', () => {
    loadSettings();
  });
}

window.addEventListener('DOMContentLoaded', loadSettings);
