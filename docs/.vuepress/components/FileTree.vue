<template>
  <div class="filetree">
      <slot></slot>
  </div>
</template>

<style>
.filetree {
    font-family: "Fira Code", monospace;
    font-size: 1em;
    line-height: 1.1;
    -webkit-text-size-adjust: none;
}

.filetree ul {
    margin-left: 0;
    margin-right: 0;
    padding-left: 0;
    list-style: none;
}

.filetree li {
    margin: 0;
    padding: 0;
}

/* level 1 */
.filetree li::before { content: ""; }

/* level 2 */
.filetree li li::before { content: "├──\00a0"; }
.filetree li li:last-child::before { content: "└──\00a0"; }

/* level 3 */
.filetree li li li::before { content: "│\00a0\00a0\00a0├──\00a0"}
.filetree li li li:last-child::before { content: "│\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li::before { content: "\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li:last-child::before { content: "\00a0\00a0\00a0\00a0└──\00a0"; }

/* level 4 */
.filetree li li li li::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li li li:last-child::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li li:last-child li::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li li:last-child li:last-child::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li li::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li li:last-child::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li:last-child li::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li:last-child li:last-child::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }

/* level 5 */
.filetree li li li li li::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li li li li:last-child::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li li li:last-child li::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li li li:last-child li:last-child::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li li:last-child li li::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li li:last-child li li:last-child::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li li:last-child li:last-child li::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li li:last-child li:last-child li:last-child::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li li li::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li li li:last-child::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li li:last-child li::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li li:last-child li:last-child::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li:last-child li li::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li:last-child li li:last-child::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li:last-child li:last-child li::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li:last-child li:last-child li:last-child::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }

/* level 6 */
.filetree li li li li li li::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li li li li li:last-child::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li li li li:last-child li::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li li li li:last-child li:last-child::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li li li:last-child li li::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li li li:last-child li li:last-child::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li li li:last-child li:last-child li::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li li li:last-child li:last-child li:last-child::before { content: "│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li li:last-child li li li::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li li:last-child li li li:last-child::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li li:last-child li li:last-child li::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li li:last-child li li:last-child li:last-child::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li li:last-child li:last-child li li::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li li:last-child li:last-child li li:last-child::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li li:last-child li:last-child li:last-child li::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li li:last-child li:last-child li:last-child li:last-child::before { content: "│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li li li li::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li li li li:last-child::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li li li:last-child li::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li li li:last-child li:last-child::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li li:last-child li li::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li li:last-child li li:last-child::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li li:last-child li:last-child li::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li li:last-child li:last-child li:last-child::before { content: "\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li:last-child li li li::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li:last-child li li li:last-child::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li:last-child li li:last-child li::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li:last-child li li:last-child li:last-child::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li:last-child li:last-child li li::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li:last-child li:last-child li li:last-child::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0│\00a0\00a0\00a0└──\00a0"; }
.filetree li li:last-child li:last-child li:last-child li:last-child li::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0├──\00a0"}
.filetree li li:last-child li:last-child li:last-child li:last-child li:last-child::before { content: "\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0\00a0└──\00a0"; }
</style>