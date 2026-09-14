<script setup lang="ts">
import {computed, onMounted, ref, watch} from "vue";
import {useRoute, useRouter} from "vue-router";
import {useMessage} from "naive-ui";
import {api} from "../core/tauri";
import type {Chart, FlowAnnual, RecordInfo, Star} from "../core/defined";
import {BRANCHES, STEMS} from "../core/defined";
import {STAR_INTRO} from "../core/starIntro";

const route = useRoute();
const router = useRouter();
const message = useMessage();

const record = ref<RecordInfo | null>(null);
const chart = ref<Chart | null>(null);
const flow = ref<FlowAnnual | null>(null);
const loading = ref(true);

const decadeDist = ref<number | null>(null);
const flowYear = ref<number | null>(null);
const flowMonth = ref<number | null>(null);
const flowDay = ref<number | null>(null);

const introShow = ref(false);
const introStar = ref("");
const introText = ref("");

const fixIdx = (n: number) => ((n % 12) + 12) % 12;
const stemBranchText = (y: number) => {
    const s = ((y - 4) % 10 + 10) % 10;
    const b = ((y - 4) % 12 + 12) % 12;
    return `${STEMS[s]}${BRANCHES[b]}`;
};

const soulIndex = computed(() => {
    const p = chart.value?.palaces.find((x) => x.isSoul);
    return p ? p.index : -1;
});

const forward = computed(() => {
    if (!chart.value || soulIndex.value < 0) return true;
    const yang = ["甲", "丙", "戊", "庚", "壬"].includes(chart.value.yearStem);
    const gender = chart.value.birth.gender;
    return (gender === "男" && yang) || (gender === "女" && !yang);
});

function distToIndex(d: number): number {
    if (soulIndex.value < 0) return d;
    return forward.value ? fixIdx(soulIndex.value + d) : fixIdx(soulIndex.value - d);
}

function indexToDist(i: number): number {
    if (soulIndex.value < 0) return i;
    return forward.value ? fixIdx(i - soulIndex.value) : fixIdx(soulIndex.value - i);
}

const decadePalace = computed(() =>
    decadeDist.value == null ? null : distToIndex(decadeDist.value),
);

const decadeOptions = computed(() => {
    if (!chart.value) return [];
    const arr: { value: number; label: string }[] = [];
    for (let d = 0; d < 12; d++) {
        const p = chart.value.palaces[distToIndex(d)];
        arr.push({value: d, label: `${p.decadeStart}-${p.decadeEnd}`});
    }
    return arr;
});

const birthLunarYear = computed(() => chart.value?.birth.lunar.year ?? 0);

const yearOptions = computed(() => {
    if (!chart.value || decadePalace.value == null) return [];
    const p = chart.value.palaces[decadePalace.value];
    const arr: { value: number; label: string }[] = [];
    for (let age = p.decadeStart; age <= p.decadeEnd; age++) {
        const y = birthLunarYear.value + (age - 1);
        arr.push({value: y, label: `${y}年 ${stemBranchText(y)}`});
    }
    return arr;
});

const monthOptions = Array.from({length: 12}, (_, i) => ({value: i + 1, label: `农历${i + 1}月`}));
const dayOptions = Array.from({length: 30}, (_, i) => ({value: i + 1, label: `农历${i + 1}日`}));

watch(decadePalace, (idx) => {
    if (!chart.value || idx == null) return;
    const p = chart.value.palaces[idx];
    if (!p) return;
    const first = birthLunarYear.value + p.decadeStart - 1;
    const last = first + 9;
    if (flowYear.value == null || flowYear.value < first || flowYear.value > last) {
        flowYear.value = first;
    }
}, {immediate: true});

const flowYearBranchIndex = computed(() => {
    if (flowYear.value == null) return -1;
    return ((flowYear.value - 4) % 12 + 12) % 12;
});

const flowYearPalace = computed(() => {
    if (chart.value == null || flowYearBranchIndex.value < 0) return null;
    const b = BRANCHES[flowYearBranchIndex.value];
    const p = chart.value.palaces.find((x) => x.earthlyBranch === b);
    return p ? p.index : null;
});

const flowMonthPalace = computed(() => {
    if (flowYearPalace.value == null || flowMonth.value == null) return null;
    return fixIdx(flowYearPalace.value + flowMonth.value - 1);
});

const flowDayPalace = computed(() => {
    if (flowMonthPalace.value == null || flowDay.value == null) return null;
    return fixIdx(flowMonthPalace.value + flowDay.value - 1);
});

const activeLevel = ref<"decade" | "year" | "month" | "day" | null>(null);

const activePalace = computed(() => {
    switch (activeLevel.value) {
        case "decade":
            return decadePalace.value;
        case "year":
            return flowYearPalace.value;
        case "month":
            return flowMonthPalace.value;
        case "day":
            return flowDayPalace.value;
        default:
            return null;
    }
});

const triadSet = computed(() => {
    const set = new Set<number>();
    if (activePalace.value == null) return set;
    for (const off of [4, 8, 6]) set.add(fixIdx(activePalace.value + off));
    return set;
});

function palaceClass(i: number) {
    if (activePalace.value === i) return "hl-violet";
    if (triadSet.value.has(i)) return "hl-red";
    return "";
}

function palaceAge(i: number): string {
    if (flowYear.value == null || i !== flowYearPalace.value) return "";
    return `${flowYear.value - birthLunarYear.value + 1}岁`;
}

function allStars(p: { major: Star[]; minor: Star[]; adjective: Star[] }): Star[] {
    return [...p.major, ...p.minor, ...p.adjective];
}

function glyphs(s: Star): { ch: string; cls: string }[] {
    const list: { ch: string; cls: string }[] = [];
    for (const ch of s.name) list.push({ch, cls: `cat-${s.category}`});
    if (s.brightness) list.push({ch: s.brightness, cls: "cat-bright"});
    if (s.mutagen) list.push({ch: s.mutagen, cls: `mut-${s.mutagen}`});
    return list;
}

// 4x4 布局：寅序 index -> (row, col)
const POS: [number, number][] = [
    [3, 0], [2, 0], [1, 0], [0, 0], // 寅卯辰巳
    [0, 1], [0, 2], [0, 3], // 午未申
    [1, 3], [2, 3], [3, 3], // 酉戌亥
    [3, 2], [3, 1], // 子丑
];

function gridPos(i: number) {
    const [row, col] = POS[i];
    return {gridRow: row + 1, gridColumn: col + 1};
}

function onPalaceClick(i: number) {
    decadeDist.value = indexToDist(i);
    activeLevel.value = "decade";
}

function openStarIntro(s: Star) {
    introStar.value = s.name;
    introText.value = STAR_INTRO[s.name] ?? `暂无「${s.name}」的原文介绍。`;
    introShow.value = true;
}

let pressTimer: number | null = null;
function onPressStart(s: Star) {
    pressTimer = window.setTimeout(() => openStarIntro(s), 500);
}
function onPressEnd() {
    if (pressTimer) {
        clearTimeout(pressTimer);
        pressTimer = null;
    }
}

const flowYearText = computed(() =>
    flowYear.value == null ? "" : `${flowYear.value}年 ${stemBranchText(flowYear.value)}`,
);

const flowYearPalaceName = computed(() => {
    if (chart.value == null || flowYearPalace.value == null) return "";
    return chart.value.palaces[flowYearPalace.value].name;
});

async function load() {
    loading.value = true;
    try {
        const id = Number(route.params.id);
        const [rec, c] = await api.chartForRecord(id);
        record.value = rec;
        chart.value = c;
        decadeDist.value = null;
        flowMonth.value = null;
        flowDay.value = null;
    } catch (e) {
        message.error(String(e));
    } finally {
        loading.value = false;
    }
}

watch(flowYear, async (y) => {
    if (y == null || !chart.value) return;
    flow.value = await api.flowAnnual(y);
}, {immediate: true});

function goBack() {
    router.push("/recorder");
}

onMounted(load);
</script>

<template>
  <div class="chart-page">
    <div class="chart-topbar">
      <n-button text size="small" @click="goBack">‹ 返回</n-button>
      <span class="chart-name">{{ record?.name || "紫微斗数" }}</span>
      <n-spin v-if="loading" size="small"/>
    </div>

    <div v-if="chart" class="chart-grid">
      <div
          v-for="i in 12"
          :key="i - 1"
          class="palace"
          :style="gridPos(i - 1)"
          :class="palaceClass(i - 1)"
          @click="onPalaceClick(i - 1)"
      >
        <div class="p-stars">
          <div
              v-for="(s, si) in allStars(chart.palaces[i - 1])"
              :key="s.name"
              class="p-star"
              :class="{
                major: s.category === 'major',
                minor: s.category === 'minor',
                adjective: s.category === 'adjective',
                small: si >= 6,
              }"
              @pointerdown="onPressStart(s)"
              @pointerup="onPressEnd"
              @pointerleave="onPressEnd"
              @contextmenu.prevent="openStarIntro(s)"
          >
            <div
                v-for="(g, gi) in glyphs(s)"
                :key="gi"
                class="glyph"
                :class="g.cls"
            >{{ g.ch }}</div>
          </div>
        </div>
        <span v-if="palaceAge(i - 1)" class="p-age">{{ palaceAge(i - 1) }}</span>
        <div class="p-foot">
          <span class="p-ganzhi">{{ chart.palaces[i - 1].heavenlyStem }}{{ chart.palaces[i - 1].earthlyBranch }}</span>
          <span class="p-decade">{{ chart.palaces[i - 1].decadeStart }}-{{ chart.palaces[i - 1].decadeEnd }}</span>
          <span class="p-name">
            {{ chart.palaces[i - 1].name }}
            <span class="p-mark">
              {{ chart.palaces[i - 1].isSoul ? "命" : "" }}{{ chart.palaces[i - 1].isBody ? "身" : "" }}
            </span>
          </span>
        </div>
      </div>

      <div class="p-center">
        <div class="pc-name">{{ record?.name }}</div>
        <div class="pc-line"><span class="pc-label">阳历</span>{{ chart.solarText }} {{ chart.timeText }}</div>
        <div class="pc-line"><span class="pc-label">农历</span>{{ chart.lunarText }} {{ chart.timeText }}</div>
        <div class="pc-line"><span class="pc-label">性别</span>{{ record?.gender }}</div>
        <div class="pc-line"><span class="pc-label">八字</span>{{ chart.bazi.join(" ") }}</div>
        <div class="pc-line"><span class="pc-label">命主</span>{{ chart.soul }}<span class="pc-sep">身主</span>{{ chart.body }}</div>
        <div class="pc-flow">{{ flowYearText }} · 流年命宫 {{ flowYearPalaceName }}</div>
      </div>
    </div>

    <div v-if="chart" class="chart-controls">
      <div class="ctrl-block">
        <div class="ctrl-label">大限</div>
        <div class="decade-row">
          <button
              v-for="opt in decadeOptions"
              :key="opt.value"
              class="decade-chip"
              :class="{active: opt.value === decadeDist}"
              @click="decadeDist = opt.value; activeLevel = 'decade'"
          >{{ opt.label }}</button>
        </div>
      </div>
      <div class="ctrl-block">
        <div class="ctrl-label">流年</div>
        <n-select
            v-model:value="flowYear"
            :options="yearOptions"
            placeholder="未选择"
            size="small"
            style="flex: 1"
            @update:value="activeLevel = 'year'"
        />
      </div>
      <div class="ctrl-block">
        <div class="ctrl-label">流月</div>
        <n-select
            v-model:value="flowMonth"
            :options="monthOptions"
            placeholder="未选择"
            size="small"
            style="flex: 1"
            @update:value="activeLevel = 'month'"
        />
      </div>
      <div class="ctrl-block">
        <div class="ctrl-label">流日</div>
        <n-select
            v-model:value="flowDay"
            :options="dayOptions"
            placeholder="未选择"
            size="small"
            style="flex: 1"
            @update:value="activeLevel = 'day'"
        />
      </div>
    </div>

    <n-modal
        v-model:show="introShow"
        preset="card"
        :title="introStar"
        style="width: 86%; max-width: 440px"
    >
      <div class="intro-text">{{ introText }}</div>
    </n-modal>
  </div>
</template>

<style scoped>
.chart-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #f2f3f5;
}

.chart-topbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: #fff;
  border-bottom: 1px solid #eee;
}

.chart-name {
  font-weight: 700;
  color: #333;
  font-size: 15px;
}

.chart-grid {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  grid-template-rows: repeat(4, 1fr);
  gap: 2px;
  padding: 6px 8px;
  box-sizing: border-box;
}

.palace {
  background: #fffdf6;
  border: 1px solid #e8e2d0;
  border-radius: 4px;
  padding: 3px 4px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  cursor: pointer;
  position: relative;
  transition: background 0.15s;
  min-width: 0;
  min-height: 0;
}

.palace.hl-violet {
  background: #e6e4f9;
  border-color: #b5a9e8;
}

.palace.hl-red {
  background: #fdeaea;
  border-color: #efb1b1;
}

.p-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 3px;
  flex-shrink: 0;
  border-top: 1px dashed #e8e0cc;
  margin-top: 2px;
  padding-top: 2px;
  line-height: 1.2;
}

.p-name {
  font-weight: 700;
  font-size: 13px;
  color: #7a4a12;
  white-space: nowrap;
  line-height: 1.1;
}

.p-age {
  position: absolute;
  right: 5px;
  bottom: 22px;
  font-size: 10px;
  font-weight: 600;
  color: #1e8cd6;
  background: rgba(255, 255, 255, 0.75);
  border-radius: 2px;
  padding: 0 3px;
  line-height: 1.3;
  z-index: 1;
}

.p-ganzhi {
  font-size: 10px;
  color: #8a8070;
  white-space: nowrap;
}

.p-mark {
  font-size: 10px;
  color: #c0452b;
  font-weight: 700;
}

.p-decade {
  font-size: 10px;
  color: #bb9040;
  font-weight: 600;
  line-height: 1.2;
  white-space: nowrap;
}

.p-stars {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  align-content: flex-start;
  gap: 2px;
  overflow: hidden;
  padding: 2px 0;
}

.p-star {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex-shrink: 0;
  line-height: 1.12;
  border: 1px solid;
  border-radius: 3px;
  padding: 1px 3px;
}

.p-star.major {
  border-color: #e0a0a0;
  background: #fdf3f3;
}

.p-star.minor {
  border-color: #c3b3e8;
  background: #f6f3fc;
}

.p-star.adjective {
  border-color: #d8d2c4;
  background: #faf9f5;
}

.p-star .glyph {
  font-size: 12px;
  font-weight: 600;
  text-align: center;
}

.p-star.small .glyph {
  font-size: 10px;
}

.p-star.adjective .glyph {
  font-size: 11px;
}

.glyph.cat-major {
  color: #c02a2a;
}

.glyph.cat-minor {
  color: #6a4fd4;
}

.glyph.cat-adjective {
  color: #333;
}

.glyph.cat-bright {
  color: #8a8070;
  font-weight: 500;
}

.glyph.mut-科 {
  background: #1e8cd6;
  color: #fff;
  border-radius: 2px;
}

.glyph.mut-禄 {
  background: #2e9e52;
  color: #fff;
  border-radius: 2px;
}

.glyph.mut-权 {
  background: #8a5cd6;
  color: #fff;
  border-radius: 2px;
}

.glyph.mut-忌 {
  background: #d64545;
  color: #fff;
  border-radius: 2px;
}

.p-center {
  grid-row: 2 / 4;
  grid-column: 2 / 4;
  background: #fff8ec;
  border: 1px solid #e8dcc0;
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 6px;
  box-sizing: border-box;
  min-width: 0;
  min-height: 0;
}

.pc-name {
  font-size: 19px;
  font-weight: 800;
  color: #7a3f10;
}

.pc-line {
  font-size: 11px;
  color: #4a4035;
  line-height: 1.2;
}

.pc-label {
  color: #9a8768;
  margin-right: 4px;
}

.pc-sep {
  margin-left: 10px;
  color: #9a8768;
}

.pc-flow {
  font-size: 10px;
  color: #b07a30;
  font-weight: 600;
}

.chart-controls {
  background: #fff;
  border-top: 1px solid #eee;
  padding: 8px 12px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.ctrl-block {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ctrl-label {
  font-size: 12px;
  color: #666;
  width: 34px;
  flex-shrink: 0;
  font-weight: 600;
}

.decade-row {
  flex: 1;
  display: flex;
  flex-wrap: nowrap;
  gap: 4px;
  overflow-x: auto;
  overflow-y: hidden;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}

.decade-row::-webkit-scrollbar {
  display: none;
}

.decade-chip {
  font-size: 10px;
  padding: 2px 5px;
  border-radius: 3px;
  border: 1px solid #ddd;
  background: #fafafa;
  color: #666;
  cursor: pointer;
  line-height: 1.2;
  flex-shrink: 0;
}

.decade-chip.active {
  background: #1e8cd6;
  color: #fff;
  border-color: #1e8cd6;
}

.intro-text {
  font-size: 13px;
  color: #333;
  line-height: 1.7;
  max-height: 60vh;
  overflow-y: auto;
  white-space: pre-wrap;
}
</style>