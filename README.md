# A simple Vi-like TUI text editor on Rust.
Dependencies: ratatui, crossterm latest
## Current features:
- Command/insert mode
- Simple command parses
- Navigating in buffer using keyboard arrows; pgup/pgdn for scrolling
## Default commands:
- !w filename - saves buffer into filename; filename arg is optional if some file already was opened/written
- !r filename - reads file into buffer
- !exec args - executes command from args into sh (in unix-like), and into cmd on windows.
- !exec_f filename - executes shell script filename
- !q - quit editor (unsaved changes will be lost!)
## Planned features:
- Plugin system and manager for Lua plugins
- Multiple tabs
- Editor command aliases
