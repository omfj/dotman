# Dotman

Manage (symlink) your dotfiles with dotman.

## Example

```toml
version = "1"

[[links]]
source = "hosts/common/config/git"
target = "~/.config/git"

[[links]]
source = "hosts/common/config/ghostty"
target = "~/.config/ghostty"

[[links]]
source = "hosts/common/config/helix"
target = "~/.config/helix"

[[links]]
source = "hosts/common/config/nvim"
target = "~/.config/nvim"

[[links]]
source = "hosts/common/config/starship.toml"
target = "~/.config/starship.toml"

[[links]]
source = "hosts/mac/zshrc"
target = "~/.zshrc"
condition = { os = ["macos"] }
```
