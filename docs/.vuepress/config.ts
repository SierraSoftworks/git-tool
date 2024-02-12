import { defineUserConfig, PageHeader } from 'vuepress'
import { viteBundler } from '@vuepress/bundler-vite'
import { defaultTheme } from '@vuepress/theme-default'
import { path } from '@vuepress/utils'

import { googleAnalyticsPlugin } from '@vuepress/plugin-google-analytics'
import { registerComponentsPlugin } from '@vuepress/plugin-register-components'

function htmlDecode(input: string): string {
  return input.replace("&#39;", "'").replace("&amp;", "&").replace("&quot;", '"')
}

function fixPageHeader(header: PageHeader) {
  header.title = htmlDecode(header.title)
  header.children.forEach(child => fixPageHeader(child))
}

export default defineUserConfig({
  lang: 'en-GB',
  title: 'Git-Tool',
  description: 'Keep your repos organized without having to try.',

  head: [
    ['meta', { name: "description", content: "Documentation for Git-Tool, a powerful command-line helper which keeps your Git repositories organized automatically." }],
    ['link', { rel: 'icon', href: '/favicon.ico' }]
  ],

  bundler: viteBundler(),

  extendsPage(page, app) {
    const fixedHeaders = page.headers || []
    fixedHeaders.forEach(header => fixPageHeader(header))
  },

  theme: defaultTheme({
    logo: 'https://cdn.sierrasoftworks.com/logos/icon.png',

    repo: "SierraSoftworks/git-tool",
    docsRepo: "SierraSoftworks/git-tool",
    docsDir: 'docs',
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
          text: "Getting Started",
          children: [
            '/guide/README.md',
            '/guide/installation.md',
            '/guide/usage.md',
            '/guide/github.md',
            '/guide/updates.md',
            '/guide/migrating-v3.md',
            '/guide/reporting-errors.md',
            '/guide/faq.md'
          ]
        }
      ],
      '/commands/': [
        {
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
  }),

  plugins: [
    googleAnalyticsPlugin({ id: "G-BF54T4NKMN" }),
    registerComponentsPlugin({
      componentsDir: path.resolve(__dirname, './components'),
    })
  ]
})
