pub mod ziwei;

use serde::{Deserialize, Serialize};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Sqlite, SqlitePool};
use tauri::Manager;

use chinese_lunisolar_calendar::{
    LunisolarDate, LunisolarYear, LunarMonth, SolarDate, SolarYear,
};

use ziwei::{calculate_lunar, flow::FlowAnnual, BirthInput, Chart, LunarInput};

static MIGRATOR: Migrator = sqlx::migrate!();

struct AppState {
    pool: SqlitePool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecordInput {
    name: Option<String>,
    /// solar | lunar
    calendar_type: String,
    solar_year: Option<u16>,
    solar_month: Option<u8>,
    solar_day: Option<u8>,
    lunar_year: Option<u16>,
    lunar_month: Option<u8>,
    lunar_day: Option<u8>,
    is_leap: Option<bool>,
    /// 子=0 .. 亥=11
    time_index: u8,
    /// 男 | 女
    gender: Option<String>,
}

/// 归一化输入：无论按阳历还是农历输入，都得到 (阳历年月日, 农历数据)
fn normalize(input: &RecordInput) -> Result<(u16, u8, u8, LunarInput), String> {
    let solar = match input.calendar_type.as_str() {
        "solar" => {
            let (y, m, d) = (
                input.solar_year.ok_or("缺少阳历年份")?,
                input.solar_month.ok_or("缺少阳历月份")?,
                input.solar_day.ok_or("缺少阳历日期")?,
            );
            let solar_date = SolarDate::from_ymd(y, m, d).map_err(|e| format!("阳历日期无效: {}", e))?;
            let lunar = LunisolarDate::from_solar_date(solar_date)
                .map_err(|e| format!("农历转换失败: {}", e))?;
            let li = LunarInput {
                year: lunar.to_lunisolar_year().to_u16(),
                month: lunar.to_lunar_month().to_u8(),
                day: lunar.to_lunar_day().to_u8(),
                is_leap: lunar.to_lunar_month().is_leap_month(),
            };
            (y, m, d, li)
        }
        "lunar" => {
            let (y, m, d) = (
                input.lunar_year.ok_or("缺少农历年份")?,
                input.lunar_month.ok_or("缺少农历月份")?,
                input.lunar_day.ok_or("缺少农历日期")?,
            );
            let is_leap = input.is_leap.unwrap_or(false);
            let lunar = LunisolarDate::from_ymd(y, m, is_leap, d)
                .map_err(|e| format!("农历日期无效: {}", e))?;
            let solar_date = lunar.to_solar_date();
            let (sy, sm, sd) = (
                solar_date.to_solar_year().to_u16(),
                solar_date.to_solar_month().to_u8(),
                solar_date.to_solar_day().to_u8(),
            );
            let li = LunarInput { year: y, month: m, day: d, is_leap };
            (sy, sm, sd, li)
        }
        other => return Err(format!("未知的历法类型: {}", other)),
    };
    Ok(solar)
}

fn build_chart(input: &RecordInput) -> Result<Chart, String> {
    let (solar_year, solar_month, solar_day, lunar) = normalize(input)?;
    let gender = input.gender.clone().unwrap_or_default();
    Ok(calculate_lunar(BirthInput {
        calendar_type: input.calendar_type.clone(),
        lunar,
        solar: (solar_year, solar_month, solar_day),
        time_index: input.time_index,
        gender,
    }))
}

/// 命宫主星列表；命宫无主星时借对宫（用于列表摘要）
fn main_stars(chart: &Chart) -> String {
    let soul = chart.palaces.iter().find(|p| p.is_soul).unwrap();
    if !soul.major.is_empty() {
        return soul.major.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join("+");
    }
    let opposite = chart.palaces.iter().find(|p| (p.index + 6) % 12 == soul.index).unwrap();
    opposite.major.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join("+")
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    id: i64,
    name: String,
    calendar_type: String,
    solar_year: i64,
    solar_month: i64,
    solar_day: i64,
    lunar_year: i64,
    lunar_month: i64,
    lunar_day: i64,
    is_leap: bool,
    time_index: i64,
    gender: String,
    year_stem: String,
    year_branch: String,
    five_elements_class: String,
    main_stars: String,
    created_at: i64,
}

/// 实时计算命盘（不落库），前端可用于填表预览
#[tauri::command]
fn calculate_chart(input: RecordInput) -> Result<Chart, String> {
    build_chart(&input)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LunarYearInfo {
    year: u16,
    /// 闰月 1..=12，无闰月为 0
    leap_month: u8,
    /// 正月至腊月各月的天数（index 0..=11）
    month_days: Vec<u8>,
    /// 闰月天数，无闰月为 0
    leap_month_days: u8,
}

/// 农历某年（年号即农历年）的月信息：闰月及每月天数
#[tauri::command]
fn lunar_year_info(year: u16) -> Result<LunarYearInfo, String> {
    let ly = LunisolarYear::from_solar_year(SolarYear::from_u16(year))
        .map_err(|_| format!("农历年份超出支持范围: {}", year))?;
    let mut month_days = Vec::with_capacity(12);
    for m in 1..=12u8 {
        let lm = LunarMonth::from_u8_with_leap(m, false).map_err(|_| "月份非法".to_string())?;
        month_days.push(ly.get_total_days_in_a_month(lm).ok_or("月份天数计算失败")?);
    }
    let (leap_month, leap_month_days) = match ly.get_leap_lunar_month() {
        Some(lm) => (lm.to_u8(), ly.get_total_days_in_a_month(lm).unwrap_or(0)),
        None => (0, 0),
    };
    Ok(LunarYearInfo {
        year,
        leap_month,
        month_days,
        leap_month_days,
    })
}

#[tauri::command]
fn flow_annual(year: u16) -> FlowAnnual {
    ziwei::flow::flow_annual_stars(year)
}

#[tauri::command]
async fn add_record(
    state: tauri::State<'_, AppState>,
    input: RecordInput,
) -> Result<i64, String> {
    let (solar_year, solar_month, solar_day, lunar) = normalize(&input)?;
    let chart = build_chart(&input)?;
    let name = input.name.unwrap_or_default();
    let gender = input.gender.unwrap_or_default();

    let id = sqlx::query(
        "INSERT INTO records (
            name, calendar_type,
            solar_year, solar_month, solar_day,
            lunar_year, lunar_month, lunar_day, is_leap,
            time_index, gender,
            year_stem, year_branch, five_elements_class, main_stars
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(name)
    .bind(input.calendar_type)
    .bind(solar_year)
    .bind(solar_month)
    .bind(solar_day)
    .bind(lunar.year)
    .bind(lunar.month)
    .bind(lunar.day)
    .bind(lunar.is_leap)
    .bind(input.time_index)
    .bind(gender)
    .bind(&chart.year_stem)
    .bind(&chart.year_branch)
    .bind(&chart.five_elements_class)
    .bind(main_stars(&chart))
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    Ok(id)
}

#[derive(sqlx::FromRow)]
struct RecordRow {
    id: i64,
    name: String,
    calendar_type: String,
    solar_year: i64,
    solar_month: i64,
    solar_day: i64,
    lunar_year: i64,
    lunar_month: i64,
    lunar_day: i64,
    is_leap: bool,
    time_index: i64,
    gender: String,
    year_stem: String,
    year_branch: String,
    five_elements_class: String,
    main_stars: String,
    created_at: i64,
}

fn row_to_record(r: RecordRow) -> Record {
    Record {
        id: r.id,
        name: r.name,
        calendar_type: r.calendar_type,
        solar_year: r.solar_year,
        solar_month: r.solar_month,
        solar_day: r.solar_day,
        lunar_year: r.lunar_year,
        lunar_month: r.lunar_month,
        lunar_day: r.lunar_day,
        is_leap: r.is_leap,
        time_index: r.time_index,
        gender: r.gender,
        year_stem: r.year_stem,
        year_branch: r.year_branch,
        five_elements_class: r.five_elements_class,
        main_stars: r.main_stars,
        created_at: r.created_at,
    }
}

#[tauri::command]
async fn list_records(state: tauri::State<'_, AppState>) -> Result<Vec<Record>, String> {
    let rows = sqlx::query_as::<Sqlite, RecordRow>(
        "SELECT id, name, calendar_type,
                solar_year, solar_month, solar_day,
                lunar_year, lunar_month, lunar_day, is_leap,
                time_index, gender,
                year_stem, year_branch, five_elements_class, main_stars,
                created_at
         FROM records ORDER BY id DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(row_to_record).collect())
}

#[tauri::command]
async fn delete_record(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM records WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn chart_for_record(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<(Record, Chart), String> {
    let row = sqlx::query_as::<Sqlite, RecordRow>(
        "SELECT id, name, calendar_type,
                solar_year, solar_month, solar_day,
                lunar_year, lunar_month, lunar_day, is_leap,
                time_index, gender,
                year_stem, year_branch, five_elements_class, main_stars,
                created_at
         FROM records WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "记录不存在".to_string())?;

    let input = RecordInput {
        name: Some(row.name.clone()),
        calendar_type: row.calendar_type.clone(),
        solar_year: Some(row.solar_year as u16),
        solar_month: Some(row.solar_month as u8),
        solar_day: Some(row.solar_day as u8),
        lunar_year: Some(row.lunar_year as u16),
        lunar_month: Some(row.lunar_month as u8),
        lunar_day: Some(row.lunar_day as u8),
        is_leap: Some(row.is_leap),
        time_index: row.time_index as u8,
        gender: Some(row.gender.clone()),
    };

    let chart = build_chart(&input)?;
    Ok((row_to_record(row), chart))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&db_dir).expect("failed to create app data dir");
            let db_path = db_dir.join("ziweidoushu.db");

            let pool = tauri::async_runtime::block_on(async {
                let opts = sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(&db_path)
                    .create_if_missing(true);
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect_with(opts)
                    .await
                    .expect("failed to connect to sqlite");
                MIGRATOR.run(&pool).await.expect("failed to run migrations");
                pool
            });

            app.manage(AppState { pool });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            calculate_chart,
            add_record,
            list_records,
            delete_record,
            chart_for_record,
            flow_annual,
            lunar_year_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}