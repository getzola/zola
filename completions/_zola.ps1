
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'zola' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'zola'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-')) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'zola' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Path to a config file other than config.toml in the root of project')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Path to a config file other than config.toml in the root of project')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Create a new Zola project')
            [CompletionResult]::new('build', 'build', [CompletionResultType]::ParameterValue, 'Deletes the output directory if there is one and builds the site')
            [CompletionResult]::new('serve', 'serve', [CompletionResultType]::ParameterValue, 'Serve the site. Rebuild and reload on change automatically')
            [CompletionResult]::new('check', 'check', [CompletionResultType]::ParameterValue, 'Try building the project without rendering it. Checks links')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Prints this message or the help of the given subcommand(s)')
            break
        }
        'zola;init' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'zola;build' {
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Force the base URL to be that value (default to the one in config.toml)')
            [CompletionResult]::new('--base-url', 'base-url', [CompletionResultType]::ParameterName, 'Force the base URL to be that value (default to the one in config.toml)')
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'Outputs the generated site in the given path')
            [CompletionResult]::new('--output-dir', 'output-dir', [CompletionResultType]::ParameterName, 'Outputs the generated site in the given path')
            [CompletionResult]::new('--drafts', 'drafts', [CompletionResultType]::ParameterName, 'Include drafts when loading the site')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'zola;serve' {
            [CompletionResult]::new('-i', 'i', [CompletionResultType]::ParameterName, 'Interface to bind on')
            [CompletionResult]::new('--interface', 'interface', [CompletionResultType]::ParameterName, 'Interface to bind on')
            [CompletionResult]::new('-p', 'p', [CompletionResultType]::ParameterName, 'Which port to use')
            [CompletionResult]::new('--port', 'port', [CompletionResultType]::ParameterName, 'Which port to use')
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'Outputs the generated site in the given path')
            [CompletionResult]::new('--output-dir', 'output-dir', [CompletionResultType]::ParameterName, 'Outputs the generated site in the given path')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Changes the base_url')
            [CompletionResult]::new('--base-url', 'base-url', [CompletionResultType]::ParameterName, 'Changes the base_url')
            [CompletionResult]::new('--watch-only', 'watch-only', [CompletionResultType]::ParameterName, 'Do not start a server, just re-build project on changes')
            [CompletionResult]::new('--drafts', 'drafts', [CompletionResultType]::ParameterName, 'Include drafts when loading the site')
            [CompletionResult]::new('-O', 'O', [CompletionResultType]::ParameterName, 'Open site in the default browser')
            [CompletionResult]::new('--open', 'open', [CompletionResultType]::ParameterName, 'Open site in the default browser')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'zola;check' {
            [CompletionResult]::new('--drafts', 'drafts', [CompletionResultType]::ParameterName, 'Include drafts when loading the site')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'zola;help' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
