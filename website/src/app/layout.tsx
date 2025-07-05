import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "NUTS - API Testing, Performance & Security CLI Tool",
  description: "Fast CLI tool for API testing, performance analysis, and security scanning. Built with Rust for modern developers.",
  keywords: ["API testing", "CLI tool", "performance testing", "security scanning", "Rust", "command line"],
  authors: [{ name: "WellCode AI" }],
  openGraph: {
    title: "NUTS - API Testing, Performance & Security CLI Tool",
    description: "Fast CLI tool for API testing, performance analysis, and security scanning.",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
      </head>
      <body className="antialiased">
        {children}
      </body>
    </html>
  );
}
