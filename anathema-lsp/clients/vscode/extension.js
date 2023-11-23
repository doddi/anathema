// @ts-check
const { LanguageClient } = require("vscode-languageclient/node");
const tmpdir = require("os").tmpdir();

module.exports = {
  /** @param {import("vscode").ExtensionContext} context*/
  activate(context) {
    /** @type {import("vscode-languageclient/node").ServerOptions} */
    const serverOptions = {
      run: {
        command: "anathema-lsp",
      },
      debug: {
        command: "anathema-lsp",
        // args: ["--file", `${tmpdir}/lsp.log`, "--level", "TRACE"],
      },
    };

    /** @type {import("vscode-languageclient/node").LanguageClientOptions} */
    const clientOptions = {
      documentSelector: [{ scheme: "file", pattern: "**/*.anat" }],
    };

    const client = new LanguageClient(
      "anathema-lsp",
      "Anathema Language Server",
      serverOptions,
      clientOptions
    );

    client.start();
  },
};
