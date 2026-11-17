import { z } from "zod";

import { frontendCode, slugFromProblemType } from "@/shared/lib/api/error-code";
import { ProblemError } from "@/shared/lib/api/problem";

export interface FieldError {
	field?: string;
	code: string;
	params?: Record<string, unknown>;
}

export type Result<T = void> =
	| { ok: true; data: T }
	| {
			ok: false;
			status: number;
			code: string;
			message: string;
			detail?: string;
			fieldErrors?: FieldError[];
		};

export type NotOkResult = Exclude<Result<unknown>, { ok: true }>;

export function safeAction<TIn, TOut>(
	handler: (input: TIn) => Promise<TOut>,
): (input: TIn) => Promise<Result<TOut>> {
	return async (input: TIn) => {
		try {
			return { ok: true, data: await handler(input) };
		} catch (err) {
			if (err instanceof z.ZodError) {
				return {
					ok: false,
					status: 422,
					code: frontendCode("validation-error"),
					message: "Invalid input",
					detail: err.issues.map((i) => i.message).join("; "),
				};
			}

			if (err instanceof ProblemError) {
				return {
					ok: false,
					status: err.problem.status,
					code: slugFromProblemType(err.problem.type),
					message: err.problem.title,
					detail: err.problem.detail,
					fieldErrors: err.problem.errors,
				};
			}

			throw err;
		}
	};
}
