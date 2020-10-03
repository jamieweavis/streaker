import octocatContributedIcon from './octocat/contributed.svg';
import octocatErrorIcon from './octocat/error.svg';
import octocatLoadingIcon from './octocat/loading.svg';
import octocatPendingIcon from './octocat/pending.svg';

interface IconSet {
  contributed: string;
  error: string;
  loading: string;
  pending: string;
}

interface Icons {
  [key: string]: IconSet;
}

enum IconSets {
  octocat = 'Octocat',
}

const octocat: IconSet = {
  contributed: octocatContributedIcon,
  error: octocatErrorIcon,
  loading: octocatLoadingIcon,
  pending: octocatPendingIcon,
};

const icons: Icons = {
  octocat,
};

export { icons, IconSets };
