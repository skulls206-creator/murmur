/**
 * Murmur Theme Manager
 *
 * Handles dark/light/system theme switching.
 * Runs as progressive enhancement — no theme = default dark mode.
 */

(function() {
  'use strict';

  const STORAGE_KEY = 'murmur-theme';

  function getStoredTheme() {
    try {
      return localStorage.getItem(STORAGE_KEY);
    } catch {
      return null;
    }
  }

  function setTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    try {
      localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      // Storage may be unavailable
    }
  }

  function getPreferredTheme() {
    const stored = getStoredTheme();
    if (stored && stored !== 'system') return stored;
    return 'system';
  }

  // Initialize
  const theme = getPreferredTheme();
  setTheme(theme);

  // Listen for system preference changes
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  mediaQuery.addEventListener('change', () => {
    const current = getStoredTheme();
    if (!current || current === 'system') {
      setTheme('system');
    }
  });

  // Expose theme API globally for the settings page
  window.MurmurTheme = {
    get: getPreferredTheme,
    set: setTheme,
  };
})();
