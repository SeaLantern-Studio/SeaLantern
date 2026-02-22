import { type LocaleCode } from "@language";

export async function fetchLocale(
  locale: LocaleCode,
  onProgress?: (loaded: number, total: number | null) => void,
  baseUrl?: string,
) {
  // 直接从本地导入语言文件
  try {
    // 语言代码映射，处理特殊情况
    const localeFileMap: Record<string, string> = {};

    // 获取实际的文件名
    const actualLocale = localeFileMap[locale] || locale;

    // 动态导入本地语言文件（使用路径别名以便解析）
    const module = await import(`@language/${actualLocale}.json`);
    const data = module.default;

    return data;
  } catch (error) {
    console.error(`Failed to load locale ${locale} from local files:`, error);
    throw error;
  }
}
