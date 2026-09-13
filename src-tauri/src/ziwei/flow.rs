use super::data::*;
use super::engine::fix;
use serde::Serialize;

/// 流曜（寅序宫位落点）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowStar {
    pub palace_index: usize,
    pub name: String,
}

/// 某一流年（农历年）的流曜盘
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowAnnual {
    pub stem: String,
    pub branch: String,
    pub stars: Vec<FlowStar>,
}

/// 求流年干支及流曜落点（寅序，0=寅）
pub fn flow_annual_stars(lunar_year: u16) -> FlowAnnual {
    let stem = fix(lunar_year as i32 - 4, 10);
    let branch = fix(lunar_year as i32 - 4, 12);

    let mut stars: Vec<FlowStar> = Vec::new();
    let mut add = |palace: usize, name: &str| {
        stars.push(FlowStar {
            palace_index: palace,
            name: name.to_string(),
        });
    };

    // 年干系
    add(FLOW_CHANG[stem], "流昌");
    add(FLOW_QU[stem], "流曲");
    add(FLOW_KUI[stem], "流魁");
    add(FLOW_YUE[stem], "流钺");
    let lu = FLOW_LU[stem];
    add(lu, "流禄");
    add(fix(lu as i32 + 1, 12), "流羊");
    add(fix(lu as i32 - 1, 12), "流陀");

    // 年支系
    add(FLOW_MA[branch], "流马");
    let luan = fix(1 - branch as i32, 12); // 卯起子逆数
    add(luan, "流鸾");
    add(fix(luan as i32 + 6, 12), "流喜");
    add(FLOW_HUAGAI[branch], "流华盖");
    add(FLOW_XIANCHI[branch], "流咸池");

    FlowAnnual {
        stem: STEMS[stem].to_string(),
        branch: BRANCHES[branch].to_string(),
        stars,
    }
}