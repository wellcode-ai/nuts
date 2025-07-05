import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "NUTS - Network Universal Testing Suite",
  description: "A cyberpunk CLI tool for API testing, performance analysis, and security scanning. Built for the modern developer.",
  keywords: ["API testing", "CLI tool", "performance testing", "security scanning", "Rust", "cyberpunk"],
  authors: [{ name: "WellCode AI" }],
  openGraph: {
    title: "NUTS - Network Universal Testing Suite",
    description: "A cyberpunk CLI tool for API testing, performance analysis, and security scanning.",
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
      <body className="antialiased">
        {children}
      </body>
    </html>
  );
}
