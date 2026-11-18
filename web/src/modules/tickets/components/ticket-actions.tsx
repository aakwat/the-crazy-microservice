"use client";

import { Alert, AlertDescription } from "@/shared/components/ui/alert";
import { Button } from "@/shared/components/ui/button";

import { useCloseTicket } from "../application/use-close-ticket";
import { canConfirm, canWithdraw } from "../domain/lifecycle";
import type { TicketStatus } from "../domain/types";

export interface TicketActionsProps {
	id: string;
	status: TicketStatus;
}

export function TicketActions(props: TicketActionsProps) {
	const { id, status } = props;
	const { confirm, askToWithdraw, keepTicket, withdraw, askingToWithdraw, failure, pending } = useCloseTicket(id);

	if (!canConfirm(status) && !canWithdraw(status)) return null;

	return (
		<div className="grid gap-3">
			{canConfirm(status) && (
				<div className="flex flex-wrap items-center gap-3">
					<Button onClick={confirm} disabled={pending}>
						Yes, it is solved
					</Button>
					<p className="text-sm text-muted-foreground">This closes the ticket.</p>
				</div>
			)}

			{canWithdraw(status) && !askingToWithdraw && (
				<Button variant="outline" onClick={askToWithdraw} className="justify-self-start">
					I no longer need help
				</Button>
			)}

			{canWithdraw(status) && askingToWithdraw && (
				<div className="flex flex-wrap items-center gap-3">
					<p className="text-sm">Withdraw this ticket? It cannot be reopened.</p>
					<Button variant="destructive" onClick={withdraw} disabled={pending}>
						Withdraw
					</Button>
					<Button variant="ghost" onClick={keepTicket} disabled={pending}>
						Keep it open
					</Button>
				</div>
			)}

			{failure && (
				<Alert variant="destructive">
					<AlertDescription>{failure}</AlertDescription>
				</Alert>
			)}
		</div>
	);
}
