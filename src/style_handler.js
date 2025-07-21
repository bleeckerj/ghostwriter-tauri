let currentParagraphFontSize = 1.5; // em

export function setParagraphFontSize(sizeEm) {
  document.documentElement.style.setProperty('--paragraph-font-size', `${sizeEm}em`);
}

export function handleFontSizeChangeEvent(event) {
  if (event.payload === 'increase') {
    currentParagraphFontSize = Math.min(currentParagraphFontSize + 0.1, 3.0);
  } else if (event.payload === 'decrease') {
    currentParagraphFontSize = Math.max(currentParagraphFontSize - 0.1, 0.8);
  }
  setParagraphFontSize(currentParagraphFontSize);
}

// Optionally, initialize from CSS variable on load
export function initializeFontSizeFromCSS() {
  const cssValue = getComputedStyle(document.documentElement).getPropertyValue('--paragraph-font-size');
  if (cssValue) {
    const match = cssValue.match(/([\d.]+)em/);
    if (match) {
      currentParagraphFontSize = parseFloat(match[1]);
    }
  }
  setParagraphFontSize(currentParagraphFontSize);
}