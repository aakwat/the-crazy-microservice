/** One place for the cache keys; everything sits under `all`, so one refresh updates every ticket view. */
export const ticketKeys = {
	all: ["tickets"] as const,
	detail: (id: string) => ["tickets", id] as const,
	messages: (id: string) => ["tickets", id, "messages"] as const,
};
