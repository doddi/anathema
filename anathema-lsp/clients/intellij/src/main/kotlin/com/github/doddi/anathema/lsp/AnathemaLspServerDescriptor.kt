package com.github.doddi.anathema.lsp

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor
import com.intellij.platform.lsp.api.customization.LspDiagnosticsSupport
import org.eclipse.lsp4j.Diagnostic

class AnathemaLspServerDescriptor(project: Project) : ProjectWideLspServerDescriptor(project, "Anathema") {
    override fun createCommandLine(): GeneralCommandLine {
        return GeneralCommandLine().apply {
            withParentEnvironmentType(GeneralCommandLine.ParentEnvironmentType.CONSOLE)
            withCharset(Charsets.UTF_8)
            withExePath("anathema-lsp")
        }
    }

    override fun isSupportedFile(file: VirtualFile) = file.extension == "anat"

    // references resolution is implemented without using the LSP server
    override val lspGoToDefinitionSupport = false

    // code completion is implemented without using the LSP server
    override val lspCompletionSupport = null


    override val lspDiagnosticsSupport: LspDiagnosticsSupport?
        get() = MyLspDiagnosticsSupport()

}

class MyLspDiagnosticsSupport : LspDiagnosticsSupport() {
    override fun getMessage(diagnostic: Diagnostic): String {
        return super.getMessage(diagnostic)
    }
}
