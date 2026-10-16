// The part of qz-tray.js the developer demo uses. The package ships no type definitions.
declare module 'qz-tray' {
  type Resolver<T> = (resolve: (value: T) => void, reject: (reason?: unknown) => void) => void;

  export interface Qz {
    api: {
      setPromiseType(type: <T>(resolver: Resolver<T>) => Promise<T>): void;
    };
    security: {
      setCertificatePromise(handler: Resolver<string>): void;
    };
    websocket: {
      connect(options?: {
        host?: string | string[];
        retries?: number;
        delay?: number;
      }): Promise<void>;
      disconnect(): Promise<void>;
      isActive(): boolean;
      setClosedCallbacks(callback: () => void): void;
    };
    printers: {
      find(): Promise<string[]>;
      getDefault(): Promise<string | null>;
    };
    configs: {
      create(printer: string, options?: { jobName?: string }): unknown;
    };
    print(
      config: unknown,
      data: { type: 'raw'; format: 'command'; flavor: 'base64'; data: string }[],
    ): Promise<void>;
  }

  const qz: Qz;
  export default qz;
}
