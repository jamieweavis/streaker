import { Menu } from 'electron';

export interface CreateMenuOptions {
  stats?: any;
  username: string;
  onPreferencesClick: Function;
  onAboutClick: Function;
  onProfileClick: Function;
  onFeedbackClick: Function;
  onQuitClick: Function;
  requestContributionData: Function;
}

export const createMenu = ({
  stats,
  username,
  onPreferencesClick,
  onAboutClick,
  onProfileClick,
  onFeedbackClick,
  onQuitClick,
  requestContributionData,
}: CreateMenuOptions): Menu => {
  return Menu.buildFromTemplate([
    {
      label: username || 'Set username...',
      enabled: !username,
      click: (): void => onPreferencesClick(),
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
      click: (): void => requestContributionData(),
    },
    {
      label: 'Open GitHub Profile...',
      enabled: !!username,
      accelerator: 'CmdOrCtrl+O',
      click: (): void => onProfileClick(),
    },
    { type: 'separator' },
    {
      label: 'Preferences...',
      accelerator: 'CmdOrCtrl+,',
      click: (): void => onPreferencesClick(),
    },
    {
      label: 'About Streaker...',
      click: (): void => onAboutClick(),
    },
    {
      label: 'Feedback && Support...',
      click: (): void => onFeedbackClick(),
    },
    { type: 'separator' },
    {
      label: 'Quit Streaker',
      accelerator: 'CmdOrCtrl+Q',
      click: (): void => onQuitClick(),
    },
  ]);
};
