"use client";

import { Alert, AlertDescription } from "@/shared/components/ui/alert";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import { Textarea } from "@/shared/components/ui/textarea";

import { useCreateTicket } from "../application/use-create-ticket";

export function NewTicketForm() {
	const { onSubmit, fieldErrors, failure, pending } = useCreateTicket();

	return (
		<form onSubmit={onSubmit} className="grid max-w-xl gap-5" noValidate>
			<div className="grid gap-2">
				<Label htmlFor="title">What is the problem?</Label>
				<Input id="title" name="title" aria-invalid={!!fieldErrors.title} aria-describedby="title-error" />
				{fieldErrors.title && (
					<p id="title-error" className="text-sm text-destructive">
						{fieldErrors.title}
					</p>
				)}
			</div>

			<div className="grid gap-2">
				<Label htmlFor="body">Tell us more</Label>
				<Textarea id="body" name="body" rows={8} aria-invalid={!!fieldErrors.body} aria-describedby="body-error" />
				{fieldErrors.body && (
					<p id="body-error" className="text-sm text-destructive">
						{fieldErrors.body}
					</p>
				)}
			</div>

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
