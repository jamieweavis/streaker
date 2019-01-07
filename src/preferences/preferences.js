const { ipcRenderer } = require('electron');

const store = require('../store');

const githubUsername = document.getElementById('github-username');
const githubUsernameStatus = document.getElementById('github-username-status');
const githubSyncInterval = document.getElementById('github-sync-interval');
const launchAtLoginCheckbox = document.getElementById(
  'launch-at-login-checkbox',
);
const notificationCheckbox = document.getElementById('notification-checkbox');
const notificationHour = document.getElementById('notification-hour');
const notificationMinutes = document.getElementById('notification-minutes');

githubUsername.value = store.get('username');
githubSyncInterval.value = store.get('syncInterval');
launchAtLoginCheckbox.checked = store.get('autoLaunch');
notificationCheckbox.checked = store.get('notification.isEnabled');
notificationHour.value = store.get('notification.hours');
notificationMinutes.value = store.get('notification.minutes');
notificationHour.disabled = !store.get('notification.isEnabled');
notificationMinutes.disabled = !store.get('notification.isEnabled');

if (!githubUsername.value) {
  githubUsername.focus();
  isInvalid(githubUsername);
}

if (notificationMinutes.value === '' || notificationMinutes.value === '0') {
  notificationMinutes.value = '00';
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
  notificationHour.disabled = !notificationCheckbox.checked;
  notificationMinutes.disabled = !notificationCheckbox.checked;
  ipcRenderer.send('activateNotifications', notificationCheckbox.checked);
  isInvalid(notificationHour);
  isInvalid(notificationMinutes);
});

notificationHour.addEventListener('input', () => {
  if (isInvalid(notificationHour)) return;
  const hours = parseInt(notificationHour.value, 10);
  if (hours >= 0 && hours <= 23) {
    ipcRenderer.send('setNotificationTime', {
      hours: hours,
      minutes: store.get('notification.minutes'),
    });
  } else {
    notificationHour.classList.add('is-warning');
  }
});

notificationMinutes.addEventListener('input', () => {
  if (isInvalid(notificationMinutes)) return;
  const minutes = parseInt(notificationMinutes.value, 10);
  if (minutes >= 0 && minutes <= 59) {
    ipcRenderer.send('setNotificationTime', {
      hours: store.get('notification.hours'),
      minutes: minutes,
    });
  } else {
    notificationMinutes.classList.add('is-warning');
  }
});

ipcRenderer.on('usernameSet', (event, userExist) => {
  githubUsername.parentElement.classList.remove('is-loading');
  githubUsername.classList.toggle('is-danger', !userExist);
  githubUsernameStatus.classList.toggle('fa-check', userExist);
  githubUsernameStatus.classList.toggle('fa-times', !userExist);
});

function isInvalid(input) {
  const valid = !input.value && !input.disabled;
  input.classList.toggle('is-warning', valid);
  return valid;
}
