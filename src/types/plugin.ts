export interface Author {
  name: string;
  id?: string;
}

export interface Patch {
  bridge: "nativeToWebRtc" | "webRtcToNative";
  method: string;
  before?(args: unknown[]): boolean | void | Promise<boolean | void>;
  after?(result: unknown, args: unknown[]): void | Promise<void>;
  replace?(...args: unknown[]): unknown | Promise<unknown>;
}

export interface SettingsDefinition {
  [key: string]: SettingField;
}

export type SettingField =
  | { type: "boolean"; default: boolean; description: string }
  | { type: "string"; default: string; description: string }
  | { type: "number"; default: number; description: string; min?: number; max?: number }
  | { type: "select"; default: string; description: string; options: string[] };

export interface UprootedPlugin {
  name: string;
  description: string;
  version: string;
  authors: Author[];
  start?(): void | Promise<void>;
  stop?(): void | Promise<void>;
  patches?: Patch[];
  css?: string;
  settings?: SettingsDefinition;
}
