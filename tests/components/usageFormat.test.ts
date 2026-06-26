import { describe, expect, it } from "vitest";
import {
  formatTokensShort,
  getLocaleFromLanguage,
} from "@/components/usage/format";

describe("usage format helpers", () => {
  it("formats token counts with English K/M/B units", () => {
    expect(formatTokensShort(12_345)).toBe("12.3K");
    expect(formatTokensShort(123_456_789, "en", 2)).toBe("123.46M");
    expect(formatTokensShort(1_500_000_000)).toBe("1.50B");
  });

  it("always resolves English locale", () => {
    expect(getLocaleFromLanguage()).toBe("en-US");
    expect(getLocaleFromLanguage("zh_TW")).toBe("en-US");
    expect(getLocaleFromLanguage("ja-JP")).toBe("en-US");
  });
});
