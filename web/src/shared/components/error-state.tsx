import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

export interface ErrorStateProps {
	icon: LucideIcon;
	title: string;
	description: string;
	/** `error.digest`: the id support can quote, never the server's message. */
	reference?: string | undefined;
	referenceLabel?: string;
	actions?: ReactNode;
}

// Plain strings, not hooks: `global-error` renders outside the app layout too.
export function ErrorState(props: ErrorStateProps) {
	const { icon: Icon, title, description, reference, referenceLabel, actions } = props;

	return (
		<div role="alert" className="mx-auto grid max-w-md justify-items-center gap-3 px-6 py-20 text-center">
			<span className="flex size-12 items-center justify-center rounded-xl bg-muted">
				<Icon className="size-6 text-muted-foreground" />
			</span>

			<h1 className="text-lg font-semibold">{title}</h1>
			<p className="text-sm text-muted-foreground">{description}</p>

			{reference && (
				<p className="text-xs text-muted-foreground">
					{referenceLabel} <code className="rounded-sm bg-muted px-1 py-0.5 font-mono">{reference}</code>
				</p>
			)}

			{actions && <div className="flex flex-wrap justify-center gap-2 pt-3">{actions}</div>}
		</div>
	);
}
