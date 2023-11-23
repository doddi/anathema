# Anathema LSP

This is a simple language server for the Anathema template language.

Once built, ensure that the `anathema-lsp` binary is in your `$PATH`.

## Editors
### Neovim

To enable the lsp for neovim, add the following to your `init.lua`:

```lua
vim.api.nvim_create_autocmd("BufEnter", {
  pattern = "*.anat",
  callback = function()
    vim.lsp.start({
      name = "anathema-lsp",
      cmd = { "anathema-lsp" },
      root_dir = vim.fs.dirname(vim.fs.find({ "Cargo.toml" }, { upward = true })[1]),
    })
  end,
})
```

This will start the lsp when you open a file with the `.anat` extension.


## Features

Currently, the following features are supported:
- Diagnostics that show any compilations errors

![img.png](docs/diagnostics)