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
    --logo-file)
        COMPREPLY=($(compgen -f -- "$cur"))
        return
        ;;
    --accent)
        COMPREPLY=($(compgen -W "black red green yellow blue magenta cyan white gray grey orange pink purple tux void arch debian ubuntu fedora opensuse alpine gentoo nixos manjaro mint kali raspbian pop" -- "$cur"))
        return
        ;;
    --color)
        COMPREPLY=($(compgen -W "auto always never" -- "$cur"))
        return
        ;;
    --theme)
        COMPREPLY=($(compgen -W "default light mono neon" -- "$cur"))
        return
        ;;
    --width)
        COMPREPLY=()
        return
        ;;
    esac

    COMPREPLY=($(compgen -W "--help --version --no-logo --logo --logo-file --accent --color --theme --compact --no-color --ascii --width --oneline --list-logos --save-config --show-config --reset-config --only --json" -- "$cur"))
}

complete -F _nullfetch nullfetch
