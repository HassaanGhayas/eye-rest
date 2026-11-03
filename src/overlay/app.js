// EyeRest Overlay UI Controller
const countdownEl = document.getElementById('countdown');
const messageEl = document.getElementById('breakMessage');
const submessageEl = document.getElementById('breakSubmessage');
const actionsContainer = document.getElementById('actionsContainer');
const btnSkip = document.getElementById('btnSkip');
const btnSnooze = document.getElementById('btnSnooze');

// Gentle chime synthesis using Web Audio API (warm singing bowl frequency ~432Hz with harmonic decay)
class CalmChime {
  constructor() {
    this.ctx = null;
  }

  ensureContext() {
    if (!this.ctx) {
      const AudioCtx = window.AudioContext || window.webkitAudioContext;
      if (AudioCtx) this.ctx = new AudioCtx();
    }
    if (this.ctx && this.ctx.state === 'suspended') {
      this.ctx.resume();
    }
  }

  play() {
    try {
      this.ensureContext();
      if (!this.ctx) return;
      const now = this.ctx.currentTime;

      // Base sine oscillator for fundamental warm tone
      const osc1 = this.ctx.createOscillator();
      const gain1 = this.ctx.createGain();
      osc1.type = 'sine';
      osc1.frequency.setValueAtTime(432, now); // A4 warm tuning

      // Gentle attack & long soft exponential decay
      gain1.gain.setValueAtTime(0.0001, now);
      gain1.gain.exponentialRampToValueAtTime(0.2, now + 0.08);
      gain1.gain.exponentialRampToValueAtTime(0.0001, now + 2.8);

      // Warm octave harmonic
      const osc2 = this.ctx.createOscillator();
      const gain2 = this.ctx.createGain();
      osc2.type = 'sine';
      osc2.frequency.setValueAtTime(864, now);
      gain2.gain.setValueAtTime(0.0001, now);
      gain2.gain.exponentialRampToValueAtTime(0.06, now + 0.06);
      gain2.gain.exponentialRampToValueAtTime(0.0001, now + 2.2);

      osc1.connect(gain1);
      gain1.connect(this.ctx.destination);
      osc2.connect(gain2);
      gain2.connect(this.ctx.destination);

      osc1.start(now);
      osc2.start(now);
      osc1.stop(now + 3.0);
      osc2.stop(now + 2.5);
    } catch (e) {
      console.warn("Chime playback error:", e);
    }
  }
}

const chime = new CalmChime();

// Tauri IPC helpers
function invokeTauri(cmd, payload = {}) {
  if (window.__TAURI__ && window.__TAURI__.core) {
    return window.__TAURI__.core.invoke(cmd, payload);
  } else if (window.__TAURI__ && window.__TAURI__.invoke) {
    return window.__TAURI__.invoke(cmd, payload);
  }
  return Promise.resolve();
}

function listenTauri(event, handler) {
  if (window.__TAURI__ && window.__TAURI__.event) {
    return window.__TAURI__.event.listen(event, handler);
  }
}

// Window visibility & animations
function showOverlay(data = {}) {
  const seconds = data.seconds || 20;
  const isStrict = Boolean(data.strict);
  const playSound = data.sound !== false;

  countdownEl.textContent = seconds;
  if (isStrict) {
    actionsContainer.classList.add('strict-mode');
  } else {
    actionsContainer.classList.remove('strict-mode');
  }

  if (data.theme === 'light') {
    document.body.classList.add('theme-light');
    document.body.classList.remove('theme-dark');
  } else {
    document.body.classList.add('theme-dark');
    document.body.classList.remove('theme-light');
  }

  document.body.classList.add('visible');

  if (playSound) {
    chime.play();
  }
}

function updateCountdown(remaining) {
  countdownEl.textContent = Math.max(0, remaining);
}

function hideOverlay() {
  document.body.classList.remove('visible');
}

// Actions
btnSkip.addEventListener('click', () => {
  invokeTauri('skip_break');
});

btnSnooze.addEventListener('click', () => {
  invokeTauri('snooze_break', { minutes: 5 });
});

window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && !actionsContainer.classList.contains('strict-mode')) {
    invokeTauri('skip_break');
  }
});

// Setup Tauri event listeners if inside Tauri WebView
if (window.__TAURI__) {
  listenTauri('break-start', (event) => {
    showOverlay(event.payload);
  });

  listenTauri('break-tick', (event) => {
    updateCountdown(event.payload.remaining);
  });

  listenTauri('break-end', () => {
    hideOverlay();
  });
}
