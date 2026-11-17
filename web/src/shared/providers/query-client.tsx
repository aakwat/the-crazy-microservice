"use client";

import {
	environmentManager,
	QueryClient,
	QueryClientProvider as Provider,
} from "@tanstack/react-query";

function makeQueryClient() {
	return new QueryClient({
		defaultOptions: {
			queries: { staleTime: 60_000 },
		},
	});
}

let browserClient: QueryClient | undefined;

function getQueryClient() {
	if (environmentManager.isServer()) return makeQueryClient();
	if (!browserClient) browserClient = makeQueryClient();
	return browserClient;
}

export interface QueryClientProviderProps {
	children: React.ReactNode;
}

export function QueryClientProvider(props: QueryClientProviderProps) {
	const { children } = props;
	const client = getQueryClient();

	return <Provider client={client}>{children}</Provider>;
}
