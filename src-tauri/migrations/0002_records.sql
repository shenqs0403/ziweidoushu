-- 紫微斗数命盘记录表
-- 存储出生信息的输入与关键盘面缓存，命盘本身由引擎按需重算（确定性逻辑）
CREATE TABLE IF NOT EXISTS records (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT    NOT NULL DEFAULT '',
    -- solar: 按阳历输入；lunar: 按农历输入
    calendar_type       TEXT    NOT NULL DEFAULT 'solar' CHECK (calendar_type IN ('solar', 'lunar')),
    -- 阳历日期（农历输入时由转换结果得到，用于列表展示与排序）
    solar_year          INTEGER NOT NULL,
    solar_month         INTEGER NOT NULL,
    solar_day           INTEGER NOT NULL,
    -- 农历数据（排盘引擎直接使用）
    lunar_year          INTEGER NOT NULL,
    lunar_month         INTEGER NOT NULL,
    lunar_day           INTEGER NOT NULL,
    is_leap             INTEGER NOT NULL DEFAULT 0,
    -- 时辰序号 0~11（子=0 ... 亥=11）
    time_index          INTEGER NOT NULL,
    -- 性别 男/女
    gender              TEXT    NOT NULL DEFAULT '',
    -- 盘面缓存，便于列表预览
    year_stem           TEXT    NOT NULL DEFAULT '',
    year_branch         TEXT    NOT NULL DEFAULT '',
    five_elements_class TEXT    NOT NULL DEFAULT '',
    main_stars          TEXT    NOT NULL DEFAULT '',
    created_at          INTEGER NOT NULL DEFAULT (unixepoch())
);