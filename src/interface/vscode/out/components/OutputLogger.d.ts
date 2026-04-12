export declare enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3
}
export declare class OutputLogger {
    private static instance;
    private channel;
    private minLevel;
    private logs;
    private constructor();
    static getInstance(): OutputLogger;
    setMinLevel(level: LogLevel): void;
    private formatTime;
    private append;
    debug(component: string, message: string): void;
    info(component: string, message: string): void;
    warn(component: string, message: string): void;
    error(component: string, message: string): void;
    show(): void;
    clear(): void;
    export(): string;
    filter(level: LogLevel): string[];
    dispose(): void;
}
//# sourceMappingURL=OutputLogger.d.ts.map