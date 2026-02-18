using System.Text.RegularExpressions;

namespace Uprooted;

/// <summary>
/// Static color manipulation utilities for the custom theme engine.
/// All methods work with "#RRGGBB" hex strings (6-digit, with hash prefix).
/// </summary>
internal static class ColorUtils
{
    private static readonly Regex HexPattern = new(@"^#[0-9A-Fa-f]{6}$");

    public static bool IsValidHex(string? hex)
        => hex != null && HexPattern.IsMatch(hex);

    public static (byte R, byte G, byte B) ParseHex(string hex)
    {
        var h = hex.TrimStart('#');
        if (h.Length == 8) h = h[2..]; // strip alpha prefix
        return (
            Convert.ToByte(h[0..2], 16),
            Convert.ToByte(h[2..4], 16),
            Convert.ToByte(h[4..6], 16)
        );
    }

    public static string ToHex(byte r, byte g, byte b)
        => $"#{r:X2}{g:X2}{b:X2}";

    /// <summary>
    /// Darken a color by a percentage (0-100). Multiplies each channel by (1 - pct/100).
    /// </summary>
    public static string Darken(string hex, double percent)
    {
        var (r, g, b) = ParseHex(hex);
        double factor = 1.0 - Math.Clamp(percent, 0, 100) / 100.0;
        return ToHex(
            (byte)Math.Round(r * factor),
            (byte)Math.Round(g * factor),
            (byte)Math.Round(b * factor)
        );
    }

    /// <summary>
    /// Lighten a color by a percentage (0-100). Blends toward white:
    /// channel + (255 - channel) * pct/100
    /// </summary>
    public static string Lighten(string hex, double percent)
    {
        var (r, g, b) = ParseHex(hex);
        double factor = Math.Clamp(percent, 0, 100) / 100.0;
        return ToHex(
            (byte)Math.Round(r + (255 - r) * factor),
            (byte)Math.Round(g + (255 - g) * factor),
            (byte)Math.Round(b + (255 - b) * factor)
        );
    }

    /// <summary>
    /// Prepend an alpha byte to produce "#AARRGGBB" format (Avalonia convention).
    /// Alpha is 0-255 integer.
    /// </summary>
    public static string WithAlpha(string hex, int alpha)
    {
        var (r, g, b) = ParseHex(hex);
        byte a = (byte)Math.Clamp(alpha, 0, 255);
        return $"#{a:X2}{r:X2}{g:X2}{b:X2}";
    }

    /// <summary>
    /// Relative luminance per WCAG 2.0 formula (0.0 = black, 1.0 = white).
    /// </summary>
    public static double Luminance(string hex)
    {
        var (r, g, b) = ParseHex(hex);
        double rs = Linearize(r / 255.0);
        double gs = Linearize(g / 255.0);
        double bs = Linearize(b / 255.0);
        return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
    }

    private static double Linearize(double c)
        => c <= 0.03928 ? c / 12.92 : Math.Pow((c + 0.055) / 1.055, 2.4);

    /// <summary>
    /// Returns near-white for dark backgrounds, near-black for light backgrounds.
    /// </summary>
    public static string DeriveTextColor(string bgHex)
        => Luminance(bgHex) > 0.3 ? "#1A1A1A" : "#F0F0F0";

    /// <summary>
    /// Returns text color tinted with the accent hue for warmth/cohesion.
    /// Dark bg → near-white with subtle accent hue; light bg → near-black with subtle accent hue.
    /// </summary>
    public static string DeriveTextColorTinted(string bgHex, string accentHex)
    {
        var (ah, asat, _) = RgbToHsl(accentHex);
        bool isDark = Luminance(bgHex) <= 0.3;
        if (isDark)
        {
            double textSat = Math.Max(Math.Min(asat * 0.10, 0.12), 0.03);
            return HslToHex(ah, textSat, 0.94);
        }
        else
        {
            double textSat = Math.Min(asat * 0.08, 0.10);
            return HslToHex(ah, textSat, 0.10);
        }
    }

    /// <summary>
    /// Mix two hex colors by a ratio (0.0 = all colorA, 1.0 = all colorB).
    /// </summary>
    public static string Mix(string hexA, string hexB, double ratio)
    {
        var (ra, ga, ba) = ParseHex(hexA);
        var (rb, gb, bb) = ParseHex(hexB);
        double t = Math.Clamp(ratio, 0, 1);
        return ToHex(
            (byte)Math.Round(ra + (rb - ra) * t),
            (byte)Math.Round(ga + (gb - ga) * t),
            (byte)Math.Round(ba + (bb - ba) * t)
        );
    }

    /// <summary>
    /// Produce a hex string with alpha as a fraction (0.0-1.0) -> "#AARRGGBB".
    /// </summary>
    public static string WithAlphaFraction(string hex, double alpha)
        => WithAlpha(hex, (int)Math.Round(Math.Clamp(alpha, 0, 1) * 255));

    /// <summary>
    /// Convert RGB hex to HSL. H: 0-360, S: 0-1, L: 0-1.
    /// </summary>
    public static (double H, double S, double L) RgbToHsl(string hex)
    {
        var (r, g, b) = ParseHex(hex);
        double rd = r / 255.0, gd = g / 255.0, bd = b / 255.0;
        double max = Math.Max(rd, Math.Max(gd, bd));
        double min = Math.Min(rd, Math.Min(gd, bd));
        double l = (max + min) / 2.0;

        if (max == min) return (0, 0, l);

        double d = max - min;
        double s = l > 0.5 ? d / (2.0 - max - min) : d / (max + min);
        double h;
        if (max == rd)
            h = ((gd - bd) / d + (gd < bd ? 6 : 0)) * 60;
        else if (max == gd)
            h = ((bd - rd) / d + 2) * 60;
        else
            h = ((rd - gd) / d + 4) * 60;

        return (h, s, l);
    }

    /// <summary>
    /// Convert HSL to RGB hex string. H: 0-360, S: 0-1, L: 0-1.
    /// </summary>
    public static string HslToHex(double h, double s, double l)
    {
        h = ((h % 360) + 360) % 360;
        s = Math.Clamp(s, 0, 1);
        l = Math.Clamp(l, 0, 1);

        double c = (1 - Math.Abs(2 * l - 1)) * s;
        double x = c * (1 - Math.Abs((h / 60.0) % 2 - 1));
        double m = l - c / 2;

        double r1, g1, b1;
        if (h < 60)       { r1 = c; g1 = x; b1 = 0; }
        else if (h < 120) { r1 = x; g1 = c; b1 = 0; }
        else if (h < 180) { r1 = 0; g1 = c; b1 = x; }
        else if (h < 240) { r1 = 0; g1 = x; b1 = c; }
        else if (h < 300) { r1 = x; g1 = 0; b1 = c; }
        else               { r1 = c; g1 = 0; b1 = x; }

        return ToHex(
            (byte)Math.Round((r1 + m) * 255),
            (byte)Math.Round((g1 + m) * 255),
            (byte)Math.Round((b1 + m) * 255));
    }

    /// <summary>
    /// Shift hue by degrees and optionally adjust saturation/lightness.
    /// </summary>
    public static string HueShift(string hex, double hueDelta, double satMul = 1.0, double litDelta = 0.0)
    {
        var (h, s, l) = RgbToHsl(hex);
        return HslToHex(h + hueDelta, s * satMul, l + litDelta);
    }

    /// <summary>
    /// Create a color with the same hue as accent but at specific saturation and lightness.
    /// Useful for tinting backgrounds with the accent hue.
    /// </summary>
    public static string TintWithHue(string accentHex, double saturation, double lightness)
    {
        var (h, _, _) = RgbToHsl(accentHex);
        return HslToHex(h, saturation, lightness);
    }

    /// <summary>
    /// Convert RGB hex to HSV. H: 0-360, S: 0-1, V: 0-1.
    /// </summary>
    public static (double H, double S, double V) RgbToHsv(string hex)
    {
        var (r, g, b) = ParseHex(hex);
        double rd = r / 255.0, gd = g / 255.0, bd = b / 255.0;
        double max = Math.Max(rd, Math.Max(gd, bd));
        double min = Math.Min(rd, Math.Min(gd, bd));
        double d = max - min;

        double h = 0;
        if (d > 0)
        {
            if (max == rd)
                h = ((gd - bd) / d + (gd < bd ? 6 : 0)) * 60;
            else if (max == gd)
                h = ((bd - rd) / d + 2) * 60;
            else
                h = ((rd - gd) / d + 4) * 60;
        }

        double s = max > 0 ? d / max : 0;
        return (h, s, max);
    }

    /// <summary>
    /// Convert HSV to RGB hex string. H: 0-360, S: 0-1, V: 0-1.
    /// </summary>
    public static string HsvToHex(double h, double s, double v)
    {
        h = ((h % 360) + 360) % 360;
        s = Math.Clamp(s, 0, 1);
        v = Math.Clamp(v, 0, 1);

        double c = v * s;
        double x = c * (1 - Math.Abs((h / 60.0) % 2 - 1));
        double m = v - c;

        double r1, g1, b1;
        if (h < 60)       { r1 = c; g1 = x; b1 = 0; }
        else if (h < 120) { r1 = x; g1 = c; b1 = 0; }
        else if (h < 180) { r1 = 0; g1 = c; b1 = x; }
        else if (h < 240) { r1 = 0; g1 = x; b1 = c; }
        else if (h < 300) { r1 = x; g1 = 0; b1 = c; }
        else               { r1 = c; g1 = 0; b1 = x; }

        return ToHex(
            (byte)Math.Round((r1 + m) * 255),
            (byte)Math.Round((g1 + m) * 255),
            (byte)Math.Round((b1 + m) * 255));
    }

    /// <summary>
    /// Returns the fully saturated hex color for a given hue (S=1, V=1).
    /// </summary>
    public static string PureHueHex(double hue)
        => HsvToHex(hue, 1.0, 1.0);
}
