_fme() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="fme"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        fme)
            opts="-t -a -y -h -V --title --artist --at --album-title --ac --album-cover --year --te --title-exec --ae --artist-exec --help --version [FILES]..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --title)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --artist)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --album-title)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --at)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --album-cover)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ac)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --year)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -y)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --title-exec)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --te)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --artist-exec)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ae)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _fme -o nosort -o bashdefault -o default fme
