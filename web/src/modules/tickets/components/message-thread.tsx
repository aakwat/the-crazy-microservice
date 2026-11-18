"use client";

import { Alert, AlertDescription } from "@/shared/components/ui/alert";
import { formatExact } from "@/shared/lib/format";

import { useMessages } from "../application/use-messages";
import { senderLabel } from "../domain/sender";

export interface MessageThreadProps {
	id: string;
}

export function MessageThread(props: MessageThreadProps) {
	const { id } = props;
	const messages = useMessages(id);

	if (messages.isPending) return <p className="text-sm text-muted-foreground">Loading the conversation…</p>;

	if (messages.isError) {
		return (
			<Alert variant="destructive">
				<AlertDescription>We couldn’t load the conversation. Please refresh the page.</AlertDescription>
			</Alert>
		);
	}

	if (messages.data.length === 0) {
		return <p className="text-sm text-muted-foreground">No replies yet. We will answer here.</p>;
	}

	return (
		<ol className="grid gap-3">
			{messages.data.map((message) => (
				<li
					key={message.id}
					className={`rounded-xl border p-4 ${message.from === "support" ? "bg-muted/50" : "bg-background"}`}
				>
					<p className="mb-1 text-sm text-muted-foreground">
						<span className="font-medium text-foreground">{senderLabel(message.from)}</span>
						{" · "}
						{formatExact(message.created_at, "en")}
					</p>
					<p className="whitespace-pre-wrap">{message.body}</p>
				</li>
			))}
		</ol>
	);
}
