/**
 * 文档工具函数
 * 提供文档站链接生成功能
 */

const DOCS_BASE = "https://docs.ideaflash.cn";

/** 文档站链接 */
export function getDocUrl(key: string): string {
  return DOCS_BASE + "/zh/" + key;
}
