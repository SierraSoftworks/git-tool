<template>
  <div class="registry-browser">
    <FileTree>
      <ul>
        <li>
          <a href="https://github.com/SierraSoftworks/git-tool/tree/main/registry">registry/ <OutboundLink /></a>
          <ul>
            <li v-for="(items, type) in entries" :key="type">
              {{ type }}/

              <ul>
                <li v-for="item in items" :key="type + '/' + item">
                  {{ item }} &nbsp; <code>gt config add {{type}}/{{item}}</code>
                </li>
              </ul>
            </li>
          </ul>
        </li>
      </ul>
    </FileTree>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import FileTree from "./FileTree.vue";

interface GitHubTree {
  tree: GitHubTreeNode[];
  truncated: boolean;
}

interface GitHubTreeNode {
  type: string;
  path: string;
}

interface RegistryTemplate {
  name: string;
  description: string;
  version: string;
  configs: any[];
}

export default defineComponent({
  components: { FileTree },
  setup() {
    const entries = ref({});
    const error = ref(null);

    fetch(
      "https://api.github.com/repos/SierraSoftworks/git-tool/git/trees/main?recursive=true",
      {
        headers: {
          "Accept": "application/vnd.github.v3+json"
        }
      }
    )
      .then((res) => res.json())
      .then((res: GitHubTree) =>
        res.tree.filter(
          (n) =>
            n.type == "blob" &&
            n.path.startsWith("registry/") &&
            n.path.endsWith(".yaml")
        )
      )
      .then((nodes) =>
        nodes.map((node) => node.path.replace(/^registry\/(.*)\.yaml$/, "$1"))
      )
      .then((nodes) => {
        console.log(nodes);
        return nodes;
      })
      .then(
        (paths) =>
          (entries.value = paths.reduce(
            (types, path) => {
              const id_parts = path.split("/", 2);

              types[id_parts[0]] = types[id_parts[0]] || [];

              types[id_parts[0]].push(id_parts[1]);

              return types;
            },
            {
              apps: [],
              services: [],
            }
          ))
      )
      .catch((err) => (error.value = err));

    return {
      entries,
      error,
    };
  },
});
</script>

<style>
</style>