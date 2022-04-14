<template>
  <div v-if="hasReleases">
    <h2>Releases</h2>

    <div class="release-platforms">
      <div>Select your Platform:</div>
      <button
        class="release-button release-platform"
        :class="{ active: platform === selectedPlatform }"
        v-for="(name, platform) in platforms"
        :key="platform"
        v-on:click="selectedPlatform = platform"
      >
        {{ name }}
      </button>
    </div>

    <div class="release-list" v-if="selectedPlatform">
      <div
        class="release"
        v-for="release in applicableReleases"
        :key="release.id"
      >
        <h4 class="release__name">
          <a
            class="release-button"
            :href="
              getReleaseAsset(release, selectedPlatform).browser_download_url
            "
            target="_blank"
            >Download</a
          >

          {{ release.name }}
          <Badge v-if="release.prerelease" text="Early Access" type="warning" />
        </h4>

        <pre class="release__notes">{{ release.body }}</pre>

        <p></p>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed } from "vue";

interface Release {
  id: number;
  published_at: string;
  html_url: string;
  tag_name: string;
  name: string;
  prerelease: boolean;
  body: string;
  assets: ReleaseAsset[];
}

interface ReleaseAsset {
  id: number;
  name: string;
  browser_download_url: string;
  download_count: number;
}

const platforms = {
  "darwin-amd64": "MacOS (Intel)",
  "darwin-arm64": "MacOS (Apple)",
  "linux-amd64": "Linux (x64)",
  "linux-arm64": "Linux (ARM64)",
  "windows-amd64": "Windows (x64)",
};

function getReleaseAsset(
  release: Release,
  platform: string
): ReleaseAsset | undefined {
  return release.assets.find((asset) => asset.name.includes(platform));
}

export default defineComponent({
  props: {
    repo: {
      type: String,
      required: true,
    },
  },

  setup(props) {
    const selectedPlatform = ref(null);
    const releases = ref<Release[]>([]);
    const error = ref(null);
    const hasReleases = computed(() => !!releases.value?.length);
    const applicableReleases = computed(() =>
      (releases.value || [])
        .filter((r) =>
          (r.assets || []).some((a) => a.name.includes(selectedPlatform.value))
        )
        .slice(0, 5)
    );

    fetch(`https://api.github.com/repos/${props.repo}/releases`, {
      headers: {
        Accept: "application/vnd.github.v3+json",
      },
    })
      .then((res) => res.json())
      .then((res: Release[]) => res)
      .then((users) => {
        users.sort((a, b) => b.published_at.localeCompare(a.published_at));
        releases.value = users;
      })
      .catch((err) => {
        error.value = err;
      });

    return {
      releases,
      selectedPlatform,
      applicableReleases,
      hasReleases,
      getReleaseAsset,
      platforms,
      error,
    };
  },
});
</script>

<style scoped>
.release {
  display: flex;
  flex-direction: column;
  align-content: center;
  justify-content: space-between;
  margin: 20px;
}

.release__notes {
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 85%;
}

.release-platforms {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.release-platform.active {
  background: var(--c-brand);
  color: var(--c-bg);
}

.release-assets {
  display: flex;
  flex-direction: row;
  align-content: center;
  justify-content: space-between;
}

.release-asset {
  margin: 10px;
  border-radius: 5px;
  padding: 10px;
  border: 1px solid #ccc;
}

.release-button {
  background: none;
  border-radius: 5px;
  color: var(--c-brand);
  border: 1px solid var(--c-brand);
  font-size: 80%;
  padding: 7px;
  margin: 5px;
  cursor: pointer;
}

.release-button:hover,
.release-button:focus {
  background: var(--c-brand-light);
  color: var(--c-bg);
}
</style>