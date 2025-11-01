# Tracktui
A TUI to track progress of any kind.

# Setup
Requires **Rust** compiler.
```bash
git clone https://github.com/novrion/tracktui
cd tracktui
bash compile.sh
```

### Autorun
Add the following to **.bashrc** to automatically open Tracktui the first time you open the terminal each day. The idea is to give a score to the previous day.
```bash
cd <path/to/tracktui> # i.e. $HOME/git/tracktui
bash startup.sh
cd $HOME
```

