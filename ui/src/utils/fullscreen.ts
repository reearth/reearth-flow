declare global {
  interface Document {
    webkitExitFullscreen?(): void;
    msExitFullscreen?(): void;
  }
  interface Element {
    webkitRequestFullscreen?(): void;
    msRequestFullscreen?(): void;
  }
}

const elem = document.documentElement;

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
