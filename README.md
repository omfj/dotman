# Dotman

Manage (symlink) your dotfiles with dotman.

## Example

```toml
# API version, doesn't work right now, for the future.
version = "1"

# Always symlink
[[links]]
source = "hosts/common/config/git"
target = "~/.config/git"

# Only symlink if all the conditions are met
[[links]]
source = "hosts/mac/zshrc"
target = "~/.zshrc"
condition = { os = ["macos"], hostname = "omfj" }

# Run some script
[[actions]]
type = "shell-command"
name = "Install Zap for zsh"
command = "zsh <(curl -s https://raw.githubusercontent.com/zap-zsh/zap/master/install.zsh) --branch release-v1"
```
