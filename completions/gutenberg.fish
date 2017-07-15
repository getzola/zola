function __fish_using_command
    set cmd (commandline -opc)
    if [ (count $cmd) -eq (count $argv) ]
        for i in (seq (count $argv))
            if [ $cmd[$i] != $argv[$i] ]
                return 1
            end
        end
        return 0
    end
    return 1
end

complete -c gutenberg -n "__fish_using_command gutenberg" -s c -l config -d "Path to a config file other than config.toml"
complete -c gutenberg -n "__fish_using_command gutenberg" -s h -l help -d "Prints help information"
complete -c gutenberg -n "__fish_using_command gutenberg" -s V -l version -d "Prints version information"
complete -c gutenberg -n "__fish_using_command gutenberg" -f -a "init" -d "Create a new Gutenberg project"
complete -c gutenberg -n "__fish_using_command gutenberg" -f -a "build" -d "Builds the site"
complete -c gutenberg -n "__fish_using_command gutenberg" -f -a "serve" -d "Serve the site. Rebuild and reload on change automatically"
complete -c gutenberg -n "__fish_using_command gutenberg" -f -a "help" -d "Prints this message or the help of the given subcommand(s)"
complete -c gutenberg -n "__fish_using_command gutenberg init" -s h -l help -d "Prints help information"
complete -c gutenberg -n "__fish_using_command gutenberg init" -s V -l version -d "Prints version information"
complete -c gutenberg -n "__fish_using_command gutenberg build" -s h -l help -d "Prints help information"
complete -c gutenberg -n "__fish_using_command gutenberg build" -s V -l version -d "Prints version information"
complete -c gutenberg -n "__fish_using_command gutenberg serve" -s h -l help -d "Prints help information"
complete -c gutenberg -n "__fish_using_command gutenberg serve" -s V -l version -d "Prints version information"
complete -c gutenberg -n "__fish_using_command gutenberg help" -s h -l help -d "Prints help information"
complete -c gutenberg -n "__fish_using_command gutenberg help" -s V -l version -d "Prints version information"
