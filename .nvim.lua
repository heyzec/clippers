vim.lsp.config["rust-analyzer"] = {
	cmd = { "env", "rust-analyzer" },
	filetypes = { "rust" },
}
vim.lsp.enable("rust-analyzer")
