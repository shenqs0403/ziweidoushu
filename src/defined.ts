export interface RecorderInfo {
    name: string;
    gender: string;
    birthday: number;
}

export interface BirthInput {
    calendarType: string;
    lunar: { year: number; month: number; day: number; isLeap: boolean };
    solar: [number, number, number];
    timeIndex: number;
    gender: string;
}

export interface Star {
    name: string;
    category: string;
    brightness: string;
    mutagen: string;
}

export interface Palace {
    index: number;
    name: string;
    isBody: boolean;
    isSoul: boolean;
    heavenlyStem: string;
    earthlyBranch: string;
    decadeStart: number;
    decadeEnd: number;
    major: Star[];
    minor: Star[];
    adjective: Star[];
}

export interface Chart {
    birth: BirthInput;
    yearStem: string;
    yearBranch: string;
    soulStem: string;
    soulBranch: string;
    bodyBranch: string;
    fiveElementsClass: string;
    soul: string;
    body: string;
    timeText: string;
    timeRange: string;
    lunarText: string;
    solarText: string;
    bazi: string[];
    palaces: Palace[];
}

export interface RecordInfo {
    id: number;
    name: string;
    calendarType: string;
    solarYear: number;
    solarMonth: number;
    solarDay: number;
    lunarYear: number;
    lunarMonth: number;
    lunarDay: number;
    isLeap: boolean;
    timeIndex: number;
    gender: string;
    yearStem: string;
    yearBranch: string;
    fiveElementsClass: string;
    mainStars: string;
    createdAt: number;
}

export interface FlowStar {
    palaceIndex: number;
    name: string;
}

export interface FlowAnnual {
    stem: string;
    branch: string;
    stars: FlowStar[];
}

export const STEMS = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
export const BRANCHES = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
export const TIMES = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

export const MUTAGEN_BY_STEM: string[][] = [
    ["廉贞", "破军", "武曲", "太阳"],
    ["天机", "天梁", "紫微", "太阴"],
    ["天同", "天机", "文昌", "廉贞"],
    ["太阴", "天同", "天机", "巨门"],
    ["贪狼", "太阴", "右弼", "天机"],
    ["武曲", "贪狼", "天梁", "文曲"],
    ["太阳", "武曲", "太阴", "天同"],
    ["巨门", "太阳", "文曲", "文昌"],
    ["天梁", "紫微", "左辅", "武曲"],
    ["破军", "巨门", "太阴", "贪狼"],
];

export const MUTAGEN_NAMES = ["禄", "权", "科", "忌"];