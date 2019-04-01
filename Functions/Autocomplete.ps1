function SuggestRepoName {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameter)

    if (-not $fakeBoundParameter.ContainsKey("Service")) {
        $fakeBoundParameter.Service = $GitTool.Service
    }

    if (-not $fakeBoundParameter.ContainsKey("Path")) {
        $fakeBoundParameter.Path = $GitTool.Directory
    }

    Get-Repos -Service $fakeBoundParameter.Service -Path $fakeBoundParameter.Path | Where-Object { $_ -like "${wordToComplete}*" } | ForEach-Object { New-Object System.Management.Automation.CompletionResult( $_, $_, 'ParameterValue', $_ ) }
}

function SuggestRepoPrefix {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameter)

    if (-not $fakeBoundParameter.ContainsKey("Service")) {
        $fakeBoundParameter.Service = $GitTool.Service
    }

    if (-not $fakeBoundParameter.ContainsKey("Path")) {
        $fakeBoundParameter.Path = $GitTool.Directory
    }

    Get-RepoNamespaces -Service $fakeBoundParameter.Service -Path $fakeBoundParameter.Path | ForEach-Object { "$_/" } | Where-Object { $_ -like "${wordToComplete}*" } | ForEach-Object { New-Object System.Management.Automation.CompletionResult( $_, $_, 'ParameterValue', $_ ) }
}

function SuggestGitIgnoreType {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameter)

    Get-GitIgnoreTypes | Where-Object { $_ -like "${wordToComplete}*" } | Sort-Object -Unique | ForEach-Object { New-Object System.Management.Automation.CompletionResult( $_, $_, 'ParameterValue', $_ ) }
}