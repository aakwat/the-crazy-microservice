"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useState } from "react";

import { firstFieldErrors } from "@/shared/lib/form/field-errors";

import { replyRequestSchema } from "../domain/schemas";
import { replyAction } from "../infrastructure/actions";
import { ticketKeys } from "./query-keys";

/** Checks and sends a reply; the box clears only once it is sent, so a failure never loses the text. */
export function useReply(id: string) {
	const queryClient = useQueryClient();
	const [fieldError, setFieldError] = useState<string>();
	const [failure, setFailure] = useState<string>();

	const mutation = useMutation({
		mutationFn: replyAction,
		onError: () => setFailure("We couldn't send your message. Please try again."),
	});

	function onSubmit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		const form = event.currentTarget;

		const parsed = replyRequestSchema.safeParse({ body: new FormData(form).get("body") });
		if (!parsed.success) {
			setFieldError(firstFieldErrors(parsed.error).body);
			return;
		}

		setFieldError(undefined);
		setFailure(undefined);
		mutation.mutate(
			{ id, ...parsed.data },
			{
				onSuccess: async (result) => {
					if (result.ok) form.reset();
					else setFailure(result.detail ?? result.message);
					// Also after a failure: the ticket may have changed meanwhile, e.g. closed.
					await queryClient.invalidateQueries({ queryKey: ticketKeys.all });
				},
			},
		);
	}

	return { onSubmit, fieldError, failure, pending: mutation.isPending };
}
