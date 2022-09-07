import type { Merge } from "./generated-helpers";
import * as C from "./content";

export const qwik: Qwik[] = [ C.q4, C.q5, C.q6, C.q7, C.q8, C.q9];
export const blog: Blog[] = [ C.q4, C.q5, C.q6, C.q7, C.q8, C.q9];
export const partytown: Partytown[] = [ C.q8];
export const qwik_city: QwikCity[] = [ C.q5, C.q6, C.q9];
export const fun: Fun[] = [ C.q4];
export const all = [ C.q0, C.q1, C.q2, C.q3, C.q4, C.q5, C.q6, C.q7, C.q8, C.q9];

export type Qwik = Merge<typeof C.q4 | typeof C.q5 | typeof C.q6 | typeof C.q7 | typeof C.q8 | typeof C.q9>;
export type Blog = Merge<typeof C.q4 | typeof C.q5 | typeof C.q6 | typeof C.q7 | typeof C.q8 | typeof C.q9>;
export type Partytown = Merge<typeof C.q8>;
export type QwikCity = Merge<typeof C.q5 | typeof C.q6 | typeof C.q9>;
export type Fun = Merge<typeof C.q4>;
export type All = Merge<typeof C.q0 | typeof C.q1 | typeof C.q2 | typeof C.q3 | typeof C.q4 | typeof C.q5 | typeof C.q6 | typeof C.q7 | typeof C.q8 | typeof C.q9>