import { join } from 'path';

export interface IconSet {
  contributed: string;
  error: string;
  loading: string;
  pending: string;
  streaking: string;
}

export interface IconSets {
  [iconSet: string]: IconSet;
}

let iconSets: IconSets;

switch (process.platform) {
  case 'darwin':
    iconSets = {
      octocat: {
        contributed: join(
          __dirname,
          './icons/darwin/octocat/contributedTemplate.png',
        ),
        error: join(__dirname, 'icons/darwin/octocat/errorTemplate.png'),
        loading: join(__dirname, 'icons/darwin/octocat/loadingTemplate.png'),
        pending: join(__dirname, 'icons/darwin/octocat/pendingTemplate.png'),
        streaking: join(
          __dirname,
          'icons/darwin/octocat/streakingTemplate.png',
        ),
      },
      flame: {
        contributed: join(
          __dirname,
          'icons/darwin/flame/contributedTemplate.png',
        ),
        error: join(__dirname, 'icons/darwin/flame/errorTemplate.png'),
        loading: join(__dirname, 'icons/darwin/flame/loadingTemplate.png'),
        pending: join(__dirname, 'icons/darwin/flame/pendingTemplate.png'),
        streaking: join(__dirname, 'icons/darwin/flame/streakingTemplate.png'),
      },
    };
    break;
  case 'win32':
    iconSets = {
      octocat: {
        contributed: join(__dirname, 'icons/win32/octocat/contributed.ico'),
        error: join(__dirname, 'icons/win32/octocat/error.ico'),
        loading: join(__dirname, 'icons/win32/octocat/loading.ico'),
        pending: join(__dirname, 'icons/win32/octocat/pending.ico'),
        streaking: join(__dirname, 'icons/win32/octocat/streaking.ico'),
      },
      flame: {
        contributed: join(__dirname, 'icons/win32/flame/contributed.ico'),
        error: join(__dirname, 'icons/win32/flame/error.ico'),
        loading: join(__dirname, 'icons/win32/flame/loading.ico'),
        pending: join(__dirname, 'icons/win32/flame/pending.ico'),
        streaking: join(__dirname, 'icons/win32/flame/streaking.ico'),
      },
    };
    break;
  // case 'linux':
  //   break;
  default:
    console.error(`Platform "${process.platform}" is not currently supported!`);
    break;
}

export default iconSets;
