import type { GitHubStats } from 'contribution';
import { Menu, app, shell } from 'electron';

import pjson from '../../package.json';

export interface CreateTrayMenuOptions {
  username?: string;
  stats?: GitHubStats;
  createPreferencesWindow: () => void;
  requestContributionData: () => void;
  isPreferencesWindowOpen: () => boolean;
  focusPreferencesWindow: () => void;
}

export const createTrayMenu = ({
  username,
  stats,
  createPreferencesWindow,
  requestContributionData,
  isPreferencesWindowOpen,
  focusPreferencesWindow,
}: CreateTrayMenuOptions): Menu => {
  const createOrFocusPreferencesWindow = () => {
    if (!isPreferencesWindowOpen()) return createPreferencesWindow();
    focusPreferencesWindow();
  };

  return Menu.buildFromTemplate([
    {
      label: username || 'Set GitHub username...',
      enabled: !username,
      click: createOrFocusPreferencesWindow,
    },
    { type: 'separator' },
    { label: 'Streak:', enabled: false },
    {
      label: `    Best:\t\t${stats?.streak?.best || 0}`,
      enabled: false,
    },
    {
      label: `    Current:\t${stats?.streak?.current || 0}`,
      enabled: false,
    },
    {
      label: `    Previous:\t${stats?.streak?.previous || 0}`,
      enabled: false,
    },
    { label: 'Contributions:', enabled: false },
    {
      label: `    Best:\t\t${stats?.contributions?.best || 0}`,
      enabled: false,
    },
    {
      label: `    Today:\t\t${stats?.contributions?.current || 0}`,
      enabled: false,
    },
    {
      label: `    Total:\t\t${stats?.contributions?.total || 0}`,
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
      click: () => shell.openExternal(`https://github.com/${username}`),
    },
    { type: 'separator' },
    {
      label: 'Preferences...',
      accelerator: 'CmdOrCtrl+,',
      click: createOrFocusPreferencesWindow,
    },
    {
      label: 'About Streaker...',
      click: () => shell.openExternal(pjson.homepage),
    },
    {
      label: 'Feedback && Support...',
      click: () => shell.openExternal(pjson.bugs.url),
    },
    { type: 'separator' },
    {
      label: 'Quit Streaker',
      accelerator: 'CmdOrCtrl+Q',
      click: () => app.quit(),
    },
  ]);
};
