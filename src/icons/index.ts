import { join } from 'node:path';
import { type NativeImage, app, nativeImage } from 'electron';

export interface Icon {
  contributed: NativeImage;
  error: NativeImage;
  pending: NativeImage;
  streaking: NativeImage;
}

export interface Icons {
  flame: Icon;
  tile?: Icon;
  octocat: Icon;
}

export let icons: Icons;

const appPath = app.getAppPath().replace('app.asar', '');

// macOS Flame
const macosFlameContributed = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/flame/contributed@2x.png'),
);
macosFlameContributed.setTemplateImage(true);
const macosFlameError = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/flame/error@2x.png'),
);
macosFlameError.setTemplateImage(true);
const macosFlamePending = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/flame/pending@2x.png'),
);
macosFlamePending.setTemplateImage(true);
const macosFlameStreaking = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/flame/streaking@2x.png'),
);
macosFlameStreaking.setTemplateImage(true);

// macOS Tile
const macosTileContributed = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/tile/contributed@2x.png'),
);
macosTileContributed.setTemplateImage(true);
const macosTileError = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/tile/error@2x.png'),
);
macosTileError.setTemplateImage(true);
const macosTilePending = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/tile/pending@2x.png'),
);
macosTilePending.setTemplateImage(true);
const macosTileStreaking = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/tile/streaking@2x.png'),
);
macosTileStreaking.setTemplateImage(true);

// macOS Octocat
const macosOctocatContributed = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/octocat/contributed@2x.png'),
);
macosOctocatContributed.setTemplateImage(true);
const macosOctocatError = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/octocat/error@2x.png'),
);
macosOctocatError.setTemplateImage(true);
const macosOctocatPending = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/octocat/pending@2x.png'),
);
macosOctocatPending.setTemplateImage(true);
const macosOctocatStreaking = nativeImage.createFromPath(
  join(appPath, 'src/icons/macos/octocat/streaking@2x.png'),
);
macosOctocatStreaking.setTemplateImage(true);

// Windows Flame
const windowsFlameContributed = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/flame/contributed.ico'),
);
const windowsFlameError = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/flame/error.ico'),
);
const windowsFlamePending = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/flame/pending.ico'),
);
const windowsFlameStreaking = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/flame/streaking.ico'),
);

// Windows Tile
const windowsTileContributed = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/tile/contributed.ico'),
);
const windowsTileError = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/tile/error.ico'),
);
const windowsTilePending = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/tile/pending.ico'),
);
const windowsTileStreaking = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/tile/streaking.ico'),
);

// Windows Octocat
const windowsOctocatContributed = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/octocat/contributed.ico'),
);
const windowsOctocatError = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/octocat/error.ico'),
);
const windowsOctocatPending = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/octocat/pending.ico'),
);
const windowsOctocatStreaking = nativeImage.createFromPath(
  join(appPath, 'src/icons/windows/octocat/streaking.ico'),
);

switch (process.platform) {
  case 'darwin':
    icons = {
      flame: {
        contributed: macosFlameContributed,
        error: macosFlameError,
        pending: macosFlamePending,
        streaking: macosFlameStreaking,
      },
      tile: {
        contributed: macosTileContributed,
        error: macosTileError,
        pending: macosTilePending,
        streaking: macosTileStreaking,
      },
      octocat: {
        contributed: macosOctocatContributed,
        error: macosOctocatError,
        pending: macosOctocatPending,
        streaking: macosOctocatStreaking,
      },
    };
    break;
  case 'win32':
    icons = {
      flame: {
        contributed: windowsFlameContributed,
        error: windowsFlameError,
        pending: windowsFlamePending,
        streaking: windowsFlameStreaking,
      },
      tile: {
        contributed: windowsTileContributed,
        error: windowsTileError,
        pending: windowsTilePending,
        streaking: windowsTileStreaking,
      },
      octocat: {
        contributed: windowsOctocatContributed,
        error: windowsOctocatError,
        pending: windowsOctocatPending,
        streaking: windowsOctocatStreaking,
      },
    };
    break;
  case 'linux':
    icons = {
      flame: {
        contributed: macosFlameContributed,
        error: macosFlameError,
        pending: macosFlamePending,
        streaking: macosFlameStreaking,
      },
      tile: {
        contributed: macosTileContributed,
        error: macosTileError,
        pending: macosTilePending,
        streaking: macosTileStreaking,
      },
      octocat: {
        contributed: macosOctocatContributed,
        error: macosOctocatError,
        pending: macosOctocatPending,
        streaking: macosOctocatStreaking,
      },
    };
    break;
  default: {
    throw new Error(
      `Platform "${process.platform}" is not currently supported!`,
    );
  }
}
