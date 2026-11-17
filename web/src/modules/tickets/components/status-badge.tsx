import { Badge } from "@/shared/components/ui/badge";

import { statusLabel } from "../domain/status";

export interface StatusBadgeProps {
	status: string;
}

export function StatusBadge(props: StatusBadgeProps) {
	const { status } = props;

	return <Badge variant="secondary">{statusLabel(status)}</Badge>;
}
