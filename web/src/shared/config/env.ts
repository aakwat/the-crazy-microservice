import { createEnv } from "@t3-oss/env-nextjs";
import { z } from "zod";

export const env = createEnv({
	server: {
		// The gateway on the private network; it is the only BFF and holds every session.
		API_URL: z.url().default("http://localhost:8080"),
		// Proxies in front of us. Only the X-Forwarded-For entry they appended is trustworthy; 0 sends none.
		TRUSTED_PROXY_HOPS: z.coerce.number().int().min(0).default(0),
	},
	runtimeEnv: {
		API_URL: process.env.API_URL,
		TRUSTED_PROXY_HOPS: process.env.TRUSTED_PROXY_HOPS,
	},
	skipValidation: !!process.env.SKIP_ENV_VALIDATION,
});
