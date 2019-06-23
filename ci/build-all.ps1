param (
    [string]
    $Version
)

$Platforms = $("darwin", "freebsd", "linux", "windows")
$Architectures = @("amd64", "386")
$Extensions = @{ "windows" = ".exe" }
   
foreach ($plat in $Platforms) {
    $env:GOOS = "$plat"
    foreach ($arch in $Architectures) {
        $env:GOARCH = "$arch"
       
        Write-Host "Building git-tool-${plat}-${arch}$($Extensions[$plat])@$Version"
        go build -v -x -o "bin/git-tool-${plat}-${arch}$($Extensions[$plat])" -ldflags "-X main.version=$Version" "./cmd/git-tool/main.go"
    }
}

Write-Host "Build Complete"