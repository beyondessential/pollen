import { createContext, useContext } from "react";

import { callApi } from "./api";
import type { ApiBody, ApiFn, ApiResponse, PricingModelView } from "./types";

/// Which edition of the tool is showing: the public one at the root, or the
/// internal pricing edition under its unguessable path (spec COST, Access).
/// Everything that builds a link to a plan reads `base`, so moving between
/// plans never drops out of the edition the reader is in.
export type Edition = {
	/// The path prefix plans live under: empty for the public tool.
	base: string;
	/// The pricing path, when this is the internal edition.
	pricingPath: string | null;
	/// The pricing configuration, loaded under that path.
	model: PricingModelView | null;
};

const EditionContext = createContext<Edition>({ base: "", pricingPath: null, model: null });

export const EditionProvider = EditionContext.Provider;

export function useEdition(): Edition {
	return useContext(EditionContext);
}

export function planUrl(edition: Edition, id: string): string {
	return `${edition.base}/a/${id}`;
}

/// Call a pricing endpoint, naming the pricing path it is gated on.
export function callPricing<F extends ApiFn<"pricing">>(
	pricingPath: string,
	fn: F,
	params: ApiBody<"pricing", F>,
	signal?: AbortSignal,
): Promise<ApiResponse<"pricing", F>> {
	return callApi("pricing", fn, params, signal, { "x-pricing-path": pricingPath });
}

/// A figure as displayed: whole units, in the configured currency.
export function money(value: number, currency = "USD"): string {
	return new Intl.NumberFormat("en-AU", {
		style: "currency",
		currency,
		currencyDisplay: "narrowSymbol",
		maximumFractionDigits: 0,
	}).format(value);
}
