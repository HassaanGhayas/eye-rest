// Pre-break Toast UI
const ringProgress = document.getElementById('ringProgress');
const secondsLeft = document.getElementById('secondsLeft');
const btnToastSnooze = document.getElementById('btnToastSnooze');
const btnToastSkip = document.getElementById('btnToastSkip');

let totalDuration = 10;

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

function showToast(data = {}) {
  totalDuration = data.seconds || 10;
  updateTimer(totalDuration);

  if (data.theme === 'light') {
    document.body.classList.add('theme-light');
    document.body.classList.remove('theme-dark');
  } else {
    document.body.classList.add('theme-dark');
    document.body.classList.remove('theme-light');
  }

  document.body.classList.add('visible');
}

function updateTimer(remaining) {
  secondsLeft.textContent = remaining;
  const percentage = Math.max(0, Math.min(100, (remaining / totalDuration) * 100));
  ringProgress.setAttribute('stroke-dasharray', `${percentage}, 100`);
}

function hideToast() {
  document.body.classList.remove('visible');
}

btnToastSnooze.addEventListener('click', () => {
  invokeTauri('snooze_break', { minutes: 5 });
  hideToast();
});

btnToastSkip.addEventListener('click', () => {
  invokeTauri('skip_break');
  hideToast();
});

if (window.__TAURI__) {
  listenTauri('prebreak-start', (event) => {
    showToast(event.payload);
  });

  listenTauri('prebreak-tick', (event) => {
    updateTimer(event.payload.remaining);
  });

  listenTauri('prebreak-end', () => {
    hideToast();
  });
}
