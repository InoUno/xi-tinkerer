/* eslint-disable */
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

declare global {
    interface Window {
        __TAURI_INVOKE__<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
    }
}

// Function avoids 'window not defined' in SSR
const invoke = () => window.__TAURI_INVOKE__;

export function dummyEventTypeGen() {
    return invoke()<[FileNotification, DatProcessorMessage]>("dummy_event_type_gen")
}

export function selectFfxiFolder(path: string | null) {
    return invoke()<string | null>("select_ffxi_folder", { path })
}

export function selectProjectFolder(path: string | null) {
    return invoke()<string[]>("select_project_folder", { path })
}

export function loadPersistenceData() {
    return invoke()<PersistenceData>("load_persistence_data")
}

export function getStandaloneStringDats() {
    return invoke()<DatDescriptor[]>("get_standalone_string_dats")
}

export function getItemDats() {
    return invoke()<DatDescriptor[]>("get_item_dats")
}

export function getZonesForType(datDescriptor: DatDescriptor) {
    return invoke()<ZoneInfo[]>("get_zones_for_type", { datDescriptor })
}

export function getWorkingFiles() {
    return invoke()<DatDescriptor[]>("get_working_files")
}

export function makeAllDats() {
    return invoke()<null>("make_all_dats")
}

export function makeDat(datDescriptor: DatDescriptor) {
    return invoke()<null>("make_dat", { datDescriptor })
}

export function makeAllYamls() {
    return invoke()<null>("make_all_yamls")
}

export function makeYaml(datDescriptor: DatDescriptor) {
    return invoke()<null>("make_yaml", { datDescriptor })
}

export type DatProcessorOutputKind = "Dat" | "Yaml"
export type DatDescriptor = { type: "AbilityNames" } | { type: "AbilityDescriptions" } | { type: "AreaNames" } | { type: "AreaNamesAlt" } | { type: "CharacterSelect" } | { type: "ChatFilterTypes" } | { type: "DayNames" } | { type: "Directions" } | { type: "EquipmentLocations" } | { type: "ErrorMessages" } | { type: "IngameMessages1" } | { type: "IngameMessages2" } | { type: "JobNames" } | { type: "KeyItems" } | { type: "MenuItemsDescription" } | { type: "MenuItemsText" } | { type: "MoonPhases" } | { type: "PolMessages" } | { type: "RaceNames" } | { type: "RegionNames" } | { type: "SpellNames" } | { type: "SpellDescriptions" } | { type: "StatusInfo" } | { type: "StatusNames" } | { type: "TimeAndPronouns" } | { type: "Titles" } | { type: "Misc1" } | { type: "Misc2" } | { type: "WeatherTypes" } | { type: "Armor" } | { type: "Armor2" } | { type: "Currency" } | { type: "GeneralItems" } | { type: "GeneralItems2" } | { type: "PuppetItems" } | { type: "UsableItems" } | { type: "Weapons" } | { type: "VouchersAndSlips" } | { type: "Monipulator" } | { type: "Instincts" } | { type: "EntityNames"; index: number } | { type: "Dialog"; index: number } | { type: "Dialog2"; index: number }
export type FileNotification = { dat_descriptor: DatDescriptor; is_delete: boolean }
export type DatProcessingState = "Working" | "Finished" | { Error: string }
export type PersistenceData = { ffxi_path: string | null; recent_projects: string[] }
export type DatProcessorMessage = { dat_descriptor: DatDescriptor; output_kind: DatProcessorOutputKind; state: DatProcessingState }
export type ZoneInfo = { id: number; name: string }
