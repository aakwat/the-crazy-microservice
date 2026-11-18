"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { confirmTicketAction, withdrawTicketAction } from "../infrastructure/actions";
import { ticketKeys } from "./query-keys";

type CloseAction = "confirm" | "withdraw";

/** The customer's two ways to close; withdrawing asks once more because it cannot be undone. */
export function useCloseTicket(id: string) {
	const queryClient = useQueryClient();
	const [askingToWithdraw, setAskingToWithdraw] = useState(false);
	const [failure, setFailure] = useState<string>();

	const mutation = useMutation({
		mutationFn: (action: CloseAction) => (action === "confirm" ? confirmTicketAction(id) : withdrawTicketAction(id)),
		onMutate: () => setFailure(undefined),
		onSuccess: async (result) => {
			if (!result.ok) setFailure(result.detail ?? result.message);
			setAskingToWithdraw(false);
			await queryClient.invalidateQueries({ queryKey: ticketKeys.all });
		},
		onError: () => setFailure("That didn't work. Please try again."),
	});

	return {
		confirm: () => mutation.mutate("confirm"),
		askToWithdraw: () => setAskingToWithdraw(true),
		keepTicket: () => setAskingToWithdraw(false),
		withdraw: () => mutation.mutate("withdraw"),
		askingToWithdraw,
		failure,
		pending: mutation.isPending,
	};
}
