const { ipcRenderer } = require('electron');

const store = require('../store');

const githubUsername = document.getElementById('github-username');
const githubSyncInterval = document.getElementById('github-sync-interval');
const launchAtLoginCheckbox = document.getElementById(
  'launch-at-login-checkbox',
);
const notificationCheckbox = document.getElementById('notification-checkbox');
const notificationHour = document.getElementById('notification-hour');
const notificationMinutes = document.getElementById('notification-minutes');

let typingTimer;

githubUsername.value = store.get('username');
githubSyncInterval.value = store.get('syncInterval');
launchAtLoginCheckbox.checked = store.get('autoLaunch');
notificationCheckbox.checked = store.get('notification.isEnabled');
notificationHour.value = store.get('notification.hours');
notificationMinutes.value = store.get('notification.minutes');
notificationHour.disabled = !store.get('notification.isEnabled');
notificationMinutes.disabled = !store.get('notification.isEnabled');

// Auto focus the username input if empty
if (githubUsername.value === '') {
  githubUsername.focus();
  githubUsername.classList.add('is-warning');
}

// Replace '' or '0' by '00' (e.g 20:0 -> 20:00)
if (notificationMinutes.value === '' || notificationMinutes.value === '0') {
  notificationMinutes.value = '00';
}

// Send the GitHub username via IPC
githubUsername.addEventListener('input', () => {
  clearTimeout(typingTimer);
  if (isInputEmpty(githubUsername)) {
    return;
  }
  removeRightIcon();
  githubUsername.parentElement.classList.add('is-loading');
  typingTimer = setTimeout(() => {
    ipcRenderer.send('setUsername', githubUsername.value);
  }, 1000);
});

// Add a timer before sending to the main process
githubUsername.addEventListener('input', () => {
  clearTimeout(typingTimer);
});

// Send the sync interval via IPC
githubSyncInterval.addEventListener('input', () => {
  if (isInputEmpty(githubSyncInterval)) {
    return;
  }
  const syncInterval = parseInt(githubSyncInterval.value, 10);
  if (syncInterval > 0) {
    ipcRenderer.send('setSyncInterval', syncInterval);
  } else {
    githubSyncInterval.classList.add('is-warning');
  }
});

// Send the state of 'Launch at login' checkbox via IPC
launchAtLoginCheckbox.addEventListener('change', event => {
  ipcRenderer.send('activateLaunchAtLogin', launchAtLoginCheckbox.checked);
});

// Send the Notification state via IPC
notificationCheckbox.addEventListener('change', event => {
  if (notificationCheckbox.checked) {
    notificationHour.disabled = false;
    notificationMinutes.disabled = false;
    isInputEmpty(notificationHour);
    isInputEmpty(notificationMinutes);
    ipcRenderer.send('activateNotifications', notificationCheckbox.checked);
  } else {
    notificationHour.disabled = true;
    notificationMinutes.disabled = true;
    notificationHour.classList.remove('is-warning');
    notificationMinutes.classList.remove('is-warning');
    ipcRenderer.send('activateNotifications', notificationCheckbox.checked);
  }
});

// Send the notification hour via IPC
notificationHour.addEventListener('input', () => {
  if (isInputEmpty(notificationHour)) {
    return;
  }
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

// Send the notification minutes via IPC
notificationMinutes.addEventListener('input', () => {
  if (isInputEmpty(notificationMinutes)) {
    return;
  }
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

// GitHub Username : Display Success/Error icon
ipcRenderer.on('usernameSet', (event, userExist) => {
  if (userExist) {
    githubUsername.parentElement.classList.remove('is-loading');
    githubUsername.classList.remove('is-danger');
    removeRightIcon();
    addSuccessIcon(githubUsername);
  } else {
    githubUsername.parentElement.classList.remove('is-loading');
    githubUsername.classList.add('is-danger');
    removeRightIcon();
    addErrorIcon(githubUsername);
  }
});

// Helpers functions - Add '.is-warning' when empty
function isInputEmpty(element) {
  if (element.value === '') {
    element.classList.add('is-warning');
    return true;
  } else {
    element.classList.remove('is-warning');
    return false;
  }
}

// Helpers functions - Remove the GitHub username Right Icon
function removeRightIcon() {
  const icon = document.querySelector('.icon.is-right');
  if (icon) {
    icon.parentElement.removeChild(icon);
  }
}

// Helpers functions - Add an Error Icon to the given element
function addErrorIcon(element) {
  if (!element.parentElement.classList.contains('has-icons-right')) {
    element.parentElement.classList.add('has-icons-right');
  }
  element.insertAdjacentHTML(
    'afterend',
    `
		<span class="icon is-right">
			<i class="fas fa-times"></i>
		</span>`,
  );
}

// Helpers functions - Add a Success Icon to the given element
function addSuccessIcon(element) {
  if (!element.parentElement.classList.contains('has-icons-right')) {
    element.parentElement.classList.add('has-icons-right');
  }
  element.insertAdjacentHTML(
    'afterend',
    `
		<span class="icon is-right">
			<i class="fas fa-check"></i>
		</span>`,
  );
}
