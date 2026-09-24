use super::*;

fn sample() -> Chart {
    calculate_solar((2000, 8, 16), 2, "女").expect("sample chart")
}

fn p(c: &Chart, i: usize) -> &Palace {
    &c.palaces[i]
}

/// 阳历转农历：2000-8-16 -> 农历七月十七
#[test]
fn lunar_conversion() {
    let solar = SolarDate::from_ymd(2000, 8, 16).unwrap();
    let lunar = LunisolarDate::from_solar_date(solar).unwrap();
    assert_eq!(lunar.to_lunisolar_year().to_u16(), 2000);
    assert_eq!(lunar.to_lunar_month().to_u8(), 7);
    assert_eq!(lunar.to_lunar_month().is_leap_month(), false);
    assert_eq!(lunar.to_lunar_day().to_u8(), 17);
}

#[test]
fn basic_info() {
    let c = sample();
    assert_eq!(c.year_stem, "庚");
    assert_eq!(c.year_branch, "辰");
    assert_eq!(c.five_elements_class, "木三局");
    assert_eq!(c.soul_stem, "壬");
    assert_eq!(c.soul_branch, "午");
    assert_eq!(c.body_branch, "戌");
    // 命主（命宫地支午）/ 身主（年支辰）
    assert_eq!(c.soul, "破军");
    assert_eq!(c.body, "文昌");
    assert_eq!(c.lunar_text, "2000年七月十七日");
}

#[test]
fn soul_body_palaces() {
    let c = sample();
    // 命宫：寅序4（午）
    let soul = c.palaces.iter().find(|p| p.is_soul).unwrap();
    assert_eq!(soul.name, "命宫");
    assert_eq!(soul.index, 4);
    assert_eq!(soul.earthly_branch, "午");
    assert_eq!(soul.heavenly_stem, "壬");
    // 身宫：寅序8（戌）为官禄
    let body = c.palaces.iter().find(|p| p.is_body).unwrap();
    assert_eq!(body.index, 8);
    assert_eq!(body.name, "官禄");
    assert_eq!(body.earthly_branch, "戌");
}

#[test]
fn palace_names_and_branches() {
    let c = sample();
    let expect: [(&str, &str); 12] = [
        ("财帛", "寅"),
        ("子女", "卯"),
        ("夫妻", "辰"),
        ("兄弟", "巳"),
        ("命宫", "午"),
        ("父母", "未"),
        ("福德", "申"),
        ("田宅", "酉"),
        ("官禄", "戌"),
        ("仆役", "亥"),
        ("迁移", "子"),
        ("疾厄", "丑"),
    ];
    for (i, (name, branch)) in expect.iter().enumerate() {
        assert_eq!(p(&c, i).name, *name, "palace {i} name");
        assert_eq!(p(&c, i).earthly_branch, *branch, "palace {i} branch");
    }
    // 宫干从寅(戊寅)顺行
    let stems: Vec<&str> = (0..4).map(|i| p(&c, i).heavenly_stem.as_str()).collect();
    assert_eq!(stems, ["戊", "己", "庚", "辛"]);
    assert_eq!(p(&c, 4).heavenly_stem, "壬");
}

#[test]
fn major_stars() {
    let c = sample();
    let major = |i: usize| {
        p(&c, i)
            .major
            .iter()
            .map(|s| (s.name.clone(), s.brightness.clone(), s.mutagen.clone()))
            .collect::<Vec<_>>()
    };
    // 午 命宫
    assert_eq!(major(4), vec![("紫微".into(), "庙".into(), "".into())]);
    // 巳 兄弟
    assert_eq!(major(3), vec![("天机".into(), "平".into(), "".into())]);
    // 戌 官禄（身宫）
    assert_eq!(
        major(8),
        vec![("廉贞".into(), "利".into(), "".into()), ("天府".into(), "庙".into(), "".into())]
    );
    // 辰 夫妻
    assert_eq!(major(2), vec![("七杀".into(), "庙".into(), "".into())]);
    // 申 福德
    assert_eq!(major(6), vec![("破军".into(), "得".into(), "".into())]);
    // 亥 仆役
    assert_eq!(major(9), vec![("太阴".into(), "庙".into(), "科".into())]);
    // 子 迁移
    assert_eq!(major(10), vec![("贪狼".into(), "旺".into(), "".into())]);
    // 丑 疾厄
    assert_eq!(major(11), vec![("天同".into(), "不".into(), "忌".into()), ("巨门".into(), "不".into(), "".into())]);
}

#[test]
fn minor_stars() {
    let c = sample();
    let minor = |i: usize| p(&c, i).minor.iter().map(|s| s.name.clone()).collect::<Vec<_>>();
    assert_eq!(minor(8), vec!["左辅"]);   // 官禄戌
    assert_eq!(minor(2), vec!["右弼", "火星"]); // 夫妻辰
    assert_eq!(minor(6), vec!["文昌", "禄存"]); // 福德申
    assert_eq!(minor(4), vec!["文曲"]);   // 命宫午
    assert_eq!(minor(11), vec!["天魁", "地劫"]); // 疾厄丑
    assert_eq!(minor(5), vec!["天钺", "陀罗"]); // 父母未
    assert_eq!(minor(7), vec!["地空", "擎羊"]); // 田宅酉
    assert_eq!(minor(10), vec!["铃星"]);  // 迁移子
    assert_eq!(minor(0), vec!["天马"]);   // 财帛寅
    // 亮度
    let wenqu = p(&c, 4).minor.iter().find(|s| s.name == "文曲").unwrap();
    assert_eq!(wenqu.brightness, "陷");
    let chang = p(&c, 6).minor.iter().find(|s| s.name == "文昌").unwrap();
    assert_eq!(chang.brightness, "得");
    let huo = p(&c, 2).minor.iter().find(|s| s.name == "火星").unwrap();
    assert_eq!(huo.brightness, "陷");
    let ling = p(&c, 10).minor.iter().find(|s| s.name == "铃星").unwrap();
    assert_eq!(ling.brightness, "陷");
}

#[test]
fn adjective_stars() {
    let c = sample();
    let adj = |i: usize| {
        let mut v: Vec<String> = p(&c, i).adjective.iter().map(|s| s.name.clone()).collect();
        v.sort();
        v
    };
    assert_eq!(adj(9), vec!["天伤", "天官", "天月", "恩光", "红鸾"]); // 仆役亥
    assert_eq!(adj(3), vec!["天喜", "天空", "孤辰"]); // 兄弟巳
    assert_eq!(adj(11), vec!["天使", "天德", "寡宿", "破碎"]); // 疾厄丑
    assert_eq!(adj(2), vec!["华盖", "封诰"]); // 夫妻辰
    assert_eq!(adj(6), vec!["台辅", "旬空", "龙池"]); // 福德申
    assert_eq!(adj(10), vec!["八座"]); // 迁移子
    // 命宫午
    let mut soul_adj: Vec<String> = p(&c, 4).adjective.iter().map(|s| s.name.clone()).collect();
    soul_adj.sort();
    assert_eq!(soul_adj, vec!["凤阁", "天福", "年解", "截路", "蜚廉"]);
}

#[test]
fn time_text() {
    let c = sample();
    assert_eq!(c.time_text, "寅时");
    assert_eq!(c.time_range, "03:00~05:00");
}

#[test]
fn bazi_anchors() {
    // 日柱基准（权威记载）：1900-01-01 甲戌日；1986-05-29 癸酉日
    let b1 = super::engine::get_bazi(0, 0, 1, (1900, 1, 1), 0);
    assert_eq!(b1[2], "甲戌");
    let b2 = super::engine::get_bazi(2, 2, 4, (1986, 5, 29), 0);
    assert_eq!(b2[2], "癸酉");
}

#[test]
fn bazi_sample() {
    let c = sample();
    // 庚辰年 七月
    assert_eq!(c.bazi[0], "庚辰");
    assert_eq!(c.bazi[1], "甲申");
    assert_eq!(c.solar_text, "2000年8月16日");
    // 时干 = 五鼠遁(日干) + 时辰；2000-08-16 日柱应满足该关系
    let day = &c.bazi[2];
    let day_stem = super::data::STEMS
        .iter()
        .position(|s| s.chars().next() == day.chars().next())
        .unwrap();
    let rat = super::data::RAT_RULE[day_stem];
    let hour_stem = super::data::STEMS[super::engine::fix(rat as i32 + 2, 10)];
    assert_eq!(c.bazi[3], format!("{}{}", hour_stem, super::data::BRANCHES[2]));
}

#[test]
fn decades_map() {
    // 女 庚辰年：阴年？庚为阳干，阳女逆行 → 命宫起逆数
    let c = sample();
    let soul = c.palaces.iter().find(|p| p.is_soul).unwrap();
    assert_eq!(soul.index, 4);
    // 木三局，命宫大限 3-12
    assert_eq!(soul.decade_start, 3);
    assert_eq!(soul.decade_end, 12);
    // 逆行：命宫(4) 下一段至 兄弟(3)
    let next = c.palaces.iter().find(|p| p.index == 3).unwrap();
    assert_eq!(next.name, "兄弟");
    assert_eq!(next.decade_start, 13);
    assert_eq!(next.decade_end, 22);
    // 最后一段 113-122 落于 父母(5)
    let last = c.palaces.iter().find(|p| p.index == 5).unwrap();
    assert_eq!(last.name, "父母");
    assert_eq!(last.decade_start, 113);
    assert_eq!(last.decade_end, 122);
}

#[test]
fn forward_direction() {
    // 男 庚辰年：阳男顺行 → 命宫(4) 下一段至 父母(5)
    let c = calculate_solar((2000, 8, 16), 2, "男").expect("sample chart");
    let soul = c.palaces.iter().find(|p| p.is_soul).unwrap();
    assert_eq!(soul.decade_start, 3);
    let next = c.palaces.iter().find(|p| p.index == 5).unwrap();
    assert_eq!(next.name, "父母");
    assert_eq!(next.decade_start, 13);
}

#[test]
fn flow_stars_placement() {
    // 庚辰年：庚=6，辰=4
    use super::flow;
    let fa = flow::flow_annual_stars(2000);
    assert_eq!(fa.stem, "庚");
    assert_eq!(fa.branch, "辰");
    let names = |palace: usize| {
        let mut v: Vec<&str> = fa
            .stars
            .iter()
            .filter(|s| s.palace_index == palace)
            .map(|s| s.name.as_str())
            .collect();
        v.sort();
        v
    };
    assert_eq!(names(9), vec!["流昌", "流鸾"]); // 庚流昌亥，辰流鸾亥
    assert_eq!(names(10), vec!["流曲"]);         // 庚流曲子
    assert_eq!(names(11), vec!["流魁"]);         // 庚流魁丑
    assert_eq!(names(5), vec!["流钺", "流陀"]);  // 庚流钺未，流陀未
    assert_eq!(names(6), vec!["流禄"]);          // 庚流禄申
    assert_eq!(names(7), vec!["流咸池", "流羊"]); // 流羊酉，辰咸池酉
    assert_eq!(names(0), vec!["流马"]);          // 辰马在寅
    assert_eq!(names(3), vec!["流喜"]);          // 流喜巳
    assert_eq!(names(2), vec!["流华盖"]);        // 辰华盖辰
}