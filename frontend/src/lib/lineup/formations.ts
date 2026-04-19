import type { Formation } from "$lib/api/types";

export type { Formation };

export interface SlotPosition {
  x: number; // percentage from left (0–100)
  y: number; // percentage from top (0–100); 0 = centre line, 100 = own goal
}

export interface FormationDef {
  name: Formation;
  slots: SlotPosition[]; // 11 entries, index 0 = GK (near own goal)
}

export const FORMATIONS: FormationDef[] = [
  {
    name: "4-4-2",
    slots: [
      { x: 50, y: 88 }, // GK
      { x: 80, y: 70 },
      { x: 60, y: 70 },
      { x: 40, y: 70 },
      { x: 20, y: 70 }, // DEF
      { x: 80, y: 45 },
      { x: 60, y: 45 },
      { x: 40, y: 45 },
      { x: 20, y: 45 }, // MID
      { x: 62, y: 18 },
      { x: 38, y: 18 }, // FWD
    ],
  },
  {
    name: "4-3-3",
    slots: [
      { x: 50, y: 88 },
      { x: 80, y: 70 },
      { x: 60, y: 70 },
      { x: 40, y: 70 },
      { x: 20, y: 70 },
      { x: 67, y: 45 },
      { x: 50, y: 45 },
      { x: 33, y: 45 },
      { x: 80, y: 18 },
      { x: 50, y: 18 },
      { x: 20, y: 18 },
    ],
  },
  {
    name: "4-2-3-1",
    slots: [
      { x: 50, y: 88 },
      { x: 80, y: 72 },
      { x: 60, y: 72 },
      { x: 40, y: 72 },
      { x: 20, y: 72 },
      { x: 62, y: 55 },
      { x: 38, y: 55 },
      { x: 75, y: 35 },
      { x: 50, y: 35 },
      { x: 25, y: 35 },
      { x: 50, y: 15 },
    ],
  },
  {
    name: "3-5-2",
    slots: [
      { x: 50, y: 88 },
      { x: 70, y: 70 },
      { x: 50, y: 70 },
      { x: 30, y: 70 },
      { x: 85, y: 48 },
      { x: 67, y: 48 },
      { x: 50, y: 48 },
      { x: 33, y: 48 },
      { x: 15, y: 48 },
      { x: 63, y: 18 },
      { x: 37, y: 18 },
    ],
  },
  {
    name: "5-3-2",
    slots: [
      { x: 50, y: 88 },
      { x: 85, y: 70 },
      { x: 68, y: 70 },
      { x: 50, y: 70 },
      { x: 32, y: 70 },
      { x: 15, y: 70 },
      { x: 67, y: 45 },
      { x: 50, y: 45 },
      { x: 33, y: 45 },
      { x: 63, y: 18 },
      { x: 37, y: 18 },
    ],
  },
  {
    name: "4-1-4-1",
    slots: [
      { x: 50, y: 88 },
      { x: 80, y: 73 },
      { x: 60, y: 73 },
      { x: 40, y: 73 },
      { x: 20, y: 73 },
      { x: 50, y: 58 },
      { x: 80, y: 38 },
      { x: 60, y: 38 },
      { x: 40, y: 38 },
      { x: 20, y: 38 },
      { x: 50, y: 18 },
    ],
  },
];

export function getFormation(name: Formation): FormationDef {
  return FORMATIONS.find((f) => f.name === name) ?? FORMATIONS[0];
}
