import starlight from "@astrojs/starlight";
import rehypeRelativeMarkdownLinks from "astro-rehype-relative-markdown-links";
import { defineConfig } from "astro/config";
import starlightLinksValidator from "starlight-links-validator";

const rawBase = process.env.DOCS_BASE ?? "/";
const base = rawBase === "/" ? "/" : rawBase.replace(/\/+$/, "");
const prefix = base === "/" ? "" : base;

export default defineConfig({
  site: "https://immich-edit.iridiumlab.dev",
  base,
  trailingSlash: "always",
  redirects: {
    "/getting-started/": `${prefix}/start/install/`,
    "/use/": `${prefix}/editing/workspace/`,
    "/cull/": `${prefix}/editing/browse-and-cull/`,
    "/edit/": `${prefix}/editing/develop/`,
    "/export/": `${prefix}/editing/export/`,
    "/operate/": `${prefix}/hosting/deployment/`,
    "/deploy/": `${prefix}/hosting/deployment/`,
    "/gpu-passthrough/": `${prefix}/hosting/gpu/`,
    "/rendering/": `${prefix}/hosting/rendering/`,
    "/oauth/": `${prefix}/hosting/oauth/`,
    "/configuration/": `${prefix}/hosting/configuration/`,
    "/administration/": `${prefix}/hosting/administration/`,
    "/backup-and-upgrade/": `${prefix}/hosting/backup-and-upgrade/`,
    "/troubleshooting/": `${prefix}/hosting/troubleshooting/`,
    "/features/": `${prefix}/reference/features/`,
    "/compatibility/": `${prefix}/reference/formats/`,
    "/shortcuts/": `${prefix}/reference/shortcuts/`,
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
          items: [{ slug: "start/install" }],
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
