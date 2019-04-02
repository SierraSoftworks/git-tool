function SuggestAutocomplete {
    param(
        [string]
        $commandName, 
        [string]
        $parameterName, 
        [string]
        $wordToComplete,
        $commandAst,
        $fakeBoundParameter
    )

    switch ($parameterName) {
        GitIgnore {
            Get-GitIgnoreTypes | Where-Object { $_ -like "${wordToComplete}*" } | Sort-Object -Unique | ForEach-Object { New-Object System.Management.Automation.CompletionResult( $_, $_, 'ParameterValue', $_ ) }
        }

        Service {
            $GitTool.Services.Keys | Where-Object { $_ -like "${wordToComplete}*" } | Sort-Object -Unique | ForEach-Object { New-Object System.Management.Automation.CompletionResult( $_, $_, 'ParameterValue', $_ ) }
        }

        Repo {
            if (-not $fakeBoundParameter.ContainsKey("Service")) {
                $fakeBoundParameter.Service = $GitTool.Service
            }
        
            if (-not $fakeBoundParameter.ContainsKey("Path")) {
                $fakeBoundParameter.Path = $GitTool.Directory
            }

            $names = Get-Repos -Service $fakeBoundParameter.Service -Path $fakeBoundParameter.Path

            if ($commandName -like "New-Repo") {
                $names = Get-RepoNamespaces -Service $fakeBoundParameter.Service -Path $fakeBoundParameter.Path | ForEach-Object { "$_/" }
            }

            $names | Where-Object { $_ -like "${wordToComplete}*" } | ForEach-Object { New-Object System.Management.Automation.CompletionResult( $_, $_, 'ParameterValue', $_ ) }
        }

        default { return }
    }
}