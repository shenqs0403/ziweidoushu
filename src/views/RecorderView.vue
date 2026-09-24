<script setup lang="ts">
import {computed, nextTick, onMounted, ref, watch} from "vue";
import {useRouter} from "vue-router";
import {useMessage} from "naive-ui";
import {api} from "../core/tauri";
import type {LunarYearInfo} from "../core/tauri";
import type {RecordInfo} from "../core/defined";
import {TIMES} from "../core/defined";

const router = useRouter();
const message = useMessage();

const records = ref<RecordInfo[]>([]);
const showDrawer = ref(false);
const saving = ref(false);

const LUNAR_MONTH_NAMES = ["正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "冬月", "腊月"];
const lunarInfo = ref<LunarYearInfo | null>(null);

const form = ref({
    name: "",
    gender: "男" as "男" | "女",
    calendarType: "solar" as "solar" | "lunar",
    birthYear: null as number | null,
    birthMonth: null as number | null,
    birthDay: null as number | null,
    birthHour: null as number | null,
});

const pickerShow = ref(false);

const years = Array.from({length: 201}, (_, i) => 1900 + i);
const hours = Array.from({length: 24}, (_, i) => i + 1);

const OPT_H = 34;

function daysInMonth(year: number, month: number) {
    return new Date(year, month, 0).getDate();
}

const monthItems = computed<{ value: number; label: string }[]>(() => {
    if (form.value.calendarType === "solar") {
        return Array.from({length: 12}, (_, i) => ({value: i + 1, label: `${i + 1}月`}));
    }
    const leap = lunarInfo.value?.leapMonth ?? 0;
    const arr: { value: number; label: string }[] = [];
    for (let m = 1; m <= 12; m++) {
        arr.push({value: m, label: LUNAR_MONTH_NAMES[m - 1]});
        if (leap === m) arr.push({value: 100 + m, label: `闰${LUNAR_MONTH_NAMES[m - 1]}`});
    }
    return arr;
});

const dayItems = computed(() => {
    if (form.value.birthYear == null || form.value.birthMonth == null) return [];
    if (form.value.calendarType === "solar") {
        const n = daysInMonth(form.value.birthYear, form.value.birthMonth);
        return Array.from({length: n}, (_, i) => i + 1);
    }
    const info = lunarInfo.value;
    if (!info) return [];
    const leap = form.value.birthMonth > 99;
    const m = leap ? form.value.birthMonth - 100 : form.value.birthMonth;
    const n = leap ? info.leapMonthDays : info.monthDays[m - 1] ?? 0;
    if (n <= 0) return [];
    return Array.from({length: n}, (_, i) => i + 1);
});

const birthText = computed(() => {
    const f = form.value;
    if (f.birthYear == null || f.birthMonth == null || f.birthDay == null || f.birthHour == null) return "";
    if (f.calendarType === "solar") {
        return `${f.birthYear}年${f.birthMonth}月${f.birthDay}日 ${f.birthHour}时`;
    }
    const leap = f.birthMonth > 99;
    const m = leap ? f.birthMonth - 100 : f.birthMonth;
    return `${f.birthYear}年${leap ? "闰" : ""}${LUNAR_MONTH_NAMES[m - 1]}${f.birthDay}日 ${f.birthHour}时`;
});

const colEls: Record<string, HTMLElement | null> = {
    year: null,
    month: null,
    day: null,
    hour: null,
};

type ColKind = "year" | "month" | "day" | "hour";

function colScrollTop(idx: number) {
    return idx * OPT_H;
}

function scrollToIndex(kind: ColKind, idx: number) {
    const el = colEls[kind];
    if (el) el.scrollTop = colScrollTop(idx);
}

function monthItemIndex(val: number) {
    return monthItems.value.findIndex((x) => x.value === val);
}

function pick(kind: ColKind, val: number) {
    if (kind === "year") form.value.birthYear = val;
    else if (kind === "month") form.value.birthMonth = val;
    else if (kind === "day") form.value.birthDay = val;
    else form.value.birthHour = val;
    const idx = kind === "year" ? years.indexOf(val) : kind === "month" ? monthItemIndex(val) : val - 1;
    scrollToIndex(kind, Math.max(0, idx));
}

function onColScroll(kind: ColKind, e: Event) {
    const el = e.currentTarget as HTMLElement;
    const idx = Math.round(el.scrollTop / OPT_H);
    let val: number | undefined;
    if (kind === "year") val = years[idx];
    else if (kind === "month") val = monthItems.value[idx]?.value;
    else if (kind === "day") val = dayItems.value[idx];
    else val = hours[idx];
    if (val == null) return;
    if (kind === "year") form.value.birthYear = val;
    else if (kind === "month") form.value.birthMonth = val;
    else if (kind === "day") form.value.birthDay = val;
    else form.value.birthHour = val;
}

watch(
    () => [form.value.calendarType, form.value.birthYear, form.value.birthMonth],
    async () => {
        if (form.value.calendarType === "lunar" && form.value.birthYear != null) {
            try {
                lunarInfo.value = await api.lunarYearInfo(form.value.birthYear);
            } catch {
                lunarInfo.value = null;
            }
        } else {
            lunarInfo.value = null;
        }
        if (form.value.birthYear == null || form.value.birthMonth == null) return;
        const max = dayItems.value.length;
        if (form.value.birthDay != null && form.value.birthDay > max) form.value.birthDay = max || null;
    },
);

function openPicker() {
    if (form.value.birthYear == null) form.value.birthYear = new Date().getFullYear();
    if (form.value.birthMonth == null) form.value.birthMonth = 1;
    if (form.value.birthDay == null) form.value.birthDay = 1;
    const hh = new Date().getHours();
    if (form.value.birthHour == null) form.value.birthHour = hh === 0 ? 24 : hh;
    pickerShow.value = true;
    nextTick(() => {
        scrollToIndex("year", years.indexOf(form.value.birthYear!));
        scrollToIndex("month", Math.max(0, monthItemIndex(form.value.birthMonth!)));
        scrollToIndex("day", form.value.birthDay! - 1);
        scrollToIndex("hour", Math.max(0, form.value.birthHour! - 1));
    });
}

function closePicker() {
    pickerShow.value = false;
}

function timeText(index: number) {
    return TIMES[((index % 12) + 12) % 12] + "时";
}

function solarText(r: RecordInfo) {
    return `${r.solarYear}年${r.solarMonth}月${r.solarDay}日`;
}

function lunarText(r: RecordInfo) {
    if (r.lunarYear == null || r.lunarMonth == null) return "";
    const m = r.lunarMonth > 0 ? r.lunarMonth : 1;
    return `${r.lunarYear}年${r.isLeap ? "闰" : ""}${LUNAR_MONTH_NAMES[m - 1] ?? `${m}月`}${r.lunarDay}日`;
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
    form.value = {
        name: "",
        gender: "男",
        calendarType: "solar",
        birthYear: null,
        birthMonth: null,
        birthDay: null,
        birthHour: null,
    };
    lunarInfo.value = null;
    showDrawer.value = true;
}

async function save() {
    if (!form.value.name.trim()) {
        message.warning("请输入姓名");
        return;
    }
    if (form.value.birthYear == null || form.value.birthMonth == null || form.value.birthDay == null || form.value.birthHour == null) {
        message.warning("请选择出生时间");
        return;
    }
    const hour = form.value.birthHour === 24 ? 0 : form.value.birthHour;
    const timeIndex = Math.floor((hour + 1) / 2) % 12;
    saving.value = true;
    try {
        const base = {name: form.value.name.trim(), timeIndex, gender: form.value.gender};
        if (form.value.calendarType === "solar") {
            const d = new Date(form.value.birthYear, form.value.birthMonth - 1, form.value.birthDay);
            if (
                d.getFullYear() !== form.value.birthYear ||
                d.getMonth() + 1 !== form.value.birthMonth ||
                d.getDate() !== form.value.birthDay
            ) {
                message.warning("出生日期不合法");
                return;
            }
            await api.addRecord({
                ...base,
                calendarType: "solar",
                solarYear: form.value.birthYear,
                solarMonth: form.value.birthMonth,
                solarDay: form.value.birthDay,
            });
        } else {
            const leap = form.value.birthMonth > 99;
            const lm = leap ? form.value.birthMonth - 100 : form.value.birthMonth;
            await api.addRecord({
                ...base,
                calendarType: "lunar",
                lunarYear: form.value.birthYear,
                lunarMonth: lm,
                lunarDay: form.value.birthDay,
                isLeap: leap,
            });
        }
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
          <span v-if="r.calendarType === 'lunar'" class="recorder-lunar">农历 {{ lunarText(r) }}</span>
        </div>
        <div v-if="r.mainStars" class="recorder-card-sub">
          {{ yearStemBranch(r) }} {{ r.fiveElementsClass }} · {{ r.mainStars }}
        </div>
      </div>
    </div>

    <n-drawer v-model:show="showDrawer" placement="bottom" :height="290">
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
        <n-form-item label="历法">
          <n-radio-group v-model:value="form.calendarType">
            <n-radio value="solar">阳历</n-radio>
            <n-radio value="lunar">农历</n-radio>
          </n-radio-group>
        </n-form-item>
        <n-form-item label="出生时间" class="birth-item" @click="openPicker">
          <div class="birth-field" @click.stop="openPicker">
            <span :class="{ph: !birthText}">{{ birthText || "点击选择年月日时" }}</span>
          </div>
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

    <div v-if="pickerShow" class="dt-picker-mask" @click="closePicker">
      <div class="dt-picker" @click.stop>
        <div class="dt-head">
          <span class="dt-btn-cancel" @click="closePicker">取消</span>
          <span class="dt-title">选择出生时间</span>
          <span class="dt-btn-ok" @click="closePicker">确定</span>
        </div>
        <div class="dt-cols">
          <div
              :ref="(el) => { colEls.year = el as HTMLElement | null }"
              class="dt-col"
              @scroll.passive="onColScroll('year', $event)"
          >
            <div class="dt-spacer"></div>
            <div
                v-for="y in years"
                :key="y"
                class="dt-opt"
                :class="{on: y === form.birthYear}"
                @click="pick('year', y)"
            >{{ y }}年</div>
            <div class="dt-spacer"></div>
          </div>
          <div
              :ref="(el) => { colEls.month = el as HTMLElement | null }"
              class="dt-col"
              @scroll.passive="onColScroll('month', $event)"
          >
            <div class="dt-spacer"></div>
            <div
                v-for="mo in monthItems"
                :key="mo.value"
                class="dt-opt"
                :class="{on: mo.value === form.birthMonth}"
                @click="pick('month', mo.value)"
            >{{ mo.label }}</div>
            <div class="dt-spacer"></div>
          </div>
          <div
              :ref="(el) => { colEls.day = el as HTMLElement | null }"
              class="dt-col"
              @scroll.passive="onColScroll('day', $event)"
          >
            <div class="dt-spacer"></div>
            <div
                v-for="dd in dayItems"
                :key="dd"
                class="dt-opt"
                :class="{on: dd === form.birthDay}"
                @click="pick('day', dd)"
            >{{ dd }}日</div>
            <div class="dt-spacer"></div>
          </div>
          <div
              :ref="(el) => { colEls.hour = el as HTMLElement | null }"
              class="dt-col"
              @scroll.passive="onColScroll('hour', $event)"
          >
            <div class="dt-spacer"></div>
            <div
                v-for="h in hours"
                :key="h"
                class="dt-opt"
                :class="{on: h === form.birthHour}"
                @click="pick('hour', h)"
            >{{ h }}时</div>
            <div class="dt-spacer"></div>
          </div>
          <div class="dt-line"></div>
        </div>
      </div>
    </div>
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

.recorder-lunar {
  color: #8a6d3b;
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

.birth-item .n-form-item-control {
  width: 100%;
}

.birth-field {
  width: 100%;
  border: 1px solid #dcdfe6;
  border-radius: 3px;
  height: 34px;
  line-height: 34px;
  padding: 0 12px;
  box-sizing: border-box;
  font-size: 14px;
  color: #333;
  background: #fff;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.birth-field .ph {
  color: #a0a5ad;
}

.dt-picker-mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 9999;
  display: flex;
  align-items: flex-end;
}

.dt-picker {
  width: 100%;
  background: #fff;
  border-radius: 12px 12px 0 0;
}

.dt-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
}

.dt-title {
  font-size: 15px;
  font-weight: 600;
  color: #333;
}

.dt-btn-cancel {
  font-size: 14px;
  color: #999;
  cursor: pointer;
}

.dt-btn-ok {
  font-size: 14px;
  color: #1e8cd6;
  font-weight: 600;
  cursor: pointer;
}

.dt-cols {
  position: relative;
  display: flex;
  height: 204px;
  overflow: hidden;
}

.dt-col {
  flex: 1;
  min-width: 0;
  height: 100%;
  overflow-y: auto;
  scroll-snap-type: y mandatory;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}

.dt-col::-webkit-scrollbar {
  display: none;
}

.dt-spacer {
  height: 85px;
  flex-shrink: 0;
  scroll-snap-align: none;
}

.dt-opt {
  height: 34px;
  line-height: 34px;
  text-align: center;
  font-size: 15px;
  color: #555;
  scroll-snap-align: center;
  cursor: pointer;
}

.dt-opt.on {
  color: #1e8cd6;
  font-weight: 700;
}

.dt-line {
  position: absolute;
  left: 0;
  right: 0;
  top: 85px;
  height: 34px;
  border-top: 1px solid #d9ecfb;
  border-bottom: 1px solid #d9ecfb;
  background: rgba(232, 244, 253, 0.5);
  pointer-events: none;
}
</style>