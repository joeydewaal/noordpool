import type { Formation } from "$lib/api/types";

export type { Formation };

export interface SlotPosition {
  x: number; // percentage from left (0–100)
  y: number; // percentage from top (0–100); 0 = centre line, 100 = own goal
  label: string; // position abbreviation, e.g. "GK", "CB", "ST"
}

export interface FormationDef {
  name: Formation;
  slots: SlotPosition[]; // 11 entries, index 0 = GK (near own goal)
}

export const FORMATIONS: FormationDef[] = [
  {
    name: "4-4-2",
    slots: [
      { x: 50, y: 88, label: "GK" },
      { x: 80, y: 70, label: "RB" },
      { x: 60, y: 70, label: "CB" },
      { x: 40, y: 70, label: "CB" },
      { x: 20, y: 70, label: "LB" },
      { x: 80, y: 45, label: "RM" },
      { x: 60, y: 45, label: "CM" },
      { x: 40, y: 45, label: "CM" },
      { x: 20, y: 45, label: "LM" },
      { x: 62, y: 18, label: "ST" },
      { x: 38, y: 18, label: "ST" },
    ],
  },
  {
    name: "4-3-3",
    slots: [
      { x: 50, y: 88, label: "GK" },
      { x: 80, y: 70, label: "RB" },
      { x: 60, y: 70, label: "CB" },
      { x: 40, y: 70, label: "CB" },
      { x: 20, y: 70, label: "LB" },
      { x: 67, y: 45, label: "CM" },
      { x: 50, y: 45, label: "CM" },
      { x: 33, y: 45, label: "CM" },
      { x: 80, y: 18, label: "RW" },
      { x: 50, y: 18, label: "ST" },
      { x: 20, y: 18, label: "LW" },
    ],
  },
  {
    name: "4-2-3-1",
    slots: [
      { x: 50, y: 88, label: "GK" },
      { x: 80, y: 72, label: "RB" },
      { x: 60, y: 72, label: "CB" },
      { x: 40, y: 72, label: "CB" },
      { x: 20, y: 72, label: "LB" },
      { x: 62, y: 55, label: "CDM" },
      { x: 38, y: 55, label: "CDM" },
      { x: 75, y: 35, label: "RAM" },
      { x: 50, y: 35, label: "CAM" },
      { x: 25, y: 35, label: "LAM" },
      { x: 50, y: 15, label: "ST" },
    ],
  },
  {
    name: "3-5-2",
    slots: [
      { x: 50, y: 88, label: "GK" },
      { x: 70, y: 70, label: "CB" },
      { x: 50, y: 70, label: "CB" },
      { x: 30, y: 70, label: "CB" },
      { x: 85, y: 48, label: "RWB" },
      { x: 67, y: 48, label: "CM" },
      { x: 50, y: 48, label: "CM" },
      { x: 33, y: 48, label: "CM" },
      { x: 15, y: 48, label: "LWB" },
      { x: 63, y: 18, label: "ST" },
      { x: 37, y: 18, label: "ST" },
    ],
  },
  {
    name: "5-3-2",
    slots: [
      { x: 50, y: 88, label: "GK" },
      { x: 85, y: 70, label: "RWB" },
      { x: 68, y: 70, label: "RCB" },
      { x: 50, y: 70, label: "CB" },
      { x: 32, y: 70, label: "LCB" },
      { x: 15, y: 70, label: "LWB" },
      { x: 67, y: 45, label: "CM" },
      { x: 50, y: 45, label: "CM" },
      { x: 33, y: 45, label: "CM" },
      { x: 63, y: 18, label: "ST" },
      { x: 37, y: 18, label: "ST" },
    ],
  },
  {
    name: "4-1-4-1",
    slots: [
      { x: 50, y: 88, label: "GK" },
      { x: 80, y: 73, label: "RB" },
      { x: 60, y: 73, label: "CB" },
      { x: 40, y: 73, label: "CB" },
      { x: 20, y: 73, label: "LB" },
      { x: 50, y: 58, label: "CDM" },
      { x: 80, y: 38, label: "RM" },
      { x: 60, y: 38, label: "CM" },
      { x: 40, y: 38, label: "CM" },
      { x: 20, y: 38, label: "LM" },
      { x: 50, y: 18, label: "ST" },
    ],
  },
];

export function getFormation(name: Formation): FormationDef {
  return FORMATIONS.find((f) => f.name === name) ?? FORMATIONS[0];
}
