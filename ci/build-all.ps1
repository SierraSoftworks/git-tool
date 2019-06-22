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
       
        go build -o "bin/git-tool-${plat}-${arch}$($Extensions[$plat])" -ldflags "-X main.version='$Version'" "./cmd/git-tool/main.go"
    }
}