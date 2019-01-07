const { ipcRenderer } = require('electron');

const store = require('../store');

const githubUsername = document.getElementById('github-username');
const githubUsernameStatus = document.getElementById('github-username-status');
const githubSyncInterval = document.getElementById('github-sync-interval');
const launchAtLoginCheckbox = document.getElementById(
  'launch-at-login-checkbox',
);
const notificationCheckbox = document.getElementById('notification-checkbox');
const notificationTime = document.getElementById('notification-time');

githubUsername.value = store.get('username');
githubSyncInterval.value = store.get('syncInterval');
launchAtLoginCheckbox.checked = store.get('autoLaunch');
notificationCheckbox.checked = store.get('notification.isEnabled');
notificationTime.value = store.get('notification.time');
notificationTime.disabled = !store.get('notification.isEnabled');

if (!githubUsername.value) {
  githubUsername.focus();
  isInvalid(githubUsername);
}

let typingTimer;
githubUsername.addEventListener('input', () => {
  githubUsername.parentElement.classList.remove('is-loading');
  clearTimeout(typingTimer);
  isInvalid(githubUsername);

  githubUsername.parentElement.classList.add('is-loading');
  githubUsernameStatus.classList.remove('fa-check');
  githubUsernameStatus.classList.remove('fa-times');

  typingTimer = setTimeout(() => {
    ipcRenderer.send('setUsername', githubUsername.value);
  }, 1000);
});

githubSyncInterval.addEventListener('input', () => {
  if (isInvalid(githubSyncInterval)) return;
  const syncInterval = parseInt(githubSyncInterval.value, 10);
  if (syncInterval > 0) {
    ipcRenderer.send('setSyncInterval', syncInterval);
  } else {
    githubSyncInterval.classList.add('is-warning');
  }
});

launchAtLoginCheckbox.addEventListener('change', event => {
  ipcRenderer.send('activateLaunchAtLogin', launchAtLoginCheckbox.checked);
});

notificationCheckbox.addEventListener('change', event => {
  notificationTime.disabled = !notificationCheckbox.checked;
  ipcRenderer.send('activateNotifications', notificationCheckbox.checked);
});

notificationTime.addEventListener('input', () => {
  if (isInvalid(notificationTime)) return;
  ipcRenderer.send('setNotificationTime', notificationTime.value);
});

ipcRenderer.on('usernameSet', (event, userExists) => {
  githubUsername.parentElement.classList.remove('is-loading');
  githubUsername.classList.toggle('is-danger', !userExists);
  githubUsernameStatus.classList.toggle('fa-check', userExists);
  githubUsernameStatus.classList.toggle('fa-times', !userExists);
});

function isInvalid(input) {
  const valid = !input.value && !input.disabled;
  input.classList.toggle('is-warning', valid);
  return valid;
}
