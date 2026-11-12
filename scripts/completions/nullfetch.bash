_nullfetch() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    case "$prev" in
    --only | -o)
        COMPREPLY=($(compgen -W "user host os kernel cpu gpu ram disk" -- "$cur"))
        return
        ;;
    --logo)
        COMPREPLY=($(compgen -W "auto none alpine arch cactus cat coffee computer cow debian fedora frog gentoo ghost linux mint moon nixos opensuse penguin robot rocket skull tree ubuntu void" -- "$cur"))
        return
        ;;
    esac

    COMPREPLY=($(compgen -W "--help --version --no-logo --logo --no-color --only --json" -- "$cur"))
}

complete -F _nullfetch nullfetch
