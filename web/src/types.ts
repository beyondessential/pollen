// Wire types for the pollen API.
//
// The schemas here are *generated* from the Rust handler annotations (utoipa)
// → `web/openapi.json` → `api-types.ts`. Regenerate with `just gen-openapi`
// after changing any handler's request or response. UI-only types and
// constants stay hand-written below the re-exports.

import type { components, paths } from "./api-types";

type Schemas = components["schemas"];

// ── Path-based typing for `callApi` ────────────────────────────────────────
//
// `openapi-typescript` emits a `paths` interface keyed by the OpenAPI path
// strings (e.g. `/api/meta/version`). These helpers project that into the
// `(module, fn)` pair the React hooks use, and pull the post operation's
// request body and 200 response type.

export type ApiPath = keyof paths & string;
type ModuleOf<P extends string> = P extends `/api/${infer M}/${string}`
	? M
	: never;
type FnOf<P extends string> = P extends `/api/${string}/${infer F}`
	? F
	: never;

export type ApiModule = ModuleOf<ApiPath>;
export type ApiFn<M extends ApiModule> = {
	[P in ApiPath]: ModuleOf<P> extends M ? FnOf<P> : never;
}[ApiPath];

export type ApiPathFor<M extends ApiModule, F extends ApiFn<M>> =
	`/api/${M}/${F}` extends ApiPath ? `/api/${M}/${F}` : never;

type PostOp<P extends ApiPath> = paths[P]["post"];

export type ApiResponse<M extends ApiModule, F extends ApiFn<M>> =
	PostOp<ApiPathFor<M, F>> extends {
		responses: { 200: { content: { "application/json": infer R } } };
	}
		? Solidify<R>
		: void;

export type ApiBody<M extends ApiModule, F extends ApiFn<M>> =
	PostOp<ApiPathFor<M, F>> extends {
		requestBody: { content: { "application/json": infer B } };
	}
		? Solidify<B>
		: Record<string, unknown> | undefined;

// utoipa marks `Option<T>` Rust fields as not-required AND nullable, so
// `openapi-typescript` emits them as `field?: T | null`. serde always emits the
// field (as `null` for None), so the optional `?` is wrong at runtime.
// `Solidify` peels that off, making `field: T | null`. Tuples are handled
// separately so `[u64, u64]` doesn't collapse into `(u64 | u64)[]`.
export type Solidify<T> = T extends readonly unknown[]
	? number extends T["length"]
		? Solidify<T[number]>[]
		: { [K in keyof T]: Solidify<Exclude<T[K], undefined>> }
	: T extends object
		? { [K in keyof T]-?: Solidify<Exclude<T[K], undefined>> }
		: T;

// ── Wire types ─────────────────────────────────────────────────────────────

export type VersionInfo = Solidify<Schemas["VersionInfo"]>;

// ── Application + ruleset wire types ────────────────────────────────────────
export type AppView = Solidify<Schemas["AppView"]>;
export type QuestionView = Solidify<Schemas["QuestionView"]>;
export type Opt = Solidify<Schemas["Opt"]>;
export type QuestionKind = Solidify<Schemas["QuestionKind"]>;
export type ApplicationStatus = Solidify<Schemas["ApplicationStatus"]>;
export type Evaluation = Solidify<Schemas["Evaluation"]>;
export type TriggeredConsequence = Solidify<Schemas["TriggeredConsequence"]>;
export type TriggeredGuidance = Solidify<Schemas["TriggeredGuidance"]>;
export type Consequence = Solidify<Schemas["Consequence"]>;
export type Cost = Solidify<Schemas["Cost"]>;
export type MigrationView = Solidify<Schemas["MigrationView"]>;
export type Severity = Solidify<Schemas["Severity"]>;
export type ConsequenceType = Solidify<Schemas["ConsequenceType"]>;
export type Status = Solidify<Schemas["Status"]>;
export type Audience = Solidify<Schemas["Audience"]>;
export type Verdict = Solidify<Schemas["Verdict"]>;

/// An answer value: one option id (single/band) or several (multi).
export type AnswerValue = string | string[];

// UI-only label/colour maps.
export const CONSEQUENCE_TYPE_LABEL: Record<ConsequenceType, string> = {
	Cost: "Cost",
	Operational: "Operational",
	Capability: "Capability loss",
	Support: "Support",
};

export const STATUS_LABEL: Record<Status, string> = {
	Requirement: "Requirement",
	Advisory: "Advisory",
	Referral: "Referral",
};

export const AUDIENCE_LABEL: Record<Audience, string> = {
	Client: "Client IT — required actions",
	Bes: "BES technical — setup",
	Record: "Record & acknowledgments",
};
