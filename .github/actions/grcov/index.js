const os = require('os')
const core = require('@actions/core')
const github = require('@actions/github')
const https = require('https')
const fs = require('fs')
const tarfs = require('tar-fs')
const unbzip2 = require('unbzip2-stream')
const bzip2 = require('unbzip2-stream/lib/bzip2')

const grcovRepo = {
    owner: "mozilla",
    repo: "grcov"
}

const platformAssets = {
    'linux': "grcov-linux-x86_64.tar.bz2",
    'darwin': 'grcov-osx-x86_64.tar.bz2'
}

try {
    const version = core.getInput("version", { required: false }) || "latest"
    const githubToken = core.getInput("github-token", { required: true })

    const assetName = platformAssets[os.platform()]
    if (!assetName) {
        throw new Error(`mozilla/grcov does not support ${os.platform()}`)
    }

    const octokit = github.getOctokit(githubToken)

    core.debug("Fetching releases for mozilla/grcov")
    const releases = await octokit.repos.listReleases(grcovRepo)

    core.debug("Found the following mozilla/grcov releases")
    releases.data.forEach(r => core.debug(`  - ${r.tag_name}`))

    const release = releases.data.find(r => version === 'latest' || r.tag_name === version)

    if (!release) {
        throw new Error(`Could not find a release matching '${version}'.`)
    }

    core.debug(`Fetching assets for mozilla/grcov@${release.tag_name}`)
    const assets = await octokit.repos.listReleaseAssets({
        ...grcovRepo,
        release_id: release.id
    })

    const asset = assets.data.find(a => a.name === assetName)
    if (!asset) {
        throw new Error(`mozilla/grcov@${release.tag_name} has not published the ${assetName} asset`)
    }

    core.info(`Installing mozilla/grcov@${release.tag_name}`)

    https.get(asset.browser_download_url).pipe(unbzip2()).pipe(tarfs.extract('.installed/grcov')).end(() => {
        core.addPath(".installed/grcov")
        core.info(`Installed mozilla/grcov@${release.tag_name}`)
    })

} catch (err) {
    core.setFailed(err.message)
}