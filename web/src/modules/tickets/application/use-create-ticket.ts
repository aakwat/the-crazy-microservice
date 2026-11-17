"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { type FormEvent, useState } from "react";

import { firstFieldErrors } from "@/shared/lib/form/field-errors";

import { createTicketAction } from "../infrastructure/actions";
import { ticketKeys } from "./query-keys";
import { createTicketRequestSchema } from "../domain/schemas";
import type { CreateTicketRequest } from "../domain/types";

/** Checks the form, sends the ticket, then opens it; failures come back as messages for the form. */
export function useCreateTicket() {
	const router = useRouter();
	const queryClient = useQueryClient();
	const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof CreateTicketRequest, string>>>({});
	const [failure, setFailure] = useState<string>();

	const mutation = useMutation({
		mutationFn: createTicketAction,
		onSuccess: async (result) => {
			if (!result.ok) {
				setFailure(result.detail ?? result.message);
				return;
			}
			await queryClient.invalidateQueries({ queryKey: ticketKeys.all });
			router.push(`/tickets/${result.data.id}`);
		},
		onError: () => setFailure("We couldn't send your ticket. Please try again."),
	});

	// onSubmit, not a form action: React resets an action form, which would wipe what the customer typed.
	function onSubmit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		const form = new FormData(event.currentTarget);
		const parsed = createTicketRequestSchema.safeParse({ title: form.get("title"), body: form.get("body") });
		if (!parsed.success) {
			setFieldErrors(firstFieldErrors(parsed.error));
			return;
		}
		setFieldErrors({});
		setFailure(undefined);
		mutation.mutate(parsed.data);
	}

	return { onSubmit, fieldErrors, failure, pending: mutation.isPending };
}
