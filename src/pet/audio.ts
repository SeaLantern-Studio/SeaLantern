import friendsWineDjUrl from "@assets/pet/friends-wine-dj.mp3";

/**
 * 通透模式音频。
 *
 * 桌面图标弹弹弹和桌宠本体都使用参考项目的 1.1 次/秒的原版
 * Squash & Stretch 周期；音频只负责循环播放用户提供的本地音源。
 */
const VOLUME = 0.68;

export class PetAudioEngine {
  private audio: HTMLAudioElement | null = null;

  get playing(): boolean {
    return this.audio !== null && !this.audio.paused;
  }

  async start(): Promise<void> {
    if (this.playing) return;
    const audio = this.ensureAudio();
    audio.currentTime = 0;
    await audio.play();
  }

  stop(): void {
    if (!this.audio) return;
    this.audio.pause();
    try {
      this.audio.currentTime = 0;
    } catch {
      // 某些 WebView 在媒体尚未完成加载时会拒绝 seek，忽略即可。
    }
  }

  dispose(): void {
    this.stop();
    this.audio = null;
  }

  private ensureAudio(): HTMLAudioElement {
    if (this.audio) return this.audio;
    const audio = new Audio(friendsWineDjUrl);
    audio.loop = true;
    audio.preload = "auto";
    audio.volume = VOLUME;
    this.audio = audio;
    return audio;
  }
}
