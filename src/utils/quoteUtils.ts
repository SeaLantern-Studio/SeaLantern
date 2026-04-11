import { ref } from "vue";
import { i18n } from "@language";

interface HitokotoResponse {
  id: number;
  hitokoto: string;
  type: string;
  from: string;
  from_who: string | null;
  creator: string;
  creator_uid: number;
  review_status: number;
  uuid: string;
  created_at: string;
}

interface Quote {
  text: string;
  author: string;
}

// 引用相关的响应式数据
const currentQuote = ref<Quote>({ text: "", author: "" });
const displayText = ref("");
const isTyping = ref(false);
const quoteCache = ref<Quote[]>([]);
let typeTimer: ReturnType<typeof setInterval> | null = null;
let quoteTimer: ReturnType<typeof setInterval> | null = null;

/**
 * 打字机效果函数
 * @param text 要显示的文本
 * @param callback 完成后的回调函数
 */
function typeWriter(text: string, callback?: () => void) {
  if (typeTimer) clearInterval(typeTimer);
  displayText.value = "";
  isTyping.value = true;
  let index = 0;
  typeTimer = setInterval(() => {
    if (index < text.length) {
      displayText.value += text[index];
      index++;
    } else {
      if (typeTimer) clearInterval(typeTimer);
      isTyping.value = false;
      if (callback) callback();
    }
  }, 50);
}

/**
 * 打字机效果消失函数
 * @param callback 完成后的回调函数
 */
function typeWriterOut(callback?: () => void) {
  if (typeTimer) clearInterval(typeTimer);
  if (!displayText.value) {
    if (callback) callback();
    return;
  }
  isTyping.value = true;
  let chars = displayText.value.split("");
  typeTimer = setInterval(() => {
    if (chars.length > 0) {
      chars.pop();
      displayText.value = chars.join("");
    } else {
      if (typeTimer) clearInterval(typeTimer);
      isTyping.value = false;
      if (callback) callback();
    }
  }, 30);
}

/**
 * 检查引用是否已在缓存中
 * @param quote 要检查的引用
 * @returns 是否在缓存中
 */
function isQuoteInCache(quote: Quote): boolean {
  return quoteCache.value.some((cachedQuote) => cachedQuote.text === quote.text);
}

/**
 * 获取一句名言/引用
 * @returns 名言/引用对象
 */
async function requestHitokoto(): Promise<Quote> {
  const response = await fetch("https://v1.hitokoto.cn/?encode=json");
  if (!response.ok) {
    throw new Error("Failed to fetch hitokoto");
  }
  const data: HitokotoResponse = await response.json();
  return {
    text: data.hitokoto,
    author: data.from_who || data.from || i18n.t("common.unknown"),
  };
}

async function fetchHitokoto(): Promise<Quote> {
  if (quoteCache.value.length > 0) {
    const quote = quoteCache.value.shift();
    void replenishCache();
    return quote!;
  }

  try {
    const quote = await requestHitokoto();
    void replenishCache();
    return quote;
  } catch (error) {
    console.error("Error fetching hitokoto:", error);
    const defaultQuote = { text: i18n.t("common.quote_text"), author: "Sea Lantern" };
    return defaultQuote;
  }
}

/**
 * 补充引用缓存
 */
async function replenishCache() {
  const needed = Math.max(0, 2 - quoteCache.value.length);
  if (needed === 0) {
    return;
  }

  try {
    const results = await Promise.allSettled(
      Array.from({ length: needed }, () => requestHitokoto()),
    );
    for (const result of results) {
      if (result.status === "fulfilled") {
        if (!isQuoteInCache(result.value)) {
          quoteCache.value.push(result.value);
        }
        continue;
      }

      console.error("Error replenishing quote cache:", result.reason);
    }
  } catch (error) {
    console.error("Error replenishing quote cache:", error);
  }
}

/**
 * 更新引用
 */
async function updateQuote() {
  if (isTyping.value) {
    return;
  }
  typeWriterOut(async () => {
    try {
      const newQuote = await fetchHitokoto();
      currentQuote.value = newQuote;
      typeWriter(newQuote.text);
    } catch (error) {
      console.error("Error updating quote:", error);
    }
  });
}

/**
 * 初始化引用
 */
async function initQuote() {
  try {
    await replenishCache();
    const initialQuote = await fetchHitokoto();
    currentQuote.value = initialQuote;
    typeWriter(initialQuote.text);
  } catch (error) {
    console.error("Error initializing quote:", error);
  }
}

/**
 * 启动引用定时更新
 * @param interval 更新间隔（毫秒），默认30000毫秒
 */
function startQuoteTimer(interval: number = 30000) {
  stopQuoteTimer();
  quoteTimer = setInterval(updateQuote, interval);
}

/**
 * 停止引用定时更新
 */
function stopQuoteTimer() {
  if (quoteTimer) {
    clearInterval(quoteTimer);
    quoteTimer = null;
  }
}

/**
 * 清理引用相关资源
 */
function cleanupQuoteResources() {
  if (typeTimer) {
    clearInterval(typeTimer);
    typeTimer = null;
  }
  stopQuoteTimer();
}

export {
  currentQuote,
  displayText,
  isTyping,
  typeWriter,
  typeWriterOut,
  fetchHitokoto,
  replenishCache,
  updateQuote,
  initQuote,
  startQuoteTimer,
  stopQuoteTimer,
  cleanupQuoteResources,
};
