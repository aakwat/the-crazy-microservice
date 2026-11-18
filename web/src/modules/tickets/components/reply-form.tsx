"use client";

import { Alert, AlertDescription } from "@/shared/components/ui/alert";
import { Button } from "@/shared/components/ui/button";
import { Label } from "@/shared/components/ui/label";
import { Textarea } from "@/shared/components/ui/textarea";

import { useReply } from "../application/use-reply";
import type { TicketStatus } from "../domain/types";

export interface ReplyFormProps {
	id: string;
	status: TicketStatus;
}

export function ReplyForm(props: ReplyFormProps) {
	const { id, status } = props;
	const { onSubmit, fieldError, failure, pending } = useReply(id);

	return (
		<form onSubmit={onSubmit} className="grid gap-3" noValidate>
			<Label htmlFor="reply">
				{status === "resolved" ? "Still not fixed? Tell us and we will reopen it." : "Reply"}
			</Label>
			<Textarea id="reply" name="body" rows={4} aria-invalid={!!fieldError} aria-describedby="reply-error" />
			{fieldError && (
				<p id="reply-error" className="text-sm text-destructive">
					{fieldError}
				</p>
			)}

			{failure && (
				<Alert variant="destructive">
					<AlertDescription>{failure}</AlertDescription>
				</Alert>
			)}

			<Button type="submit" disabled={pending} className="justify-self-start">
				{pending ? "Sending…" : "Send"}
			</Button>
		</form>
	);
}
