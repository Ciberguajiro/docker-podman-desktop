import type { AppSettings } from '../types';

function createSettingsStore() {
  let settings = $state<AppSettings>({
    theme: 'dark',
    language: 'en',
    autoRefresh: true,
    refreshInterval: 10, // seconds
  });

  // Load from localStorage if available
  const saved = localStorage.getItem('app-settings');
  if (saved) {
    try {
      const parsed = JSON.parse(saved);
      if (parsed.theme) settings.theme = parsed.theme;
      if (parsed.language) settings.language = parsed.language;
      if (parsed.autoRefresh !== undefined) settings.autoRefresh = parsed.autoRefresh;
      if (parsed.refreshInterval) settings.refreshInterval = parsed.refreshInterval;
    } catch (e) {
      console.error('Failed to parse settings', e);
    }
  }

  return {
    get value() {
      return settings;
    },
    get theme() {
      return settings.theme;
    },
    set theme(val: string) {
      settings.theme = val;
      this.save();
    },
    get language() {
      return settings.language;
    },
    set language(val: string) {
      settings.language = val;
      this.save();
    },
    get autoRefresh() {
      return settings.autoRefresh;
    },
    set autoRefresh(val: boolean) {
      settings.autoRefresh = val;
      this.save();
    },
    get refreshInterval() {
      return settings.refreshInterval;
    },
    set refreshInterval(val: number) {
      settings.refreshInterval = val;
      this.save();
    },
    save() {
      localStorage.setItem('app-settings', JSON.stringify(settings));
    },
  };
}

export const settingsStore = createSettingsStore();
