<template>
  <div class="frp-view">
    <div class="frp-form">
      <div class="form-row">
        <label class="form-label">FRP提供商</label>
        <select class="form-select" v-model="provider">
          <option value="openfrp">OpenFrp</option>
          <option value="sakurafrp">SakuraFrp</option>
          <option value="chmlfrp">ChmlFrp</option>
        </select>
      </div>
      <div class="form-row">
        <label class="form-label">token</label>
        <input type="text" class="form-input" placeholder="粘贴token" v-model="token" />
      </div>
      <div class="form-row">
        <label class="form-label">隧道ID</label>
        <input type="text" class="form-input" placeholder="输入隧道ID" v-model="tunnelId" />
      </div>
      <div class="form-actions">
        <button class="start-btn" @click="startTunnel">启动隧道</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { startFrpTunnel } from "@api/system";
import { useToast } from "@composables/useToast";

const provider = ref("openfrp");
const token = ref("");
const tunnelId = ref("");
const { success, error } = useToast();

async function startTunnel() {
  if (!token.value || !tunnelId.value) {
    error("请填写完整信息");
    return;
  }

  try {
    await startFrpTunnel(provider.value, token.value, tunnelId.value);
    success("隧道启动成功");
  } catch (err) {
    error("隧道启动失败");
    console.error(err);
  }
}
</script>

<style scoped>
.frp-view {
  display: flex;
  justify-content: center;
  align-items: flex-start;
  height: 100%;
  width: 100%;
  padding-top: 50px;
}

.frp-form {
  width: 400px;
  padding: 20px;
}

.form-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.form-label {
  font-family: "黑体", Arial, sans-serif;
  font-weight: bold;
  font-size: 14px;
  width: 100px;
}

.form-select {
  font-family: "Calibri", Arial, sans-serif;
  font-size: 14px;
  padding: 8px;
  width: 250px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.form-input {
  font-family: "宋体", Arial, sans-serif;
  font-style: italic;
  font-size: 14px;
  padding: 8px;
  width: 250px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.form-input::placeholder {
  color: #999;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 30px;
}

.start-btn {
  font-family: "黑体", Arial, sans-serif;
  font-weight: bold;
  font-size: 14px;
  padding: 10px 20px;
  background-color: #60a5fa;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.start-btn:hover {
  background-color: #3b82f6;
}
</style>
