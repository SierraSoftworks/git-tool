<#
    .SYNOPSIS
        Gets the configured dev directory which holds all of your repositories.

    .EXAMPLE
        Get-DevDirectory
        C:\dev\
#>
function Get-DevDirectory {
    $GitTool.Directory
}