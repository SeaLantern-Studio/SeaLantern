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

function createDefaultQuote(): Quote {
  return { text: i18n.t("common.quote_text"), author: "Sea Lantern" };
}

// 引用相关的响应式数据
const currentQuote = ref<Quote>({ text: "", author: "" });
const displayText = ref("");
const isTyping = ref(false);
const quoteCache = ref<Quote[]>([]);
let typeTimer: ReturnType<typeof setInterval> | null = null;
let quoteTimer: ReturnType<typeof setInterval> | null = null;

function isSameQuote(left: Quote, right: Quote): boolean {
  return left.text === right.text && left.author === right.author;
}

function isCurrentQuote(quote: Quote): boolean {
  return !!currentQuote.value.text && isSameQuote(currentQuote.value, quote);
}

function stopTypeTimer() {
  if (typeTimer) {
    clearInterval(typeTimer);
    typeTimer = null;
  }
}

function restoreCurrentQuoteDisplay() {
  stopTypeTimer();
  if (currentQuote.value.text) {
    displayText.value = currentQuote.value.text;
  }
  isTyping.value = false;
}

/**
 * 打字机效果函数
 * @param text 要显示的文本
 * @param callback 完成后的回调函数
 */
function typeWriter(text: string, callback?: () => void) {
  stopTypeTimer();
  displayText.value = "";
  isTyping.value = true;
  let index = 0;
  typeTimer = setInterval(() => {
    if (index < text.length) {
      displayText.value += text[index];
      index++;
    } else {
      stopTypeTimer();
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
  stopTypeTimer();
  if (!displayText.value) {
    isTyping.value = false;
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
      stopTypeTimer();
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
  return quoteCache.value.some((cachedQuote) => isSameQuote(cachedQuote, quote));
}

function shouldCacheQuote(quote: Quote): boolean {
  return !isCurrentQuote(quote) && !isQuoteInCache(quote);
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

async function requestDistinctHitokoto(remainingRetries: number): Promise<Quote> {
  const quote = await requestHitokoto();
  if (!isCurrentQuote(quote) || remainingRetries <= 0) {
    return quote;
  }

  return requestDistinctHitokoto(remainingRetries - 1);
}

async function fetchHitokoto(): Promise<Quote> {
  while (quoteCache.value.length > 0) {
    const quote = quoteCache.value.shift();
    if (!quote) {
      break;
    }

    if (isCurrentQuote(quote)) {
      continue;
    }

    void replenishCache();
    return quote;
  }

  try {
    const quote = await requestDistinctHitokoto(3);
    void replenishCache();
    return quote;
  } catch (error) {
    console.error("Error fetching hitokoto:", error);
    return createDefaultQuote();
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
        if (shouldCacheQuote(result.value)) {
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
  if (currentQuote.value.text) {
    restoreCurrentQuoteDisplay();
    void replenishCache();
    return;
  }

  try {
    const initialFallback = createDefaultQuote();
    currentQuote.value = initialFallback;
    restoreCurrentQuoteDisplay();

    void replenishCache();

    const initialQuote = await fetchHitokoto();
    if (isCurrentQuote(initialQuote)) {
      return;
    }

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
 * 暂停引用自动更新，并恢复到可立即展示的稳定状态
 */
function pauseQuoteUpdates() {
  stopQuoteTimer();

  if (
    isTyping.value ||
    (currentQuote.value.text && displayText.value !== currentQuote.value.text)
  ) {
    restoreCurrentQuoteDisplay();
    return;
  }

  stopTypeTimer();
  isTyping.value = false;
}

/**
 * 恢复引用展示和自动更新
 * @param interval 更新间隔（毫秒），默认30000毫秒
 */
function resumeQuoteUpdates(interval: number = 30000) {
  if ((currentQuote.value.text && !displayText.value) || isTyping.value) {
    restoreCurrentQuoteDisplay();
  }

  startQuoteTimer(interval);
}

/**
 * 清理引用相关资源
 */
function cleanupQuoteResources() {
  stopTypeTimer();
  stopQuoteTimer();
  isTyping.value = false;
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
  pauseQuoteUpdates,
  resumeQuoteUpdates,
  cleanupQuoteResources,
};
