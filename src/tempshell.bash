if [[ -r ~/.bashrc ]]; then
    source ~/.bashrc
fi

_declutter_mark_prompt() {
    local suffix=' \[\e[5m\]TEMPSHELL\[\e[25m\] '
    PS1="${PS1%"$suffix"}$suffix"
}
PROMPT_COMMAND+=(_declutter_mark_prompt)
