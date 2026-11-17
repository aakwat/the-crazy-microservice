import type { Metadata } from "next";
import { Geist } from "next/font/google";

import { SiteHeader } from "@/modules/account";
import { cn } from "@/shared/lib/utils";
import { QueryClientProvider } from "@/shared/providers/query-client";

import "./globals.css";

const geist = Geist({ subsets: ["latin"], variable: "--font-sans" });

export const metadata: Metadata = {
	title: "Support",
	description: "Tell us about a problem and follow it until it is solved.",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
	return (
		<html lang="en" className={cn("font-sans", geist.variable)}>
			<body>
				<QueryClientProvider>
					<SiteHeader />
					<main className="mx-auto max-w-4xl px-6 py-8">{children}</main>
				</QueryClientProvider>
			</body>
		</html>
	);
}
