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

githubUsername.value = store.get('username') || '';
githubSyncInterval.value = store.get('syncInterval') || '';
notificationHour.value = store.get('notification.hours') || '';
notificationMinutes.value = store.get('notification.minutes') || '';

if (githubUsername.value === '') {
  githubUsername.focus();
  githubUsername.classList.add('is-warning');
}

if (store.get('autoLaunch')) {
  launchAtLoginCheckbox.checked = true;
}

if (!store.get('notification.isEnabled')) {
  notificationHour.disabled = true;
  notificationMinutes.disabled = true;
} else {
  notificationCheckbox.checked = true;
}

if (notificationMinutes.value === '' || notificationMinutes.value === '0') {
  notificationMinutes.value = '00';
}

githubUsername.addEventListener('keyup', () => {
  clearTimeout(typingTimer);
  if (isInputEmpty(githubUsername)) {
    return;
  }
  removeIconRight();
  githubUsername.parentElement.classList.add('is-loading');
  typingTimer = setTimeout(() => {
    ipcRenderer.send('setUsername', githubUsername.value);
  }, 1000);
});

githubUsername.addEventListener('keydown', () => {
  clearTimeout(typingTimer);
});

githubSyncInterval.addEventListener('keyup', () => {
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

launchAtLoginCheckbox.addEventListener('change', event => {
  ipcRenderer.send('activateLaunchAtLogin', launchAtLoginCheckbox.checked);
});

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

notificationHour.addEventListener('keyup', () => {
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

notificationMinutes.addEventListener('keyup', () => {
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

ipcRenderer.on('usernameSet', (event, userExist) => {
  const iconRight = document.querySelector('.icon.is-right');
  if (userExist) {
    console.log('usernameSet : ', store.get('username'));
    githubUsername.parentElement.classList.remove('is-loading');
    githubUsername.classList.remove('is-danger');
    removeIconRight();
    addSuccessIcon(githubUsername);
  } else {
    githubUsername.parentElement.classList.remove('is-loading');
    githubUsername.classList.add('is-danger');
    removeIconRight();
    addErrorIcon(githubUsername);
  }
});

ipcRenderer.on('syncIntervalSet', () => {
  console.log('syncInterval : ', store.get('syncInterval'));
});

ipcRenderer.on('activateLaunchAtLoginSet', () => {
  console.log('autoLaunch : ', store.get('autoLaunch'));
});

ipcRenderer.on('activateNotificationsSet', () => {
  console.log('Notification state : ', store.get('notification.isEnabled'));
});

ipcRenderer.on('NotificationTimeSet', () => {
  console.log('NotificationTimeSet : ', store.get('notification.time'));
});

function isInputEmpty(element) {
  if (element.value === '') {
    element.classList.add('is-warning');
    return true;
  } else {
    element.classList.remove('is-warning');
    return false;
  }
}

function removeIconRight() {
  const icon = document.querySelector('.icon.is-right');
  if (icon) {
    icon.parentElement.removeChild(icon);
  }
}

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
