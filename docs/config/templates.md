# Templates

Git-Tool uses [Go templates](https://golang.org/pkg/text/template/) to allow [applications](apps.md) and [services](services.md) you add to your [config](./) to access information about the repositories and scratchpads they are targeting. This lets you do some pretty powerful stuff, including handling repositories and scratchpads differently if you want to.

## Interpolation

The most basic construct in Go's templating language is the interpolation block, which is replaced with the value of the property it references.

```text
{{ .Target.Name }}
```

**The properties available to you are described in the** [**context**](templates.md#context) **section below.**

This is used extensively when writing [services](services.md) as it allows you to inject information about the repository in question into things like the URLs used by git. You can also use it to inject information into your [applications](apps.md) through any combination of their `command`, `args` and `environment`.

## Context

The context available within the template evaluator depends on whether you are dealing with a scratchpad, a repo whose service is not in your config, or a repo with a matching service.

The `.Target` property will always be available with all of its children, so it is the safest thing to use if you just need some quick information or wish to support scratchpads.

If you are targeting repositories \(either because you're writing a service entry, in which case this is implied\), or because your application only makes sense within the context of a repository, then the `.Service` and `.Repo` properties will be available.

::: tip Go's template language allows you to conditionally use properties, if they exist, with the following construct:

```text
{{ with .Repo }}
    # Note that . now refers to .Repo, so this is the same as .Repo.FullName
    {{ .FullName }}
{{ else }}
    {{ .Target.Name }}
{{ end }}
```

:::

 - . - Target - \*\*Name\*\*: SierraSoftworks/git-tool - \*\*Path\*\*: /home/bpannell/dev/github.com/SierraSoftworks/git-tool - \*\*Exists\*\*: \`true\` - Service - \*\*Domain\*\*: github.com - \*\*Pattern\*\*: \\*/\\* - \*\*DirectoryGlob\*\* \\*/\\* - Repo - Service - \*\*Domain\*\*: github.com - \*\*Pattern\*\*: \\*/\\* - \*\*DirectoryGlob\*\* \\*/\\* - \*\*Domain\*\*: github.com - \*\*FullName\*\*: SierraSoftworks/git-tool - \*\*Name\*\*: git-tool - \*\*Namespace\*\*: SierraSoftworks - \*\*Path\*\*: /home/bpannell/dev/github.com/SierraSoftworks/git-tool - \*\*Exists\*\*: \`true\` - \*\*Valid\*\*: \`true\` - \*\*Website\*\*: https://github.com/SierraSoftworks/git-tool - \*\*GitURL\*\*: git@github.com:SierraSoftworks/git-tool.git - \*\*HttpURL\*\*: https://github.com/SierraSoftworks/git-tool.git

