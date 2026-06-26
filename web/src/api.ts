import { useCallback, useEffect, useRef, useState } from "react";

import type { ApiBody, ApiFn, ApiModule, ApiResponse } from "./types";

export class ApiError extends Error {
	readonly status: number;
	readonly detail: unknown;

	constructor(status: number, message: string, detail: unknown) {
		super(message);
		this.name = "ApiError";
		this.status = status;
		this.detail = detail;
	}
}

// `M` and `F` are inferred from the positional args and constrain the
// (module, fn) pair against the generated `paths` interface. `T` defaults to
// the response type for that path.
export async function callApi<
	M extends ApiModule,
	F extends ApiFn<M>,
	T = ApiResponse<M, F>,
>(
	module: M,
	fn: F,
	params: ApiBody<M, F> | Record<string, unknown> = {},
	signal?: AbortSignal,
): Promise<T> {
	const response = await fetch(`/api/${module}/${fn}`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify(params),
		signal,
	});

	if (!response.ok) {
		let detail: unknown = null;
		try {
			detail = await response.json();
		} catch {
			detail = await response.text().catch(() => null);
		}
		let extra = "";
		if (
			detail &&
			typeof detail === "object" &&
			"title" in detail &&
			typeof (detail as { title?: unknown }).title === "string"
		) {
			extra = `: ${(detail as { title: string }).title}`;
		}
		throw new ApiError(
			response.status,
			`server fn ${module}.${fn} failed: ${response.status}${extra}`,
			detail,
		);
	}

	return (await response.json()) as T;
}

export type ApiState<T> =
	| { status: "idle" }
	| { status: "loading" }
	| { status: "ok"; data: T }
	| { status: "error"; error: Error };

export function useApi<
	M extends ApiModule,
	F extends ApiFn<M>,
	T = ApiResponse<M, F>,
>(
	module: M,
	fn: F,
	params: Record<string, unknown> = {},
	deps: ReadonlyArray<unknown> = [],
): ApiState<T> & { reload: () => void } {
	const [state, setState] = useState<ApiState<T>>({ status: "idle" });
	const tick = useRef(0);

	const run = useCallback(() => {
		const myTick = ++tick.current;
		const controller = new AbortController();
		// Keep prior data on screen during background refetches.
		setState((prev) => (prev.status === "ok" ? prev : { status: "loading" }));
		callApi<M, F, T>(module, fn, params, controller.signal)
			.then((data) => {
				if (tick.current === myTick) setState({ status: "ok", data });
			})
			.catch((error: unknown) => {
				if (controller.signal.aborted) return;
				if (tick.current !== myTick) return;
				setState({
					status: "error",
					error: error instanceof Error ? error : new Error(String(error)),
				});
			});
		return () => controller.abort();
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, deps);

	useEffect(() => run(), [run]);

	return { ...state, reload: run };
}

/**
 * Hook for write/mutation server fns. Returns a `call` function and
 * `pending` / `error` state for the UI.
 */
export function useApiAction<
	M extends ApiModule,
	F extends ApiFn<M>,
	T = ApiResponse<M, F>,
>(
	module: M,
	fn: F,
): {
	call: (params?: Record<string, unknown>) => Promise<T>;
	pending: boolean;
	error: Error | null;
	reset: () => void;
} {
	const [pending, setPending] = useState(false);
	const [error, setError] = useState<Error | null>(null);

	const call = useCallback(
		async (params: Record<string, unknown> = {}): Promise<T> => {
			setPending(true);
			setError(null);
			try {
				return await callApi<M, F, T>(module, fn, params);
			} catch (err) {
				const e = err instanceof Error ? err : new Error(String(err));
				setError(e);
				throw e;
			} finally {
				setPending(false);
			}
		},
		[module, fn],
	);

	const reset = useCallback(() => setError(null), []);

	return { call, pending, error, reset };
}
