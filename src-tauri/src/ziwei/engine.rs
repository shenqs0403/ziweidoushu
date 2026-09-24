use super::data::*;

/// 模运算（i mod max，结果 0~max-1）
pub fn fix(index: i32, max: usize) -> usize {
    let m = max as i32;
    let mut i = index % m;
    if i < 0 {
        i += m;
    }
    i as usize
}

/// 地支名 -> 寅序（寅=0）
fn br(name: &str) -> usize {
    let idx = BRANCHES.iter().position(|b| *b == name).unwrap();
    fix(idx as i32 - 2, 12)
}

/// 农历年干支下标（正月初一分界）
pub fn year_stem_branch(lunar_year: u16) -> (usize, usize) {
    let y = lunar_year as i32;
    (fix(y - 4, 10), fix(y - 4, 12))
}

/// 月索引（寅序，0 代表正月），闰月前 15 天按上月、之后按下月
pub fn month_index(lunar_month: u8, lunar_day: u8, is_leap: bool, fix_leap: bool) -> usize {
    let m = fix(lunar_month as i32 - 1, 12);
    if is_leap && fix_leap && lunar_day > 15 {
        fix(m as i32 + 1, 12)
    } else {
        m
    }
}

/// 命身宫
pub struct SoulAndBody {
    pub soul_index: usize, // 寅序
    pub body_index: usize, // 寅序
    /// 命宫干下标（十天干）
    pub soul_stem: usize,
    /// 命宫支下标（子=0 十二地支）
    pub soul_branch: usize,
}

/// 命身宫：月索引（寅序）+ 时辰（子=0）在“寅位正月”下运算
pub fn get_soul_and_body(year_stem: usize, month: usize, time: usize) -> SoulAndBody {
    let soul_index = fix(month as i32 - time as i32, 12);
    let body_index = fix(month as i32 + time as i32, 12);

    let tiger = TIGER_RULE[year_stem];
    let soul_stem = fix(tiger as i32 + soul_index as i32, 10);
    let soul_branch = fix(soul_index as i32 + 2, 12);

    SoulAndBody {
        soul_index,
        body_index,
        soul_stem,
        soul_branch,
    }
}

/// 定五行局，返回（表下标, 局名, 局数）
pub fn get_five_elements_class(stem: usize, branch: usize) -> (usize, &'static str, u8) {
    let stem_num = stem / 2 + 1;
    let branch_num = (branch % 6) / 2 + 1;
    let mut idx = stem_num + branch_num;
    while idx > 5 {
        idx -= 5;
    }
    (idx - 1, FIVE_ELEMENTS[idx - 1].0, FIVE_ELEMENTS[idx - 1].1)
}

/// 紫微 / 天府宫位（寅序）
pub fn get_start_index(lunar_day: u8, class: u8) -> (usize, usize) {
    let day = lunar_day as i32;
    let mut offset = -1i32;
    let mut quotient;
    loop {
        offset += 1;
        let divisor = day + offset;
        quotient = divisor / class as i32;
        if divisor % class as i32 == 0 {
            break;
        }
    }
    quotient %= 12;
    let mut ziwei = quotient - 1;
    if offset % 2 == 0 {
        ziwei += offset;
    } else {
        ziwei -= offset;
    }
    let ziwei_index = fix(ziwei, 12);
    let tianfu_index = 12 - ziwei_index;
    (ziwei_index, tianfu_index)
}

/// 宫名：寅序 i -> 宫名
pub fn palace_name(soul_index: usize, i: usize) -> String {
    let idx = fix(i as i32 - soul_index as i32, 12);
    PALACES[idx].to_string()
}

/// 儒略日（proleptic Gregorian），用于干支日计算
fn days_from_civil(y: i64, m: u64, d: u64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

/// 八字四柱（年月日时），返回成形干支字符串数组。
/// 年柱按农历年（与新盘一致）；月柱按农历月 + 五虎遁（闰月取当月）；
/// 日柱以 1900-01-01（甲戌日）为基准；时柱按五鼠遁（晚子时视作当日/当日子时）。
/// solar: 出生日对应的阳历年月日（公历）。
pub fn get_bazi(
    year_stem: usize,
    year_branch: usize,
    lunar_month: u8,
    solar: (u16, u8, u8),
    time: usize,
) -> [String; 4] {
    // 年柱
    let y = format!("{}{}", STEMS[year_stem], BRANCHES[year_branch]);

    // 月柱：五虎遁，正月起寅
    let m_index = lunar_month as i32 - 1;
    let m_stem = fix(TIGER_RULE[year_stem] as i32 + m_index, 10);
    let m_branch = fix(2 + m_index, 12);
    let m = format!("{}{}", STEMS[m_stem], BRANCHES[m_branch]);

    // 日柱
    let (sy, sm, sd) = solar;
    let diff =
        days_from_civil(sy as i64, sm as u64, sd as u64) - days_from_civil(1900, 1, 1);
    let d_stem = fix(diff as i32, 10);
    let d_branch = fix((diff + 10) as i32, 12);
    let d = format!("{}{}", STEMS[d_stem], BRANCHES[d_branch]);

    // 时柱：五鼠遁，子时起
    let h_stem = fix(RAT_RULE[d_stem] as i32 + time as i32, 10);
    let h = format!("{}{}", STEMS[h_stem], BRANCHES[time]);

    [y, m, d, h]
}

/// 阳历文本，如 2000年8月16日
pub fn solar_text(solar: (u16, u8, u8)) -> String {
    format!("{}年{}月{}日", solar.0, solar.1, solar.2)
}

/// 亮度（寅序）
pub fn brightness(star: &str, idx: usize) -> &'static str {
    for (name, arr) in BRIGHTNESS {
        if name == star {
            return arr[idx];
        }
    }
    ""
}

/// 四化：返回 "禄"/"权"/"科"/"忌"，无则空
pub fn mutagen(star: &str, year_stem: usize) -> &'static str {
    for (i, s) in MUTAGEN_BY_STEM[year_stem].iter().enumerate() {
        if *s == star {
            return MUTAGEN_NAMES[i];
        }
    }
    ""
}

/// 14 主星：12 宫（寅序）主星列表
pub fn get_major(ziwei: usize, tianfu: usize) -> [Vec<String>; 12] {
    let mut stars: [Vec<String>; 12] = std::array::from_fn(|_| Vec::new());
    for (name, i) in ZIWEI_GROUP {
        let p = fix(ziwei as i32 - i as i32, 12);
        stars[p].push(name.to_string());
    }
    for (name, i) in TIANFU_GROUP {
        let p = fix(tianfu as i32 + i as i32, 12);
        stars[p].push(name.to_string());
    }
    stars
}

/// 14 辅星：12 宫（寅序）辅星列表
pub fn get_minor(
    year_stem: usize,
    year_branch: usize,
    month: usize, // 月索引（已闰月校正）
    time: usize,
) -> [Vec<String>; 12] {
    let mut stars: [Vec<String>; 12] = std::array::from_fn(|_| Vec::new());
    let mut add = |p: usize, name: &str| stars[p].push(name.to_string());

    // 左辅 辰起（卯宫=寅序2）顺数月；右弼 戌起（寅序8）逆数月
    let zuo = fix(2 + month as i32, 12);
    let you = fix(8 - month as i32, 12);
    // 文昌 戌起（寅序8）逆数时；文曲 辰起（寅序2）顺数时
    let chang = fix(8 - time as i32, 12);
    let qu = fix(2 + time as i32, 12);
    // 天魁 天钺（年干）
    let (kui, yue) = get_kui_yue(year_stem);
    // 禄存 天马
    let (lu, ma) = get_lu_ma(year_stem, year_branch);
    let yang = fix(lu as i32 + 1, 12);
    let tuo = fix(lu as i32 - 1, 12);
    // 火星 铃星（年支 + 时）
    let (huo, ling) = get_huo_ling(year_branch, time);
    // 地空 地劫：亥起子时，地劫顺、地空逆
    let kong = fix(9 - time as i32, 12);
    let jie = fix(9 + time as i32, 12);

    add(zuo, "左辅");
    add(you, "右弼");
    add(chang, "文昌");
    add(qu, "文曲");
    add(kui, "天魁");
    add(yue, "天钺");
    add(lu, "禄存");
    add(ma, "天马");
    add(kong, "地空");
    add(jie, "地劫");
    add(huo, "火星");
    add(ling, "铃星");
    add(yang, "擎羊");
    add(tuo, "陀罗");

    stars
}

/// 天魁 天钺（年干）
fn get_kui_yue(stem: usize) -> (usize, usize) {
    match stem {
        0 | 4 | 6 => (br("丑"), br("未")), // 甲戊庚
        1 | 5 => (br("子"), br("申")),     // 乙己
        7 => (br("午"), br("寅")),          // 辛
        2 | 3 => (br("亥"), br("酉")),     // 丙丁
        _ => (br("卯"), br("巳")),          // 壬癸
    }
}

/// 禄存 天马（年干，年支）
fn get_lu_ma(stem: usize, branch: usize) -> (usize, usize) {
    let lu = match stem {
        0 => br("寅"),        // 甲
        1 => br("卯"),        // 乙
        2 | 4 => br("巳"),    // 丙戊
        3 | 5 => br("午"),    // 丁己
        6 => br("申"),        // 庚
        7 => br("酉"),        // 辛
        8 => br("亥"),        // 壬
        _ => br("子"),        // 癸
    };
    let ma = match branch {
        2 | 6 | 10 => br("申"),  // 寅午戌
        8 | 0 | 4 => br("寅"),   // 申子辰
        5 | 9 | 1 => br("亥"),   // 巳酉丑
        _ => br("巳"),           // 亥卯未
    };
    (lu, ma)
}

/// 火星 铃星（年支 + 时）
fn get_huo_ling(branch: usize, time: usize) -> (usize, usize) {
    let (huo, ling) = match branch {
        2 | 6 | 10 => (br("丑"), br("卯")), // 寅午戌
        8 | 0 | 4 => (br("寅"), br("戌")),  // 申子辰
        5 | 9 | 1 => (br("卯"), br("戌")),  // 巳酉丑
        _ => (br("酉"), br("戌")),          // 亥卯未
    };
    (fix(huo as i32 + time as i32, 12), fix(ling as i32 + time as i32, 12))
}

/// 38 杂曜：12 宫（寅序）杂曜列表
pub fn get_adjective(
    year_stem: usize,
    year_branch: usize,
    soul_index: usize,
    body_index: usize,
    month: usize, // 月索引（已闰月校正）
    time: usize,
    lunar_day: u8,
) -> [Vec<String>; 12] {
    let mut stars: [Vec<String>; 12] = std::array::from_fn(|_| Vec::new());
    let mut add = |p: usize, name: &str| stars[p].push(name.to_string());

    // ============ 月系 ============
    // 月解：正二申 三四戌 五六子 七八寅 九十辰 十一十二午
    let yuejie = ["申", "戌", "子", "寅", "辰", "午"][month / 2];
    add(br(yuejie), "解神");
    add(fix(11 + month as i32, 12), "天姚"); // 天姚 丑起
    add(fix(7 + month as i32, 12), "天刑"); // 天刑 酉起
    let yinsha = ["寅", "子", "戌", "申", "午", "辰"][month % 6];
    add(br(yinsha), "阴煞");
    let tianyue = ["戌", "巳", "辰", "寅", "未", "卯", "亥", "未", "寅", "午", "戌", "寅"][month];
    add(br(tianyue), "天月");
    let tianwu = ["巳", "申", "寅", "亥"][month % 4];
    add(br(tianwu), "天巫");

    // ============ 日系 ============
    let day = lunar_day as i32 - 1;
    let zuo = fix(2 + month as i32, 12); // 左辅
    let you = fix(8 - month as i32, 12); // 右弼
    let chang = fix(8 - time as i32, 12); // 文昌
    let qu = fix(2 + time as i32, 12); // 文曲
    add(fix(zuo as i32 + day, 12), "三台");
    add(fix(you as i32 - day, 12), "八座");
    add(fix(((chang as i32 + day) % 12) - 1, 12), "恩光");
    add(fix(((qu as i32 + day) % 12) - 1, 12), "天贵");

    // ============ 时系 ============
    add(fix(4 + time as i32, 12), "台辅"); // 午起
    add(fix(0 + time as i32, 12), "封诰"); // 寅起

    // ============ 年系（年支）============
    // 红鸾：卯起子逆数至生年支；天喜对宫
    let hongluan = fix(1 - year_branch as i32, 12);
    add(hongluan, "红鸾");
    add(fix(hongluan as i32 + 6, 12), "天喜");
    // 华盖 咸池
    let (hg, xc) = get_hg_xc(year_branch);
    add(hg, "华盖");
    add(xc, "咸池");
    // 孤辰 寡宿
    let (gu, gua) = get_gu_gua(year_branch);
    add(gu, "孤辰");
    add(gua, "寡宿");
    // 天才 天寿：命/身宫起子顺数至生年支
    add(fix(soul_index as i32 + year_branch as i32, 12), "天才");
    add(fix(body_index as i32 + year_branch as i32, 12), "天寿");
    // 龙池：辰起顺；凤阁：戌起逆
    add(fix(2 + year_branch as i32, 12), "龙池");
    add(fix(8 - year_branch as i32, 12), "凤阁");
    // 天哭 天虚：午起子，哭逆行虚顺转
    add(fix(4 - year_branch as i32, 12), "天哭");
    add(fix(4 + year_branch as i32, 12), "天虚");
    // 天德（酉起顺）月德（巳起顺）
    add(fix(7 + year_branch as i32, 12), "天德");
    add(fix(3 + year_branch as i32, 12), "月德");
    // 天空：生年支顺数年支前一位
    add(fix(zodiac_to_palace(year_branch) as i32 + 1, 12), "天空");
    // 年解：戌起子逆数至生年支
    add(fix(8 - year_branch as i32, 12), "年解");

    // ============ 年系（干支组合）============
    // 天厨（年干）
    let tianchu = ["巳", "午", "子", "巳", "午", "申", "寅", "午", "酉", "亥"][year_stem];
    add(br(tianchu), "天厨");
    // 天官（年干）
    let tianguan = ["未", "辰", "巳", "寅", "卯", "酉", "亥", "酉", "戌", "午"][year_stem];
    add(br(tianguan), "天官");
    // 天福（年干）
    let tianfu = ["酉", "申", "子", "亥", "卯", "寅", "午", "巳", "午", "巳"][year_stem];
    add(br(tianfu), "天福");
    // 破碎（年支%3）
    let posui = ["巳", "丑", "酉"][year_branch % 3];
    add(br(posui), "破碎");
    // 蜚廉（年支）
    let feilian = ["申", "酉", "戌", "巳", "午", "未", "寅", "卯", "辰", "亥", "子", "丑"][year_branch];
    add(br(feilian), "蜚廉");
    // 截路（年干%5）
    let jielu = ["申", "午", "辰", "寅", "子"][year_stem % 5];
    add(br(jielu), "截路");
    // 空亡（年干%5）
    let kongwang = ["酉", "未", "巳", "卯", "丑"][year_stem % 5];
    add(br(kongwang), "空亡");
    // 旬空（年干支）
    let xunkong = xunkong_index(year_stem, year_branch);
    add(xunkong, "旬空");

    // ============ 天使 天伤 ============
    add(fix(7 + soul_index as i32, 12), "天使"); // 疾厄之相位
    add(fix(5 + soul_index as i32, 12), "天伤"); // 仆役之相位

    stars
}

/// 旬空所在宫位（寅序）
fn xunkong_index(stem: usize, branch: usize) -> usize {
    let mut idx = fix(zodiac_to_palace(branch) as i32 + 9 - stem as i32 + 1, 12);
    if branch % 2 != idx % 2 {
        idx = fix(idx as i32 + 1, 12);
    }
    idx
}

/// 地支名 -> 寅序
fn zodiac_to_palace(branch: usize) -> usize {
    fix(branch as i32 - 2, 12)
}

/// 华盖 咸池（年支）
fn get_hg_xc(branch: usize) -> (usize, usize) {
    match branch {
        2 | 6 | 10 => (br("戌"), br("卯")), // 寅午戌
        8 | 0 | 4 => (br("辰"), br("酉")),  // 申子辰
        5 | 9 | 1 => (br("丑"), br("午")),  // 巳酉丑
        _ => (br("未"), br("子")),          // 亥卯未
    }
}

/// 孤辰 寡宿（年支）
fn get_gu_gua(branch: usize) -> (usize, usize) {
    match branch {
        2 | 3 | 4 => (br("巳"), br("丑")),   // 寅卯辰
        5 | 6 | 7 => (br("申"), br("辰")),   // 巳午未
        8 | 9 | 10 => (br("亥"), br("未")), // 申酉戌
        _ => (br("寅"), br("戌")),           // 亥子丑
    }
}