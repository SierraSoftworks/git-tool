import { defineUserConfig, PageHeader, DefaultThemeOptions } from 'vuepress-vite'

function htmlDecode(input: string): string {
  return input.replace("&#39;", "'").replace("&amp;", "&").replace("&quot;", '"')
}

function fixPageHeader(header: PageHeader) {
  header.title = htmlDecode(header.title)
  header.children.forEach(child => fixPageHeader(child))
}

export default defineUserConfig<DefaultThemeOptions>({
  lang: 'en-GB',
  title: 'Git-Tool',
  description: 'Keep your repos organized without having to try.',

  head: [
    ['meta', { name: "description", content: "Documentation for Git-Tool, a powerful command-line helper which keeps your Git repositories organized automatically." }],
    ['link', { rel: 'icon', href: '/favicon.ico' }],
  ],

  bundler: "@vuepress/bundler-vite",

  extendsPageData(page, app) {
    const fixedHeaders = page.headers || []
    fixedHeaders.forEach(header => fixPageHeader(header))

    return {
      headers: fixedHeaders,
    }
  },

  themeConfig: {
    logo: 'https://cdn.sierrasoftworks.com/logos/icon.png',

    docsRepo: "SierraSoftworks/git-tool-docs",
    repo: "SierraSoftworks/git-tool",
    navbar: [
      {
        text: "Getting Started",
        link: "/guide/",
      },
      {
        text: "Commands",
        link: "/commands/",
        children: [
          '/commands/README.md',
          '/commands/repos.md',
          '/commands/scratch.md',
          '/commands/dev.md',
          '/commands/config.md',
          '/commands/setup.md',
        ]
      },
      {
        text: "Configuration",
        link: "/config/",
        children: [
          '/config/README.md',
          '/config/apps.md',
          '/config/services.md',
          '/config/features.md',
          '/config/registry.md',
          '/config/templates.md'
        ]
      },
      {
        text: "Download",
        link: "https://github.com/SierraSoftworks/git-tool/releases",
        target: "_blank"
      },
      {
        text: "Report an Issue",
        link: "https://github.com/SierraSoftworks/git-tool/issues/new/choose",
        target: "_blank"
      }
    ],

    sidebar: {
      '/guide/': [
        {
          isGroup: true,
          text: "Getting Started",
          children: [
            '/guide/README.md',
            '/guide/installation.md',
            '/guide/usage.md',
            '/guide/github.md',
            '/guide/updates.md',
          ]
        }
      ],
      '/commands/': [
        {
          isGroup: true,
          text: "Commands",
          children: [
            '/commands/README.md',
            '/commands/repos.md',
            '/commands/scratch.md',
            '/commands/dev.md',
            '/commands/config.md',
            '/commands/setup.md',
          ]
        }
      ],
      '/config/': [
        {
          isGroup: true,
          text: "Configuration",
          children: [
            '/config/README.md',
            '/config/apps.md',
            '/config/services.md',
            '/config/features.md',
            '/config/registry.md',
            '/config/templates.md'
          ]
        }
      ]
    }
  },

  plugins: [
    ["@vuepress/plugin-google-analytics", { id: "G-WJQ1PVYVH0" }]
  ]


})