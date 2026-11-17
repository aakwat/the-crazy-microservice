import "server-only";

import { cookies } from "next/headers";

import { safeFetch } from "./safe-fetch.server";

/** The gateway's cookies; it turns its session cookie into an access token upstream. */
const GATEWAY_COOKIE_PREFIX = "__Host-Http-gw-";

/** Calls the gateway as the signed-in browser: forwards only the gateway's cookies and never holds a token. */
export async function gatewayFetch(input: string | URL | Request, init?: RequestInit): Promise<Response> {
	const headers = new Headers(input instanceof Request ? input.headers : init?.headers);
	headers.set("cookie", await gatewayCookies());

	return safeFetch(input, { ...init, headers });
}

async function gatewayCookies(): Promise<string> {
	return (await cookies())
		.getAll()
		.filter((cookie) => cookie.name.startsWith(GATEWAY_COOKIE_PREFIX))
		.map((cookie) => `${cookie.name}=${cookie.value}`)
		.join("; ");
}
