// LSP-TYPES: Language Server Protocol type definitions
// Type-safe LSP message structures for JSON-RPC communication
// Zero 'any' types - full type safety

/** LSP Document URI - standardized identifier for documents */
export type DocumentUri = string;

/** LSP Position - zero-based line and character positions */
export interface Position {
  line: number;
  character: number;
}

/** LSP Range - span between start and end positions */
export interface Range {
  start: Position;
  end: Position;
}

/** LSP Location - URI with a specific range */
export interface Location {
  uri: DocumentUri;
  range: Range;
}

/** LSP TextDocumentItem - represents a document at a specific version */
export interface TextDocumentItem {
  uri: DocumentUri;
  languageId: string;
  version: number;
  text: string;
}

/** LSP VersionedTextDocumentIdentifier - document reference with version */
export interface VersionedTextDocumentIdentifier {
  uri: DocumentUri;
  version: number;
}

/** LSP TextDocumentIdentifier - identifies a document by URI */
export interface TextDocumentIdentifier {
  uri: DocumentUri;
}

/** LSP InitializeParams - client capabilities and initialization options */
export interface InitializeParams {
  processId: number | null;
  rootUri: DocumentUri | null;
  initializationOptions?: unknown;
  capabilities: ClientCapabilities;
  trace?: 'off' | 'messages' | 'verbose';
}

/** LSP ClientCapabilities - supported features and behaviors */
export interface ClientCapabilities {
  textDocument?: {
    synchronization?: {
      dynamicRegistration?: boolean;
      willSave?: boolean;
      willSaveWaitUntil?: boolean;
      didSave?: boolean;
    };
    completion?: {
      dynamicRegistration?: boolean;
      completionItem?: {
        snippetSupport?: boolean;
        commitCharactersSupport?: boolean;
        documentationFormat?: string[];
        deprecatedSupport?: boolean;
        preselectSupport?: boolean;
      };
    };
    hover?: {
      dynamicRegistration?: boolean;
      contentFormat?: string[];
    };
    definition?: {
      dynamicRegistration?: boolean;
      linkSupport?: boolean;
    };
    documentSymbol?: {
      dynamicRegistration?: boolean;
      hierarchicalDocumentSymbolSupport?: boolean;
    };
    codeAction?: {
      dynamicRegistration?: boolean;
    };
    formatOnType?: {
      dynamicRegistration?: boolean;
    };
    rename?: {
      dynamicRegistration?: boolean;
    };
    publishDiagnostics?: {
      relatedInformation?: boolean;
      versionSupport?: boolean;
      tagSupport?: { valueSet: number[] };
    };
  };
  workspace?: {
    workspaceEdit?: {
      documentChanges?: boolean;
    };
    didChangeConfiguration?: {
      dynamicRegistration?: boolean;
    };
    didChangeWatchedFiles?: {
      dynamicRegistration?: boolean;
    };
    symbol?: {
      dynamicRegistration?: boolean;
      hierarchicalWorkspaceSymbolSupport?: boolean;
    };
    executeCommand?: {
      dynamicRegistration?: boolean;
    };
    workspaceFolders?: boolean;
    configuration?: boolean;
  };
}

/** LSP InitializeResult - server capabilities after initialization */
export interface InitializeResult {
  capabilities: ServerCapabilities;
  serverInfo?: {
    name: string;
    version?: string;
  };
}

/** LSP ServerCapabilities - features supported by the server */
export interface ServerCapabilities {
  textDocumentSync?: number | {
    openClose?: boolean;
    change?: number;
    willSave?: boolean;
    willSaveWaitUntil?: boolean;
    save?: boolean | { includeText?: boolean };
  };
  completionProvider?: {
    resolveProvider?: boolean;
    triggerCharacters?: string[];
  };
  hoverProvider?: boolean;
  definitionProvider?: boolean | { linkSupport?: boolean };
  documentSymbolProvider?: boolean;
  codeActionProvider?: boolean;
  documentFormattingProvider?: boolean;
  documentRangeFormattingProvider?: boolean;
  documentOnTypeFormattingProvider?: {
    firstTriggerCharacter: string;
    moreTriggerCharacter?: string[];
  };
  renameProvider?: boolean | { prepareProvider?: boolean };
  executeCommandProvider?: {
    commands: string[];
  };
  selectionRangeProvider?: boolean;
}

/** LSP InitializedParams - notification sent after initialize */
export interface InitializedParams {
  [key: string]: unknown;
}

/** LSP DidOpenTextDocumentParams - document open notification */
export interface DidOpenTextDocumentParams {
  textDocument: TextDocumentItem;
}

/** LSP DidChangeTextDocumentParams - document change notification */
export interface DidChangeTextDocumentParams {
  textDocument: VersionedTextDocumentIdentifier;
  contentChanges: TextDocumentContentChangeEvent[];
}

/** LSP TextDocumentContentChangeEvent - incremental or full content change */
export interface TextDocumentContentChangeEvent {
  range?: Range;
  rangeLength?: number;
  text: string;
}

/** LSP DidCloseTextDocumentParams - document close notification */
export interface DidCloseTextDocumentParams {
  textDocument: TextDocumentIdentifier;
}

/** LSP CompletionParams - completion request parameters */
export interface CompletionParams {
  textDocument: TextDocumentIdentifier;
  position: Position;
  context?: CompletionContext;
}

/** LSP CompletionContext - trigger information for completions */
export interface CompletionContext {
  triggerKind: CompletionTriggerKind;
  triggerCharacter?: string;
}

/** LSP CompletionTriggerKind - how completion was triggered */
export enum CompletionTriggerKind {
  Invoked = 1,
  TriggerCharacter = 2,
  TriggerForIncompleteCompletions = 3,
}

/** LSP CompletionList - completion items or incomplete flag */
export interface CompletionList {
  isIncomplete: boolean;
  items: CompletionItem[];
}

/** LSP CompletionItem - single completion suggestion */
export interface CompletionItem {
  label: string;
  kind?: CompletionItemKind;
  detail?: string;
  documentation?: string | { kind: string; value: string };
  deprecated?: boolean;
  preselect?: boolean;
  sortText?: string;
  filterText?: string;
  insertText?: string;
  insertTextFormat?: InsertTextFormat;
  textEdit?: TextEdit;
  additionalTextEdits?: TextEdit[];
  commitCharacters?: string[];
  command?: Command;
  data?: unknown;
}

/** LSP CompletionItemKind - type of completion item */
export enum CompletionItemKind {
  Text = 1, Method = 2, Function = 3, Constructor = 4, Field = 5,
  Variable = 6, Class = 7, Interface = 8, Module = 9, Property = 10,
  Unit = 11, Value = 12, Enum = 13, Keyword = 14, Snippet = 15,
  Color = 16, File = 17, Reference = 18, Folder = 19, EnumMember = 20,
  Constant = 21, Struct = 22, Event = 23, Operator = 24, TypeParameter = 25,
}

/** LSP InsertTextFormat - format of insert text */
export enum InsertTextFormat {
  PlainText = 1,
  Snippet = 2,
}

/** LSP TextEdit - replaces range with new text */
export interface TextEdit {
  range: Range;
  newText: string;
}

/** LSP Command - executable command */
export interface Command {
  title: string;
  command: string;
  arguments?: unknown[];
}

/** LSP PublishDiagnosticsParams - diagnostic notification */
export interface PublishDiagnosticsParams {
  uri: DocumentUri;
  version?: number;
  diagnostics: Diagnostic[];
}

/** LSP Diagnostic - error/warning/info/hint marker */
export interface Diagnostic {
  range: Range;
  severity?: DiagnosticSeverity;
  code?: string | number;
  source?: string;
  message: string;
  relatedInformation?: DiagnosticRelatedInformation[];
}

/** LSP DiagnosticSeverity - severity levels */
export enum DiagnosticSeverity {
  Error = 1,
  Warning = 2,
  Information = 3,
  Hint = 4,
}

/** LSP DiagnosticRelatedInformation - related diagnostic info */
export interface DiagnosticRelatedInformation {
  location: Location;
  message: string;
}

/** LSP HoverParams - hover request parameters */
export interface HoverParams {
  textDocument: TextDocumentIdentifier;
  position: Position;
}

/** LSP Hover - hover information response */
export interface Hover {
  contents: string | { kind: string; value: string } | MarkedString[];
  range?: Range;
}

/** LSP MarkedString - marked string for hover content */
export type MarkedString = string | { language: string; value: string };

/** LSP DefinitionParams - definition request parameters */
export interface DefinitionParams {
  textDocument: TextDocumentIdentifier;
  position: Position;
}

/** LSP Shutdown - empty shutdown request params */
export interface ShutdownParams {
  [key: string]: unknown;
}

/** LSP Exit - empty exit notification params */
export interface ExitParams {
  [key: string]: unknown;
}

/** Notification handler function type */
export type NotificationHandler<T = unknown> = (params: T) => void;

/** Request handler function type */
export type RequestHandler<TParams = unknown, TResult = unknown> = (params: TParams) => TResult | Promise<TResult>;

/** LSP method names as constants */
export const LSPMethod = {
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
} as const;
