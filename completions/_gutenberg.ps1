
@('gutenberg', './gutenberg') | %{
    Register-ArgumentCompleter -Native -CommandName $_ -ScriptBlock {
        param($wordToComplete, $commandAst, $cursorPosition)

        $command = '_gutenberg'
        $commandAst.CommandElements |
            Select-Object -Skip 1 |
            %{
                switch ($_.ToString()) {

                    'Gutenberg' {
                        $command += '_Gutenberg'
                        break
                    }

                    'init' {
                        $command += '_init'
                        break
                    }

                    'build' {
                        $command += '_build'
                        break
                    }

                    'serve' {
                        $command += '_serve'
                        break
                    }

                    'help' {
                        $command += '_help'
                        break
                    }

                    default { 
                        break
                    }
                }
            }

        $completions = @()

        switch ($command) {

            '_gutenberg' {
                $completions = @('init', 'build', 'serve', 'help', '-h', '-V', '-c', '--help', '--version', '--config')
            }

            '_gutenberg_init' {
                $completions = @('-h', '-V', '--help', '--version')
            }

            '_gutenberg_build' {
                $completions = @('-h', '-V', '--help', '--version')
            }

            '_gutenberg_serve' {
                $completions = @('-h', '-V', '--help', '--version')
            }

            '_gutenberg_help' {
                $completions = @('-h', '-V', '--help', '--version')
            }

        }

        $completions |
            ?{ $_ -like "$wordToComplete*" } |
            Sort-Object |
            %{ New-Object System.Management.Automation.CompletionResult $_, $_, 'ParameterValue', $_ }
    }
}
