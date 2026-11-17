import { z } from "zod";

import { problemType } from "./error-code";

// RFC 9457 Problem Details for HTTP APIs.
export const problemSchema = z.object({
	type: z.string().default("about:blank"),
	title: z.string(),
	status: z.number().int().min(100).max(599),
	detail: z.string().optional(),
	instance: z.string().optional(),
	request_id: z.string().optional(),
	errors: z
		.array(
			z.object({
				field: z.string().optional(),
				code: z.string(),
				params: z.record(z.string(), z.unknown()).optional(),
			}),
		)
		.optional(),
});

export type Problem = z.infer<typeof problemSchema>;

export function isProblem(value: unknown): value is Problem {
	return problemSchema.safeParse(value).success;
}

export class ProblemError extends Error {
	readonly problem: Problem;

	constructor(problem: Problem) {
		super(problem.title);

		this.name = "ProblemError";
		this.problem = problem;
	}
}

export function upstreamUnavailable(title: string): ProblemError {
	return new ProblemError(problemSchema.parse({ type: problemType("upstream-unavailable"), title, status: 502 }));
}
