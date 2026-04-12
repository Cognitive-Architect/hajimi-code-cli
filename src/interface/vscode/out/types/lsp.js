"use strict";
// LSP-TYPES: Language Server Protocol type definitions
// Type-safe LSP message structures for JSON-RPC communication
// Zero 'any' types - full type safety
Object.defineProperty(exports, "__esModule", { value: true });
exports.LSPMethod = exports.DiagnosticSeverity = exports.InsertTextFormat = exports.CompletionItemKind = exports.CompletionTriggerKind = void 0;
/** LSP CompletionTriggerKind - how completion was triggered */
var CompletionTriggerKind;
(function (CompletionTriggerKind) {
    CompletionTriggerKind[CompletionTriggerKind["Invoked"] = 1] = "Invoked";
    CompletionTriggerKind[CompletionTriggerKind["TriggerCharacter"] = 2] = "TriggerCharacter";
    CompletionTriggerKind[CompletionTriggerKind["TriggerForIncompleteCompletions"] = 3] = "TriggerForIncompleteCompletions";
})(CompletionTriggerKind || (exports.CompletionTriggerKind = CompletionTriggerKind = {}));
/** LSP CompletionItemKind - type of completion item */
var CompletionItemKind;
(function (CompletionItemKind) {
    CompletionItemKind[CompletionItemKind["Text"] = 1] = "Text";
    CompletionItemKind[CompletionItemKind["Method"] = 2] = "Method";
    CompletionItemKind[CompletionItemKind["Function"] = 3] = "Function";
    CompletionItemKind[CompletionItemKind["Constructor"] = 4] = "Constructor";
    CompletionItemKind[CompletionItemKind["Field"] = 5] = "Field";
    CompletionItemKind[CompletionItemKind["Variable"] = 6] = "Variable";
    CompletionItemKind[CompletionItemKind["Class"] = 7] = "Class";
    CompletionItemKind[CompletionItemKind["Interface"] = 8] = "Interface";
    CompletionItemKind[CompletionItemKind["Module"] = 9] = "Module";
    CompletionItemKind[CompletionItemKind["Property"] = 10] = "Property";
    CompletionItemKind[CompletionItemKind["Unit"] = 11] = "Unit";
    CompletionItemKind[CompletionItemKind["Value"] = 12] = "Value";
    CompletionItemKind[CompletionItemKind["Enum"] = 13] = "Enum";
    CompletionItemKind[CompletionItemKind["Keyword"] = 14] = "Keyword";
    CompletionItemKind[CompletionItemKind["Snippet"] = 15] = "Snippet";
    CompletionItemKind[CompletionItemKind["Color"] = 16] = "Color";
    CompletionItemKind[CompletionItemKind["File"] = 17] = "File";
    CompletionItemKind[CompletionItemKind["Reference"] = 18] = "Reference";
    CompletionItemKind[CompletionItemKind["Folder"] = 19] = "Folder";
    CompletionItemKind[CompletionItemKind["EnumMember"] = 20] = "EnumMember";
    CompletionItemKind[CompletionItemKind["Constant"] = 21] = "Constant";
    CompletionItemKind[CompletionItemKind["Struct"] = 22] = "Struct";
    CompletionItemKind[CompletionItemKind["Event"] = 23] = "Event";
    CompletionItemKind[CompletionItemKind["Operator"] = 24] = "Operator";
    CompletionItemKind[CompletionItemKind["TypeParameter"] = 25] = "TypeParameter";
})(CompletionItemKind || (exports.CompletionItemKind = CompletionItemKind = {}));
/** LSP InsertTextFormat - format of insert text */
var InsertTextFormat;
(function (InsertTextFormat) {
    InsertTextFormat[InsertTextFormat["PlainText"] = 1] = "PlainText";
    InsertTextFormat[InsertTextFormat["Snippet"] = 2] = "Snippet";
})(InsertTextFormat || (exports.InsertTextFormat = InsertTextFormat = {}));
/** LSP DiagnosticSeverity - severity levels */
var DiagnosticSeverity;
(function (DiagnosticSeverity) {
    DiagnosticSeverity[DiagnosticSeverity["Error"] = 1] = "Error";
    DiagnosticSeverity[DiagnosticSeverity["Warning"] = 2] = "Warning";
    DiagnosticSeverity[DiagnosticSeverity["Information"] = 3] = "Information";
    DiagnosticSeverity[DiagnosticSeverity["Hint"] = 4] = "Hint";
})(DiagnosticSeverity || (exports.DiagnosticSeverity = DiagnosticSeverity = {}));
/** LSP method names as constants */
exports.LSPMethod = {
    Initialize: 'initialize',
    Initialized: 'initialized',
    Shutdown: 'shutdown',
    Exit: 'exit',
    TextDocumentDidOpen: 'textDocument/didOpen',
    TextDocumentDidChange: 'textDocument/didChange',
    TextDocumentDidClose: 'textDocument/didClose',
    TextDocumentCompletion: 'textDocument/completion',
    TextDocumentHover: 'textDocument/hover',
    TextDocumentDefinition: 'textDocument/definition',
    TextDocumentPublishDiagnostics: 'textDocument/publishDiagnostics',
};
//# sourceMappingURL=lsp.js.map