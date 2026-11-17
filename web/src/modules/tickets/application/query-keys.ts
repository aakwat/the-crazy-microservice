/** One place for the cache keys, so a new ticket refreshes every list that shows tickets. */
export const ticketKeys = {
	all: ["tickets"] as const,
	detail: (id: string) => ["tickets", id] as const,
};
