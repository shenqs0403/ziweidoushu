-- 为 records 表添加分组字段
ALTER TABLE records ADD COLUMN "group" TEXT NOT NULL DEFAULT '';
