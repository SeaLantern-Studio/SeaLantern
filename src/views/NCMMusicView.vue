<template>
  <div class="music-view">
    <h2>网易云音乐搜索</h2>

    <!-- 搜索框 -->
    <div class="search-box">
      <input
        v-model="keyword"
        type="text"
        placeholder="输入歌曲名或歌手"
        @keyup.enter="handleSearch"
      />
      <button @click="handleSearch" :disabled="loading">搜索</button>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="error">{{ error }}</div>

    <!-- 加载中 -->
    <div v-if="loading" class="loading">加载中...</div>

    <!-- 搜索结果列表 -->
    <ul v-else-if="songs.length" class="song-list">
      <li
        v-for="song in songs"
        :key="song.id"
        @click="handlePlay(song.id)"
        class="song-item"
      >
        <div class="song-info">
          <span class="song-name">{{ song.name }}</span>
          <span class="song-artist" v-if="song.artists.length">
            —— {{ song.artists.map(a => a.name).join('/') }}
          </span>
        </div>
        <div class="song-album" v-if="song.album.name">
          《{{ song.album.name }}》
        </div>
      </li>
    </ul>
    <div v-else-if="searched" class="empty">暂无搜索结果</div>

    <!-- 音频播放器 -->
    <div v-if="currentUrl" class="player">
      <h3>正在播放</h3>
      <audio
        :src="currentUrl"
        controls
        autoplay
        @error="handleAudioError"
        class="audio-player"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { searchMusic, getMusicUrl, type NCMMusic } from '@/api/ncm_music';

// 状态
const keyword = ref('');
const songs = ref<NCMMusic[]>([]);
const loading = ref(false);
const error = ref('');
const searched = ref(false);
const currentUrl = ref('');

// 搜索
async function handleSearch() {
  if (!keyword.value.trim()) return;
  loading.value = true;
  error.value = '';
  searched.value = true;
  try {
    songs.value = await searchMusic(keyword.value.trim());
  } catch (err: any) {
    error.value = err.message || '搜索出错';
    songs.value = [];
  } finally {
    loading.value = false;
  }
}

// 播放
async function handlePlay(songId: number) {
  loading.value = true;
  error.value = '';
  try {
    currentUrl.value = await getMusicUrl(songId);
  } catch (err: any) {
    error.value = err.message || '获取播放地址失败';
    currentUrl.value = '';
  } finally {
    loading.value = false;
  }
}

// 音频播放错误处理
function handleAudioError() {
  error.value = '播放失败，可能是链接失效或无版权';
  currentUrl.value = '';
}
</script>

<style scoped>
.music-view {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px;
  font-family: system-ui, sans-serif;
}

.search-box {
  display: flex;
  gap: 8px;
  margin: 20px 0;
}

.search-box input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ccc;
  border-radius: 4px;
  font-size: 16px;
}

.search-box button {
  padding: 8px 16px;
  background-color: #42b983;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
}

.search-box button:hover {
  background-color: #3aa876;
}

.search-box button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.error {
  color: #e74c3c;
  margin: 10px 0;
  padding: 8px;
  background-color: #fde9e8;
  border-radius: 4px;
}

.loading {
  color: #666;
  margin: 10px 0;
}

.empty {
  color: #999;
  margin: 20px 0;
  text-align: center;
}

.song-list {
  list-style: none;
  padding: 0;
  margin: 20px 0;
}

.song-item {
  padding: 12px;
  margin-bottom: 8px;
  border: 1px solid #eee;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.song-item:hover {
  background-color: #f5f5f5;
}

.song-info {
  margin-bottom: 4px;
}

.song-name {
  font-weight: bold;
  font-size: 16px;
}

.song-artist {
  color: #666;
  font-size: 14px;
  margin-left: 8px;
}

.song-album {
  color: #999;
  font-size: 13px;
}

.player {
  margin-top: 30px;
  padding-top: 20px;
  border-top: 1px solid #eee;
}

.audio-player {
  width: 100%;
  margin-top: 10px;
}
</style>