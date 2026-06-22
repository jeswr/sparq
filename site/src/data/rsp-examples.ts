// [OPUS-4.8] sq-11zy — built-in examples for the live /surface/streaming-rsp playground.
// The default is the SKILL.md / sparq-rsp-wasm README walkthrough: AVG(?v) over tumbling
// 60-tick windows, fed bare-numeric Turtle readings, with ts=65 closing the [0,60) window
// at AVG = 15.0. Each example pins a continuous SELECT, a window (range/step/maxDelay),
// the R2S operator, and a scripted reading stream the page replays one push at a time.

import type { R2S } from "@/lib/sparq-rsp-wasm";

/** One scripted reading: a Turtle (s, p, o) triple plus its logical timestamp. */
export interface RspReading {
  s: string;
  p: string;
  o: string;
  ts: number;
}

export interface RspExample {
  id: string;
  label: string;
  description: string;
  /** The continuous SELECT registered over the window. */
  query: string;
  /** RSP-QL window: RANGE / STEP (step==range tumbling, step<range sliding) / lateness. */
  range: number;
  step: number;
  maxDelay: number;
  r2s: R2S;
  /** The scripted reading stream the page replays. */
  readings: RspReading[];
}

const READING = "<http://ex/reading>";
const S1 = "<http://ex/s1>";
const ACTIVE = "<http://ex/active>";

/** Three sensors over six 60-tick windows, so AVG fires a fresh value per window close. */
function avgReadings(): RspReading[] {
  const vals = [
    [10, 0],
    [20, 30],
    [5, 65], // closes [0,60) -> AVG(10,20) = 15.0
    [30, 70],
    [40, 95],
    [50, 130], // closes [60,120) -> AVG(5,30,40) = 25.0
    [60, 190], // closes [120,180) -> AVG(50) = 50.0
  ] as const;
  return vals.map(([v, ts]) => ({ s: S1, p: READING, o: String(v), ts }));
}

export const RSP_EXAMPLES: RspExample[] = [
  {
    id: "avg-tumbling-60",
    label: "AVG per 60-tick window",
    description:
      "AVG(?v) over TUMBLING 60-tick windows (RANGE 60 STEP 60). A push at ts=65 closes [0,60) at AVG = 15.0.",
    query: "SELECT (AVG(?v) AS ?avg) WHERE { ?s <http://ex/reading> ?v }",
    range: 60,
    step: 60,
    maxDelay: 0,
    r2s: "rstream",
    readings: avgReadings(),
  },
  {
    id: "count-tumbling",
    label: "COUNT per window",
    description:
      "COUNT(*) of readings per tumbling 30-tick window — a per-window throughput meter.",
    query: "SELECT (COUNT(*) AS ?n) WHERE { ?s <http://ex/reading> ?v }",
    range: 30,
    step: 30,
    maxDelay: 0,
    r2s: "rstream",
    readings: [
      { s: S1, p: READING, o: "10", ts: 0 },
      { s: S1, p: READING, o: "11", ts: 5 },
      { s: S1, p: READING, o: "12", ts: 20 },
      { s: S1, p: READING, o: "13", ts: 35 }, // closes [0,30): n = 3
      { s: S1, p: READING, o: "14", ts: 50 },
      { s: S1, p: READING, o: "15", ts: 70 }, // closes [30,60): n = 2
    ],
  },
  {
    id: "istream-sliding-active",
    label: "ISTREAM sliding (new rows)",
    description:
      "Sliding window (RANGE 40 STEP 20) with ISTREAM: each close reports only the subjects that NEWLY appeared vs. the previous window.",
    query: "SELECT ?s WHERE { ?s <http://ex/active> ?o }",
    range: 40,
    step: 20,
    maxDelay: 0,
    r2s: "istream",
    readings: [
      { s: "<http://ex/a>", p: ACTIVE, o: "true", ts: 0 },
      { s: "<http://ex/b>", p: ACTIVE, o: "true", ts: 25 }, // close [0,20): a appears
      { s: "<http://ex/c>", p: ACTIVE, o: "true", ts: 45 }, // close [0,40): b appears (a already seen)
      { s: "<http://ex/d>", p: ACTIVE, o: "true", ts: 70 }, // close [20,60): c, d appear
    ],
  },
];
