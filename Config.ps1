$GitTool = @{
    Directory = $env:HOME;
    Service   = "github.com";
    GitIgnore = @{
        Default = "visualstudiocode";
    };
    Services  = @{
        "github.com"    = @{
            Name           = "github.com";
            NamespaceDepth = 1;
            WebURLFormat   = "https://github.com/{0}/{1}";
            GitURLFormat   = "git@github.com:{0}/{1}.git";
        };
        "bitbucket.org" = @{
            Name           = "bitbucket.org";
            NamespaceDepth = 1;
            WebURLFormat   = "https://bitbucket.org/{0}/{1}";
            GitURLFormat   = "git@bitbucket.org:{0}/{1}.git";
        };
        "gitlab.com"    = @{
            Name           = "gitlab.com";
            NamespaceDepth = 1;
            WebURLFormat   = "https://gitlab.com/{0}/{1}";
            GitURLFormat   = "git@gitlab.com:{0}/{1}.git";
        };
        "dev.azure.com" = @{
            Name           = "dev.azure.com";
            NamespaceDepth = 2;
            WebURLFormat   = "https://dev.azure.com/{0}/_git/{1}";
            GitURLFormat   = "git@ssh.dev.azure.com:v3/{0}/{1}";
        }
    }
}

if ($null -ne $env:DEV_DIRECTORY) {
    $GitTool.Directory = $env:DEV_DIRECTORY
}
