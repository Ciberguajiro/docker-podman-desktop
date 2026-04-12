import { settingsStore } from './settings.svelte';
import en from '../i18n/en.json';
import es from '../i18n/es.json';

const translations: Record<string, any> = { en, es };

export function createI18nStore() {
  const t = $derived.by(() => {
    const lang = settingsStore.language || 'en';
    const dict = translations[lang] || translations['en'];

    return (key: keyof typeof en) => {
      return dict[key] || key;
    };
  });

  return {
    get t() {
      return t;
    },
  };
}

export const i18n = createI18nStore();
