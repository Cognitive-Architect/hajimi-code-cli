export declare class ToolPicker {
    private static instance;
    private recentlyUsed;
    private allTools;
    private outputChannel;
    private static readonly MAX_RECENT;
    private static readonly TOOL_PREFIX;
    private constructor();
    static getInstance(): ToolPicker;
    private fuzzySearch;
    private categoryFilter;
    private toItem;
    private getItems;
    private addRecent;
    show(): Promise<string | undefined>;
    showByCategory(category: string): Promise<string | undefined>;
}
//# sourceMappingURL=ToolPicker.d.ts.map