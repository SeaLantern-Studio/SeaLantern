/**
 * vue-i18n 国际化配置
 */
import { createI18n } from 'vue-i18n';
import zhCN from './locales/zh-CN';
import en from './locales/en';

export type MessageSchema = typeof zhCN;

const i18n = createI18n<[MessageSchema], 'zh-CN' | 'en'>({
    legacy: false, // 使用 Composition API 模式
    locale: navigator.language.startsWith('zh') ? 'zh-CN' : 'en',
    fallbackLocale: 'zh-CN',
    messages: {
        'zh-CN': zhCN,
        'en': en,
    },
});

export default i18n;
