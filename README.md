# A simple Vi-like TUI text editor on Rust.
Dependencies: ratatui, crossterm latest
### Note: project will be most likely abandoned. Rust borrowing system makes big codebases unmaintanable.
## Current features:
- Command/insert mode
- Simple command parses
- Navigating in buffer using keyboard arrows, mouse wheel; pgup/pgdn/hm/end.
- Tabs
- Command aliases
## Default commands:
- See commands.md
## Planned features:
- Plugin system and manager for Lua plugins
- Edit history
