import starlight from "@astrojs/starlight";
import rehypeRelativeMarkdownLinks from "astro-rehype-relative-markdown-links";
import { defineConfig } from "astro/config";
import starlightLinksValidator from "starlight-links-validator";

const rawBase = process.env.DOCS_BASE ?? "/immich-edit";
const base = rawBase === "/" ? "/" : rawBase.replace(/\/+$/, "");

export default defineConfig({
  site: "https://haavardnk.github.io",
  base,
  trailingSlash: "always",
  redirects: {
    "/getting-started/": `${base}/start/install/`,
    "/use/": `${base}/editing/workspace/`,
    "/cull/": `${base}/editing/browse-and-cull/`,
    "/edit/": `${base}/editing/develop/`,
    "/export/": `${base}/editing/export/`,
    "/operate/": `${base}/hosting/deployment/`,
    "/deploy/": `${base}/hosting/deployment/`,
    "/gpu-passthrough/": `${base}/hosting/gpu/`,
    "/rendering/": `${base}/hosting/rendering/`,
    "/oauth/": `${base}/hosting/oauth/`,
    "/configuration/": `${base}/hosting/configuration/`,
    "/administration/": `${base}/hosting/administration/`,
    "/backup-and-upgrade/": `${base}/hosting/backup-and-upgrade/`,
    "/troubleshooting/": `${base}/hosting/troubleshooting/`,
    "/features/": `${base}/reference/features/`,
    "/compatibility/": `${base}/reference/formats/`,
    "/shortcuts/": `${base}/reference/shortcuts/`,
  },
  markdown: {
    rehypePlugins: [
      [
        rehypeRelativeMarkdownLinks,
        { base, collectionBase: false, trailingSlash: "always" },
      ],
    ],
  },
  integrations: [
    starlight({
      title: "immich-edit",
      description: "Self-hosted, non-destructive RAW photo editor for Immich.",
      favicon: "/favicon.svg",
      logo: {
        dark: "./src/assets/logo-dark.svg",
        light: "./src/assets/logo-light.svg",
        alt: "immich-edit",
      },
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/haavardnk/immich-edit",
        },
      ],
      editLink: {
        baseUrl: "https://github.com/haavardnk/immich-edit/edit/main/docs/",
      },
      customCss: ["./src/styles/custom.css"],
      components: {
        LanguageSelect: "./src/components/VersionSelect.astro",
        Banner: "./src/components/VersionBanner.astro",
      },
      plugins: [starlightLinksValidator({ errorOnRelativeLinks: false })],
      sidebar: [
        {
          label: "Start",
          items: [{ slug: "start/install" }, { slug: "start/first-edit" }],
        },
        {
          label: "Editing",
          items: [
            { slug: "editing/workspace" },
            { slug: "editing/browse-and-cull" },
            { slug: "editing/map" },
            { slug: "editing/develop" },
            { slug: "editing/masks-and-retouch" },
            { slug: "editing/presets-and-history" },
            { slug: "editing/export" },
          ],
        },
        {
          label: "Self-hosting",
          items: [
            { slug: "hosting/deployment" },
            { slug: "hosting/gpu" },
            { slug: "hosting/rendering" },
            { slug: "hosting/oauth" },
            { slug: "hosting/configuration" },
            { slug: "hosting/administration" },
            { slug: "hosting/backup-and-upgrade" },
            { slug: "hosting/troubleshooting" },
          ],
        },
        {
          label: "Reference",
          items: [
            { slug: "reference/features" },
            { slug: "reference/formats" },
            { slug: "reference/shortcuts" },
          ],
        },
      ],
    }),
  ],
});
