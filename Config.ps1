$GitTool = @{
    Directory = $env:HOME;
    Service   = "github.com";
    GitIgnore = @{
        Default = "visualstudiocode";
    };
}

if ($null -ne $env:DEV_DIRECTORY) {
    $GitTool.Directory = $env:DEV_DIRECTORY
}