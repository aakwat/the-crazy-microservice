import "server-only";

import createClient from "openapi-fetch";

import type { paths } from "./contracts/paths";
import { gatewayFetch } from "./gateway-fetch.server";
import { API_URL } from "./safe-fetch.server";

export const apiClient = createClient<paths>({
	baseUrl: API_URL,
	fetch: gatewayFetch,
});
