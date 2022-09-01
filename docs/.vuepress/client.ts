import { defineClientConfig } from '@vuepress/client'

declare global {
    const __VUEPRESS_DEV__: boolean
    const __VUEPRESS_SSR__: boolean

    interface Window {
        cfbeacon: boolean
    }
}

export default defineClientConfig({
    enhance({ app }) {
        if (__VUEPRESS_DEV__ || __VUEPRESS_SSR__) return

        if (window.cfbeacon) {
            return
        }

        const token = "e2a0df0a8bdc4902939764910f86dcd9"
        const cfScript = window.document.createElement("script")
        cfScript.src = "https://static.cloudflareinsights.com/beacon.min.js"
        cfScript.defer = true
        cfScript.setAttribute("data-cf-beacon", JSON.stringify({ token }))

        window.document.head.appendChild(cfScript)
        window.cfbeacon = true
    },
})