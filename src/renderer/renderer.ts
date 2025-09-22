import './styles.css';

const gitHubUsername = document.getElementById(
  'github-username',
) as HTMLInputElement;
const launchAtLogin = document.getElementById(
  'launch-at-login',
) as HTMLInputElement;
const reminderNotification = document.getElementById(
  'reminder-notification',
) as HTMLInputElement;
const reminderTime = document.getElementById(
  'reminder-time',
) as HTMLInputElement;
const iconTheme = document.getElementById('icon-theme') as HTMLSelectElement;

const closeWindow = () => window.close();
const savePreferences = () => {
  // @ts-expect-error see preload.ts
  window.streaker.savePreferences({
    username: gitHubUsername.value,
    launchAtLogin: launchAtLogin.checked,
    reminderEnabled: reminderNotification.checked,
    reminderTime: reminderTime.value,
    iconTheme: iconTheme.value,
  });
  closeWindow();
};

document.getElementById('cancel-button').addEventListener('click', closeWindow);

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') closeWindow();
  if (event.key === 'Enter') savePreferences();
});

document
  .getElementById('save-button')
  .addEventListener('click', () => savePreferences());

// @ts-expect-error see preload.ts
const initialPreferences = window.streaker.initialPreferences();

gitHubUsername.value = initialPreferences.username || '';
launchAtLogin.checked = initialPreferences.launchAtLogin === 'true';
reminderNotification.checked = initialPreferences.reminderEnabled === 'true';
reminderTime.value = initialPreferences.reminderTime;
iconTheme.value = initialPreferences.iconTheme;

console.log('Preferences:', initialPreferences);
