import { upstreamUnavailable } from "./problem";

/** A 2xx with no body is the upstream's fault, never the user's; failures already threw in safeFetch. */
export function requireData<T>(data: T | undefined, what: string): T {
	if (data === undefined) throw upstreamUnavailable(`${what} returned no data`);

	return data;
}
