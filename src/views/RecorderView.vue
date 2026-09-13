<script setup lang="ts">
import {onMounted, ref} from "vue";
import {useRouter} from "vue-router";
import {useMessage} from "naive-ui";
import {api} from "../core/tauri";
import type {RecordInfo} from "../core/defined";
import {TIMES} from "../core/defined";

const router = useRouter();
const message = useMessage();

const records = ref<RecordInfo[]>([]);
const showDrawer = ref(false);
const saving = ref(false);

const form = ref({
    name: "",
    gender: "男" as "男" | "女",
    birthday: null as number | null,
});

const pad = (n: number) => String(n).padStart(2, "0");

function timeText(index: number) {
    return TIMES[((index % 12) + 12) % 12] + "时";
}

function solarText(r: RecordInfo) {
    return `${r.solarYear}-${pad(r.solarMonth)}-${pad(r.solarDay)}`;
}

function yearStemBranch(r: RecordInfo) {
    return r.yearStem ? `${r.yearStem}${r.yearBranch}年` : "";
}

async function load() {
    records.value = await api.listRecords();
}

function openChart(r: RecordInfo) {
    router.push({path: `/ziwei/${r.id}`});
}

async function remove(r: RecordInfo) {
    await api.deleteRecord(r.id);
    message.success("已删除");
    await load();
}

function openDrawer() {
    form.value = {name: "", gender: "男", birthday: null};
    showDrawer.value = true;
}

async function save() {
    if (!form.value.name.trim()) {
        message.warning("请输入姓名");
        return;
    }
    if (!form.value.gender) {
        message.warning("请选择性别");
        return;
    }
    if (form.value.birthday == null) {
        message.warning("请选择出生时间");
        return;
    }
    const d = new Date(form.value.birthday);
    const hour = d.getHours();
    const timeIndex = Math.floor((hour + 1) / 2) % 12;
    saving.value = true;
    try {
        await api.addRecord({
            name: form.value.name.trim(),
            calendarType: "solar",
            solarYear: d.getFullYear(),
            solarMonth: d.getMonth() + 1,
            solarDay: d.getDate(),
            timeIndex,
            gender: form.value.gender,
        });
        message.success("保存成功");
        showDrawer.value = false;
        await load();
    } finally {
        saving.value = false;
    }
}

onMounted(load);
</script>

<template>
  <div class="recorder-page">
    <header class="recorder-header">
      <div class="recorder-title">紫微斗数</div>
      <n-button type="primary" size="small" @click="openDrawer">新增</n-button>
    </header>

    <div class="recorder-list">
      <div v-if="records.length === 0" class="recorder-empty">暂无记录，点击右上角「新增」开始排盘</div>
      <div
          v-for="r in records"
          :key="r.id"
          class="recorder-card"
          @click="openChart(r)"
      >
        <div class="recorder-card-head">
          <div class="recorder-card-name">{{ r.name || "未命名" }}</div>
          <n-button
              text
              type="error"
              size="tiny"
              class="recorder-del"
              @click.stop="remove(r)"
          >删除</n-button>
        </div>
        <div class="recorder-card-meta">
          <span>{{ r.gender || "-" }}</span>
          <span>{{ solarText(r) }} {{ timeText(r.timeIndex) }}</span>
        </div>
        <div v-if="r.mainStars" class="recorder-card-sub">
          {{ yearStemBranch(r) }} {{ r.fiveElementsClass }} · {{ r.mainStars }}
        </div>
      </div>
    </div>

    <n-drawer v-model:show="showDrawer" placement="bottom" :height="340">
      <div class="recorder-drawer-title">新增命盘</div>
      <n-form label-placement="left" label-width="auto" class="recorder-add-view">
        <n-form-item label="姓名">
          <n-input v-model:value="form.name" placeholder="输入姓名或标记" @keyup.enter="save"/>
        </n-form-item>
        <n-form-item label="性别">
          <n-radio-group v-model:value="form.gender">
            <n-radio value="男">男</n-radio>
            <n-radio value="女">女</n-radio>
          </n-radio-group>
        </n-form-item>
        <n-form-item label="出生时间">
          <n-date-picker
              v-model:value="form.birthday"
              type="datetime"
              :time-picker-props="{format: 'HH:mm'}"
              style="width: 100%"
              clearable
          />
        </n-form-item>
        <n-form-item>
          <n-button
              block
              type="success"
              :loading="saving"
              @click="save"
          >保存</n-button>
        </n-form-item>
      </n-form>
    </n-drawer>
  </div>
</template>

<style scoped>
.recorder-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 12px;
  box-sizing: border-box;
  overflow: hidden;
  background: #f5f7fa;
}

.recorder-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.recorder-title {
  font-size: 20px;
  font-weight: 700;
  color: #333;
}

.recorder-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-bottom: 8px;
}

.recorder-empty {
  text-align: center;
  color: #999;
  margin-top: 40px;
}

.recorder-card {
  background: #fff;
  border-radius: 8px;
  padding: 12px 14px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  cursor: pointer;
  transition: box-shadow 0.2s;
}

.recorder-card:hover {
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.15);
}

.recorder-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.recorder-card-name {
  font-size: 18px;
  font-weight: 700;
  color: #222;
}

.recorder-card-meta {
  margin-top: 4px;
  display: flex;
  gap: 14px;
  color: #666;
  font-size: 13px;
}

.recorder-card-sub {
  margin-top: 4px;
  color: #1c7a3d;
  font-size: 12px;
}

.recorder-drawer-title {
  text-align: center;
  font-weight: 600;
  color: #333;
  margin-bottom: 8px;
}

.recorder-add-view {
  padding: 4px 14px 16px;
}
</style>