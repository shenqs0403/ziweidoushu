import {invoke} from "@tauri-apps/api/core";
import type {Chart, FlowAnnual, RecordInfo} from "./defined";

export interface AddRecordInput {
    name: string;
    calendarType: string;
    solarYear: number;
    solarMonth: number;
    solarDay: number;
    timeIndex: number;
    gender: string;
}

export const api = {
    listRecords: (): Promise<RecordInfo[]> => invoke("list_records"),
    addRecord: (input: AddRecordInput): Promise<number> => invoke("add_record", {input}),
    deleteRecord: (id: number): Promise<void> => invoke("delete_record", {id}),
    chartForRecord: (id: number): Promise<[RecordInfo, Chart]> => invoke("chart_for_record", {id}),
    flowAnnual: (year: number): Promise<FlowAnnual> => invoke("flow_annual", {year}),
};