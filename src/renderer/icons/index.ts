import octocatIcons from '@renderer/icons/octocat';
import flameIcons from '@renderer/icons/flame';

export interface IconSet {
  name: string;
  displayName: string;
  icons: {
    contributed: string;
    error: string;
    loading: string;
    pending: string;
    streaking: string;
  };
}

const iconSets: IconSet[] = [
  {
    name: 'octocat',
    displayName: 'Octocat',
    icons: { ...octocatIcons },
  },
  {
    name: 'flame',
    displayName: 'Flame',
    icons: { ...flameIcons },
  },
];

export { iconSets };
