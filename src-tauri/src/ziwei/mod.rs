pub mod data;
pub mod engine;
pub mod flow;

use serde::Serialize;

use chinese_lunisolar_calendar::{LunisolarDate, SolarDate};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BirthInput {
    /// 阳历 or 农历
    pub calendar_type: String,
    pub lunar: LunarInput,
    /// 出生日对应的阳历年月日（用于日柱）
    pub solar: (u16, u8, u8),
    pub time_index: u8,
    /// "男" | "女"
    pub gender: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LunarInput {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub is_leap: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Star {
    pub name: String,
    /// major | minor | adjective
    pub category: &'static str,
    pub brightness: String,
    /// 禄权科忌，无则空
    pub mutagen: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Palace {
    /// 寅序 0-11（寅=0）
    pub index: usize,
    pub name: String,
    pub is_body: bool,
    pub is_soul: bool,
    pub heavenly_stem: String,
    pub earthly_branch: String,
    /// 大限起始虚岁（含）
    pub decade_start: i32,
    /// 大限结束虚岁（含）
    pub decade_end: i32,
    pub major: Vec<Star>,
    pub minor: Vec<Star>,
    pub adjective: Vec<Star>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chart {
    pub birth: BirthInput,
    pub year_stem: String,
    pub year_branch: String,
    /// 命宫干支，如 壬午
    pub soul_stem: String,
    pub soul_branch: String,
    /// 身宫地支，如 戌
    pub body_branch: String,
    /// 五行局，如 木三局
    pub five_elements_class: String,
    /// 命主 / 身主
    pub soul: String,
    pub body: String,
    /// 时辰文本，如 寅时
    pub time_text: String,
    pub time_range: String,
    /// 农历文本，如 庚辰年七月十七日
    pub lunar_text: String,
    /// 阳历文本，如 2000-08-16
    pub solar_text: String,
    /// 八字四柱：年月日时
    pub bazi: [String; 4],
    pub palaces: Vec<Palace>,
}

const LUNAR_MONTH_CN: [&str; 12] = ["正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "冬月", "腊月"];
const CN_DIGITS: [&str; 10] = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九"];

/// 农历日数转中文，如 17 -> 十七，2 -> 二
fn cn_day(n: u8) -> String {
    match n {
        1..=9 => CN_DIGITS[n as usize].to_string(),
        10 => "十".to_string(),
        11..=19 => format!("十{}", CN_DIGITS[(n - 10) as usize]),
        20 => "二十".to_string(),
        21..=29 => format!("二十{}", CN_DIGITS[(n - 20) as usize]),
        30 => "三十".to_string(),
        _ => n.to_string(),
    }
}

/// 农历年文本，如 2000年二月二日
fn lunar_text(year: u16, month: u8, day: u8, is_leap: bool) -> String {
    let m = LUNAR_MONTH_CN[(month - 1).clamp(0, 11) as usize];
    format!(
        "{}年{}{}日",
        year,
        if is_leap { format!("闰{}", m) } else { m.to_string() },
        cn_day(day)
    )
}

/// 由农历数据起盘。time_index: 子=0 .. 亥=11
pub fn calculate_lunar(birth: BirthInput) -> Chart {
    let year = birth.lunar.year;
    let month_input = birth.lunar.month;
    let day_input = birth.lunar.day;
    let is_leap = birth.lunar.is_leap;
    let time = birth.time_index as usize;

    let (year_stem, year_branch) = engine::year_stem_branch(year);
    let month = engine::month_index(month_input, day_input, is_leap, true);
    let bazi = engine::get_bazi(year_stem, year_branch, month_input, birth.solar, time);
    let solar_text = engine::solar_text(birth.solar);

    // 命身宫
    let sb = engine::get_soul_and_body(year_stem, month, time);

    // 五行局（命宫干、支）
    let (_, class_name, class_val) = engine::get_five_elements_class(sb.soul_stem, sb.soul_branch);

    // 紫微 天府
    let (ziwei, tianfu) = engine::get_start_index(day_input, class_val);

    // 主星 辅星 杂曜
    let major = engine::get_major(ziwei, tianfu);
    let minor = engine::get_minor(year_stem, year_branch, month, time);
    let adjective = engine::get_adjective(
        year_stem,
        year_branch,
        sb.soul_index,
        sb.body_index,
        month,
        time,
        day_input,
    );

    // 命主 身主
    let soul = data::SOUL_BY_BRANCH[sb.soul_branch];
    let body = data::BODY_BY_BRANCH[year_branch];

    // 大限顺逆行：阳男阴女顺行，阴男阳女逆行
    let forward = (birth.gender == "男" && year_stem % 2 == 0)
        || (birth.gender == "女" && year_stem % 2 == 1);

    let mut palaces = Vec::with_capacity(12);
    for i in 0..12 {
        let stem = engine::fix(
            data::TIGER_RULE[year_stem] as i32 + i as i32,
            10,
        );
        let branch = engine::fix(i as i32 + 2, 12);
        let name = engine::palace_name(sb.soul_index, i);

        // 大限：命宫起，顺行 dist = fix(i - soul)，逆行 dist = fix(soul - i)
        let dist = if forward {
            engine::fix(i as i32 - sb.soul_index as i32, 12)
        } else {
            engine::fix(sb.soul_index as i32 - i as i32, 12)
        };
        let decade_start = class_val as i32 + 10 * dist as i32;

        palaces.push(Palace {
            index: i,
            name,
            is_soul: sb.soul_index == i,
            is_body: sb.body_index == i,
            heavenly_stem: data::STEMS[stem].to_string(),
            earthly_branch: data::BRANCHES[branch].to_string(),
            decade_start,
            decade_end: decade_start + 9,
            major: major[i]
                .iter()
                .map(|s| Star {
                    name: s.clone(),
                    category: "major",
                    brightness: engine::brightness(s, i).to_string(),
                    mutagen: engine::mutagen(s, year_stem).to_string(),
                })
                .collect(),
            minor: minor[i]
                .iter()
                .map(|s| Star {
                    name: s.clone(),
                    category: "minor",
                    brightness: engine::brightness(s, i).to_string(),
                    mutagen: {
                        let m = engine::mutagen(s, year_stem);
                        if matches!(s.as_str(), "左辅" | "右弼" | "文昌" | "文曲") {
                            m.to_string()
                        } else {
                            String::new()
                        }
                    },
                })
                .collect(),
            adjective: adjective[i]
                .iter()
                .map(|s| Star {
                    name: s.clone(),
                    category: "adjective",
                    brightness: String::new(),
                    mutagen: String::new(),
                })
                .collect(),
        });
    }

    Chart {
        birth,
        year_stem: data::STEMS[year_stem].to_string(),
        year_branch: data::BRANCHES[year_branch].to_string(),
        soul_stem: data::STEMS[sb.soul_stem].to_string(),
        soul_branch: data::BRANCHES[sb.soul_branch].to_string(),
        body_branch: data::BRANCHES[engine::fix(sb.body_index as i32 + 2, 12)].to_string(),
        five_elements_class: class_name.to_string(),
        soul: soul.to_string(),
        body: body.to_string(),
        time_text: format!("{}时", data::TIMES[time]),
        time_range: data::TIME_RANGES[time].to_string(),
        lunar_text: lunar_text(year, month_input, day_input, is_leap),
        solar_text,
        bazi,
        palaces,
    }
}

/// 由阳历数据起盘：先转农历再 calculate_lunar
pub fn calculate_solar(solar: (u16, u8, u8), time_index: u8, gender: &'static str) -> Result<Chart, String> {
    let (y, m, d) = solar;
    let solar_date = SolarDate::from_ymd(y, m, d).map_err(|e| format!("阳历日期无效: {}", e))?;
    let lunar = LunisolarDate::from_solar_date(solar_date)
        .map_err(|e| format!("农历转换失败: {}", e))?;

    let l_year = lunar.to_lunisolar_year().to_u16();
    let l_month = lunar.to_lunar_month().to_u8();
    let is_leap = lunar.to_lunar_month().is_leap_month();
    let l_day = lunar.to_lunar_day().to_u8();

    Ok(calculate_lunar(BirthInput {
        calendar_type: "solar".to_string(),
        lunar: LunarInput {
            year: l_year,
            month: l_month,
            day: l_day,
            is_leap,
        },
        solar: (y, m, d),
        time_index,
        gender: gender.to_string(),
    }))
}

#[cfg(test)]
mod tests;