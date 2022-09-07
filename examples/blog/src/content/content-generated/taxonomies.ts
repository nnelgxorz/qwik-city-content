import type { Merge } from "./generated-helpers";
import * as C from "./content";

export const posts: Posts[] = [ C.q4, C.q5, C.q6, C.q7, C.q8, C.q9];
export const subfolder: Subfolder[] = [ C.q5, C.q6];
export const testimonials: Testimonials[] = [ C.q0, C.q1, C.q2, C.q3];

export type Posts = Merge<typeof C.q4 | typeof C.q5 | typeof C.q6 | typeof C.q7 | typeof C.q8 | typeof C.q9>;
export type Subfolder = Merge<typeof C.q5 | typeof C.q6>;
export type Testimonials = Merge<typeof C.q0 | typeof C.q1 | typeof C.q2 | typeof C.q3>;
