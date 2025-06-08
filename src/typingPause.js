let typingPauseEnabled = false;
let typingPauseTimeout = null;
let typingPauseDelay = 1000;
//let lastEditor = null;
let lastHandler = null;

export function enableTypingPauseDetection(shouldTrigger, delayMs = 1000, callback) {
  typingPauseDelay = delayMs;
  typingPauseEnabled = true;

  if (lastHandler) {
    document.removeEventListener('keydown', lastHandler);
  }

  lastHandler = function handleKeydown() {
    //if (!typingPauseEnabled) return;
    //if (!shouldTrigger()) return; // predicate..we may want to temporarily disable the pause detction and subsequent callback
    if (typingPauseTimeout) clearTimeout(typingPauseTimeout);
    typingPauseTimeout = setTimeout(() => {
      if (typingPauseEnabled && shouldTrigger()) {
        callback();
      }
    }, typingPauseDelay);
  };

  document.addEventListener('keydown', lastHandler);
}

export function disableTypingPauseDetection() {
  typingPauseEnabled = false;
  if (typingPauseTimeout) clearTimeout(typingPauseTimeout);
  if (lastHandler) {
    document.removeEventListener('keydown', lastHandler);
  }
}