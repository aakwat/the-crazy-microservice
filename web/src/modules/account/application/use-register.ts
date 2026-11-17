"use client";

import { type FormEvent, useState } from "react";

import { firstFieldErrors } from "@/shared/lib/form/field-errors";

import { registerAccount } from "../infrastructure/register-account";
import { type RegisterRequest, registerRequestSchema } from "../domain/schemas";

/** Checks the form, creates the account, and reports either success or a message for the form. */
export function useRegister() {
	const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof RegisterRequest, string>>>({});
	const [failure, setFailure] = useState<string>();
	const [pending, setPending] = useState(false);
	const [created, setCreated] = useState(false);

	// onSubmit, not a form action: React resets an action form, which would wipe what the visitor typed.
	async function onSubmit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		const form = new FormData(event.currentTarget);
		const parsed = registerRequestSchema.safeParse({ email: form.get("email"), password: form.get("password") });
		if (!parsed.success) {
			setFieldErrors(firstFieldErrors(parsed.error));
			return;
		}
		setFieldErrors({});
		setFailure(undefined);
		setPending(true);
		const problem = await registerAccount(parsed.data);
		setPending(false);
		if (problem) setFailure(problem);
		else setCreated(true);
	}

	return { onSubmit, fieldErrors, failure, pending, created };
}
