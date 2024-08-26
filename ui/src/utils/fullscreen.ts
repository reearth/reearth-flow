declare global {
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  interface Document {
    webkitExitFullscreen?(): void;
    msExitFullscreen?(): void;
  }
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  interface Element {
    webkitRequestFullscreen?(): void;
    msRequestFullscreen?(): void;
  }
}

const elem = document.documentElement;

export const checkIsFullscreen = () => {
  return document.fullscreenElement !== null;
};

export const openFullscreen = () => {
  if (elem.requestFullscreen) {
    elem.requestFullscreen();
  } else if (elem.webkitRequestFullscreen) {
    elem.webkitRequestFullscreen(); /* Safari */
  } else if (elem.msRequestFullscreen) {
    elem.msRequestFullscreen(); /* IE11 */
  }
};

export const closeFullscreen = () => {
  if (document.exitFullscreen) {
    document.exitFullscreen();
  } else if (document.webkitExitFullscreen) {
    document.webkitExitFullscreen(); /* Safari */
  } else if (document.msExitFullscreen) {
    document.msExitFullscreen(); /* IE11 */
  }
};
