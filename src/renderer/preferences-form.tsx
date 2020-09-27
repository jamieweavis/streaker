import { remote, ipcRenderer } from 'electron';
import React, { useState } from 'react';
import { MarkGithubIcon, ClockIcon } from '@primer/octicons-react';
import {
  TextInput,
  Text,
  Avatar,
  Button,
  ButtonPrimary,
} from '@primer/components';

import { PreferencesSavedArgs } from '../common/types';
import store from '../common/store';

const closeWindow = (): void => {
  const window = remote.getCurrentWindow();
  window.close();
};

const PreferencesForm = (): JSX.Element => {
  const [username, setUsername] = useState(store.get('username'));
  const [syncInterval, setSyncInterval] = useState(store.get('syncInterval'));
  const [launchAtLogin, setLaunchAtLogin] = useState(
    store.get('launchAtLogin'),
  );
  const [notificationEnabled, setNotificationEnabled] = useState(
    store.get('notificationEnabled'),
  );
  const [notificationTime, setNotificationTime] = useState(
    store.get('notificationTime'),
  );

  return (
    <form style={{ paddingLeft: 15 }}>
      <label htmlFor="github-username">
        <Text
          fontWeight="bold"
          fontSize="14px"
          as="p"
          style={{ marginBottom: 5 }}
        >
          GitHub username
        </Text>
      </label>
      <TextInput
        id="github-username"
        icon={MarkGithubIcon}
        placeholder="GitHub Username"
        value={username}
        width={175}
        onChange={(event): void => {
          const newUsername = event?.target?.value || '';
          setUsername(newUsername);
        }}
      />
      <Avatar
        style={{ marginLeft: 10 }}
        square
        src={`https://avatars.githubusercontent.com/${
          username ? username : '_' // If empty default to placeholder
        }`}
      />
      <label htmlFor="sync-interval">
        <Text
          fontWeight="bold"
          fontSize="14px"
          as="p"
          style={{ marginBottom: 5 }}
        >
          Sync interval
        </Text>
      </label>
      <Text fontSize="14px">Sync every </Text>
      <TextInput
        id="sync-interval"
        placeholder="15"
        value={syncInterval === 15 ? '' : syncInterval}
        width={70}
        type="number"
        onChange={(event): void => {
          let newSyncInterval = event?.target?.value;
          if (newSyncInterval === '') {
            newSyncInterval = '15';
          }
          setSyncInterval(parseInt(newSyncInterval));
        }}
      />
      <Text fontSize="14px"> minutes</Text>
      <label htmlFor="launch-at-login">
        <Text
          fontWeight="bold"
          fontSize="14px"
          as="p"
          style={{ marginBottom: 5 }}
        >
          <input
            id="launch-at-login"
            type="checkbox"
            checked={launchAtLogin}
            onChange={(event): void => {
              const newAutoLaunch = !!event?.target?.checked || false;
              setLaunchAtLogin(newAutoLaunch);
            }}
          />
          <Text> Launch at login</Text>
        </Text>
      </label>
      <label htmlFor="notification-enabled">
        <Text
          fontWeight="bold"
          fontSize="14px"
          as="p"
          style={{ marginBottom: 5 }}
        >
          <input
            id="notification-enabled"
            type="checkbox"
            checked={notificationEnabled}
            onChange={(event): void => {
              const newNotificationEnabled = !!event?.target?.checked || false;
              setNotificationEnabled(newNotificationEnabled);
            }}
          />
          <Text> Enable reminder notification</Text>
        </Text>
      </label>
      <Text color="grey" fontSize="12px" as="p">
        Get a reminder so you don&apos;t lose your streak! 🔥
      </Text>
      <Text
        fontWeight="bold"
        fontSize="14px"
        as="p"
        style={{ marginBottom: 5 }}
      >
        Reminder time
      </Text>
      <Text fontSize="14px">Remind me at </Text>
      <TextInput
        icon={ClockIcon}
        placeholder="Reminder Time"
        type="time"
        value={notificationTime}
        disabled={!notificationEnabled}
        onChange={(event): void => {
          const newReminderTime = event?.target?.value || '';
          setNotificationTime(newReminderTime);
        }}
      />
      <div style={{ position: 'absolute', bottom: 15, right: 15 }}>
        <Button onClick={closeWindow} style={{ marginRight: 10 }}>
          Cancel
        </Button>
        <ButtonPrimary
          onClick={(): void => {
            ipcRenderer.send('preferences-saved', {
              username,
              syncInterval,
              launchAtLogin,
              notificationEnabled,
              notificationTime,
            } as PreferencesSavedArgs);
            closeWindow();
          }}
        >
          Save
        </ButtonPrimary>
      </div>
    </form>
  );
};

export default PreferencesForm;
