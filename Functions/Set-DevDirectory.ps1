<#
    .SYNOPSIS
        Configures the dev directory used by Git-Tool and returned by the Get-DevDirectory command.

    .DESCRIPTION
        Sets the directory used to host your development repositories for your current session.
        
    .PARAMETER Path
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.

    .EXAMPLE
        Set-DevDirectory C:\dev\
#>
function Set-DevDirectory {
    param (
        [string]
        [Parameter(Mandatory = $true, ValueFromPipeline = $true, HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/).")]
        $Path
    )

    $GitTool.Directory = $Path
}