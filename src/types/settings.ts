export interface PluginSettings {
  enabled: boolean;
  config: Record<string, unknown>;
}

export interface UprootedSettings {
  enabled: boolean;
  plugins: Record<string, PluginSettings>;
  customCss: string;
}

export const DEFAULT_SETTINGS: UprootedSettings = {
  enabled: true,
  plugins: {},
  customCss: "",
};
