import type { Metadata } from "next";
import "./globals.css";
import { siteConfig } from "@/lib/site-config";

export const metadata: Metadata = {
  metadataBase: new URL(siteConfig.siteUrl),
  title: {
    default: siteConfig.title,
    template: `%s | ${siteConfig.name}`,
  },
  description: siteConfig.description,
  alternates: {
    canonical: "/",
    languages: {
      "zh-CN": "/",
      "x-default": "/",
    },
  },
  openGraph: {
    title: siteConfig.title,
    description: siteConfig.description,
    type: "website",
    url: siteConfig.siteUrl,
    siteName: siteConfig.name,
    locale: "zh_CN",
    images: [{
      url: "/og-image.png",
      width: 1200,
      height: 630,
      alt: siteConfig.name,
    }],
  },
  twitter: {
    card: "summary_large_image",
    images: ["/og-image.png"],
  },
  appleWebApp: {
    capable: true,
    title: siteConfig.title,
    statusBarStyle: "default",
  },
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#ffffff" },
    { media: "(prefers-color-scheme: dark)", color: "#0a0a0a" },
  ],
  icons: {
    apple: "/apple-touch-icon.png",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="zh-CN" className="h-full antialiased">
      <body className="min-h-full flex flex-col font-sans">
        <script type="application/ld+json" dangerouslySetInnerHTML={{
          __html: JSON.stringify({
            "@context": "https://schema.org",
            "@type": "SoftwareApplication",
            "name": "Pulseclaw",
            "alternateName": "Pulseclaw - 先保留上下文，再让 AI 开口",
            "description": "Desktop-first, local-first product for replayable, evidence-first context capture and bounded AI help.",
            "url": "https://cozmio.net",
            "applicationCategory": "DeveloperApplication",
            "operatingSystem": "Windows",
            "offers": {
              "@type": "Offer",
              "price": "0",
              "priceCurrency": "USD",
              "availability": "https://schema.org/SoftwareApplicationStatus"
            },
            "author": {
              "@type": "Person",
              "name": "Cozmio",
              "url": "https://cozmio.net"
            }
          })
        }} />
        {children}
      </body>
    </html>
  );
}
