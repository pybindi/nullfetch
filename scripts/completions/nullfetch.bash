_nullfetch() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    local fields="user host os kernel arch uptime boot_time loadavg cpu cpu_cores gpu ram swap disk disks network local_ip public_ip battery temperature resolution shell terminal de wm packages processes init locale timezone date motherboard bios display audio bluetooth power"
    local themes="default mono light neon dark nord gruvbox dracula tokyo solarized matrix"
    local modes="auto blocks ascii kitty iterm chafa viu jp2 img2txt timg sixel"

    case "$prev" in
    --only | -o)
        COMPREPLY=($(compgen -W "$fields" -- "$cur"))
        return
        ;;
    --logo)
        COMPREPLY=($(compgen -W "auto none alpine arch cactus cat coffee computer cow debian fedora frog gentoo ghost linux mint moon nixos opensuse penguin robot rocket skull tree ubuntu void" -- "$cur"))
        return
        ;;
    --logo-file | --image | --image-file | --config)
        COMPREPLY=($(compgen -f -- "$cur"))
        return
        ;;
    --accent | --image-bg)
        COMPREPLY=($(compgen -W "black red green yellow blue magenta cyan white gray grey orange pink purple tux void arch debian ubuntu fedora opensuse alpine gentoo nixos manjaro mint kali raspbian pop" -- "$cur"))
        return
        ;;
    --color)
        COMPREPLY=($(compgen -W "auto always never" -- "$cur"))
        return
        ;;
    --theme)
        COMPREPLY=($(compgen -W "$themes" -- "$cur"))
        return
        ;;
    --image-mode)
        COMPREPLY=($(compgen -W "$modes" -- "$cur"))
        return
        ;;
    --image-position)
        COMPREPLY=($(compgen -W "side top bottom" -- "$cur"))
        return
        ;;
    --bar-style)
        COMPREPLY=($(compgen -W "blocks ascii arrows dots lines" -- "$cur"))
        return
        ;;
    --format)
        COMPREPLY=($(compgen -W "text json csv markdown oneline" -- "$cur"))
        return
        ;;
    --width | --image-width | --image-height | --bar-width | --cache-ttl)
        COMPREPLY=()
        return
        ;;
    esac

    COMPREPLY=($(compgen -W "--help --version --no-logo --logo --logo-file --image --image-file --image-mode --image-width --image-height --image-position --image-bg --accent --color --theme --compact --no-color --ascii --width --oneline --list-logos --list-fields --list-image-modes --save-config --show-config --reset-config --init-config --set --config --all --separator --bar-width --bar-style --format --no-cache --no-network --timing --only --json" -- "$cur"))
}

complete -F _nullfetch nullfetch
