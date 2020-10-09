import { app, Menu, shell } from 'electron';
import { GitHubStats } from 'contribution';

import store from '@common/store';
import pjson from '../../package.json';

export interface CreateMenuOptions {
  username?: string;
  stats?: GitHubStats;
  createPreferencesWindow: () => void;
  requestContributionData: () => void;
  isPreferencesWindowOpen: () => boolean;
  focusPreferencesWindow: () => void;
}

export const createMenu = ({
  username,
  stats,
  createPreferencesWindow,
  requestContributionData,
  isPreferencesWindowOpen,
  focusPreferencesWindow,
}: CreateMenuOptions): Menu => {
  const onPreferencesClick = (): void => {
    if (!isPreferencesWindowOpen()) return createPreferencesWindow();
    focusPreferencesWindow();
  };

  return Menu.buildFromTemplate([
    {
      label: username || 'Set GitHub username...',
      enabled: !username,
      click: onPreferencesClick,
    },
    { type: 'separator' },
    { label: 'Streak:', enabled: false },
    { label: `    Best: ${stats?.streak?.best || 0}`, enabled: false },
    {
      label: `    Current: ${stats?.streak?.current || 0}`,
      enabled: false,
    },
    { label: 'Contributions:', enabled: false },
    {
      label: `    Best: ${stats?.contributions?.best || 0}`,
      enabled: false,
    },
    {
      label: `    Current: ${stats?.contributions?.current || 0}`,
      enabled: false,
    },
    {
      label: `    Total: ${stats?.contributions?.total || 0}`,
      enabled: false,
    },
    { type: 'separator' },
    {
      label: 'Reload',
      enabled: !!username,
      accelerator: 'CmdOrCtrl+R',
      click: requestContributionData,
    },
    {
      label: 'Open GitHub Profile...',
      enabled: !!username,
      accelerator: 'CmdOrCtrl+O',
      click: (): Promise<void> =>
        shell.openExternal(`https://github.com/${store.get('username')}`),
    },
    { type: 'separator' },
    {
      label: 'Preferences...',
      accelerator: 'CmdOrCtrl+,',
      click: onPreferencesClick,
    },
    {
      label: 'About Streaker...',
      click: (): Promise<void> => shell.openExternal(pjson.homepage),
    },
    {
      label: 'Feedback && Support...',
      click: (): Promise<void> => shell.openExternal(pjson.bugs.url),
    },
    { type: 'separator' },
    {
      label: 'Quit Streaker',
      accelerator: 'CmdOrCtrl+Q',
      click: (): void => app.quit(),
    },
  ]);
};
