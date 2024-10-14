import { useRouter } from "next/router";
import { useConfig } from "nextra-theme-docs";
import { Footer } from "./components/Footer";
import TurboLogo from "./components/logos/Turbo";

const theme = {
  project: {
    link: "https://github.com/khulnasoft/titanrepo",
  },
  docsRepositoryBase: "https://github.com/khulnasoft/titanrepo/blob/main/docs",
  titleSuffix: " | Titanrepo",
  unstable_flexsearch: true,
  unstable_staticImage: true,
  toc: {
    float: true,
  },
  font: false,
  chat: {
    link: "https://titan.khulnasoft.com/discord",
  },
  feedback: {
    link: "Question? Give us feedback →",
  },
  banner: function Banner() {
    return (
      <a
        href="https://khulnasoft.com/blog/khulnasoft-acquires-titanrepo?utm_source=titan-site&amp;utm_medium=banner&amp;utm_campaign=titan-website"
        target="_blank"
        rel="noopener noreferrer"
        className="font-medium text-current no-underline"
        title="Go to the Khulnasoft website"
      >
        Titanrepo has joined Khulnasoft. Read More →
      </a>
    );
  },
  logo: function LogoActual() {
    return (
      <>
        <TurboLogo height={32} />
        <span className="sr-only">Titanrepo</span>
      </>
    );
  },
  head: function () {
    const router = useRouter();
    const { frontMatter, title } = useConfig();
    const fullUrl =
      router.asPath === "/"
        ? "https://titan.khulnasoft.com"
        : `https://titan.khulnasoft.com${router.asPath}`;
    return (
      <>
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link
          rel="apple-touch-icon"
          sizes="180x180"
          href="/images/favicon/apple-touch-icon.png"
        />
        <link
          rel="icon"
          type="image/png"
          sizes="32x32"
          href="/images/favicon/favicon-32x32.png"
        />
        <link
          rel="icon"
          type="image/png"
          sizes="16x16"
          href="/images/favicon/favicon-16x16.png"
        />
        <link
          rel="mask-icon"
          href="/images/favicon/safari-pinned-tab.svg"
          color="#000000"
        />
        <link rel="shortcut icon" href="/images/favicon/favicon.ico" />
        <meta name="msapplication-TileColor" content="#000000" />
        <meta name="theme-color" content="#000" />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:site" content="@titanrepo" />
        <meta name="twitter:creator" content="@titanrepo" />
        <meta property="og:type" content="website" />
        <meta name="og:title" content={title} />
        <meta name="og:description" content={frontMatter.description} />
        <meta property="og:url" content={fullUrl} />
        <link rel="canonical" href={fullUrl} />
        <meta
          property="twitter:image"
          content={`https://titan.khulnasoft.com${
            frontMatter.ogImage ?? "/og-image.png"
          }`}
        />
        <meta
          property="og:image"
          content={`https://titan.khulnasoft.com${
            frontMatter.ogImage ?? "/og-image.png"
          }`}
        />
        <meta property="og:locale" content="en_IE" />
        <meta property="og:site_name" content="Titanrepo" />
      </>
    );
  },
  editLink: {
    text: "Edit this page on GitHub",
  },
  footer: {
    text: () => {
      return <Footer />;
    },
  },
  nextThemes: {
    defaultTheme: "dark",
  },
};
export default theme;
