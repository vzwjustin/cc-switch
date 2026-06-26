import React, { Suspense } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { http, HttpResponse } from "msw";
import { SettingsPage } from "@/components/settings/SettingsPage";
import {
  resetProviderState,
  getSettings,
  getAppConfigDirOverride,
} from "../msw/state";
import { server } from "../msw/server";

const toastSuccessMock = vi.fn();
const toastErrorMock = vi.fn();

vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccessMock(...args),
    error: (...args: unknown[]) => toastErrorMock(...args),
  },
}));

vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ open, children }: any) =>
    open ? <div data-testid="dialog-root">{children}</div> : null,
  DialogContent: ({ children }: any) => <div>{children}</div>,
  DialogHeader: ({ children }: any) => <div>{children}</div>,
  DialogFooter: ({ children }: any) => <div>{children}</div>,
  DialogTitle: ({ children }: any) => <h2>{children}</h2>,
  DialogDescription: ({ children }: any) => <div>{children}</div>,
}));

const TabsContext = React.createContext<{
  value: string;
  onValueChange?: (value: string) => void;
}>({
  value: "general",
});

vi.mock("@/components/ui/tabs", () => {
  return {
    Tabs: ({ value, onValueChange, children }: any) => (
      <TabsContext.Provider value={{ value, onValueChange }}>
        {children}
      </TabsContext.Provider>
    ),
    TabsList: ({ children }: any) => <div>{children}</div>,
    TabsTrigger: ({ value, children }: any) => {
      const ctx = React.useContext(TabsContext);
      return (
        <button type="button" onClick={() => ctx.onValueChange?.(value)}>
          {children}
        </button>
      );
    },
    TabsContent: ({ value, children }: any) => {
      const ctx = React.useContext(TabsContext);
      return ctx.value === value ? (
        <div data-testid={`tab-${value}`}>{children}</div>
      ) : null;
    },
  };
});

vi.mock("@/components/settings/ThemeSettings", () => ({
  ThemeSettings: () => <div data-testid="theme-settings">theme</div>,
}));

vi.mock("@/components/settings/WindowSettings", () => ({
  WindowSettings: ({ onChange }: any) => (
    <button onClick={() => onChange({ minimizeToTrayOnClose: false })}>
      window-settings
    </button>
  ),
}));

vi.mock("@/components/settings/DirectorySettings", async () => {
  const actual = await vi.importActual<
    typeof import("@/components/settings/DirectorySettings")
  >("@/components/settings/DirectorySettings");
  return actual;
});

vi.mock("@/components/settings/ImportExportSection", () => ({
  ImportExportSection: ({
    status,
    selectedFile,
    errorMessage,
    isImporting,
    onSelectFile,
    onImport,
    onExport,
    onClear,
  }: any) => (
    <div>
      <div data-testid="import-status">{status}</div>
      <div data-testid="selected-file">{selectedFile || "none"}</div>
      <button onClick={onSelectFile}>Select SQL File</button>
      <button onClick={onImport} disabled={!selectedFile || isImporting}>
        {isImporting ? "Importing..." : "Import"}
      </button>
      <button onClick={onExport}>Export SQL Backup</button>
      <button onClick={onClear}>Clear</button>
      {errorMessage ? <span>{errorMessage}</span> : null}
    </div>
  ),
}));

vi.mock("@/components/settings/AboutSection", () => ({
  AboutSection: ({ isPortable }: any) => <div>about:{String(isPortable)}</div>,
}));

const renderDialog = (
  props?: Partial<React.ComponentProps<typeof SettingsPage>>,
) => {
  const client = new QueryClient();
  return render(
    <QueryClientProvider client={client}>
      <Suspense fallback={<div data-testid="loading">loading</div>}>
        <SettingsPage open onOpenChange={() => {}} {...props} />
      </Suspense>
    </QueryClientProvider>,
  );
};

beforeEach(() => {
  resetProviderState();
  toastSuccessMock.mockReset();
  toastErrorMock.mockReset();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("SettingsPage integration", () => {
  it("loads default settings from MSW", async () => {
    renderDialog();

    await waitFor(() =>
      expect(screen.getByTestId("theme-settings")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByText("Advanced"));
    fireEvent.click(screen.getByText("Configuration Directory"));
    const appInput = await screen.findByPlaceholderText(
      "e.g., C:\\Users\\Administrator\\.cc-switch",
    );
    expect((appInput as HTMLInputElement).value).toBe("/home/mock/.cc-switch");
  });

  it("imports configuration and triggers success callback", async () => {
    const onImportSuccess = vi.fn();
    renderDialog({ onImportSuccess });

    await waitFor(() =>
      expect(screen.getByTestId("theme-settings")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByText("Advanced"));
    fireEvent.click(screen.getByText("Data Management"));
    fireEvent.click(screen.getByText("Select SQL File"));
    await waitFor(() =>
      expect(screen.getByTestId("selected-file").textContent).toContain(
        "/mock/import-settings.json",
      ),
    );

    fireEvent.click(screen.getByText("Import"));
    await waitFor(() => expect(toastSuccessMock).toHaveBeenCalled());
    await waitFor(() => expect(onImportSuccess).toHaveBeenCalled(), {
      timeout: 4000,
    });
    expect(getSettings().language).toBe("en");
  });

  it("saves settings and handles restart prompt", async () => {
    renderDialog();

    await waitFor(() =>
      expect(screen.getByTestId("theme-settings")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByText("Advanced"));
    fireEvent.click(screen.getByText("Configuration Directory"));
    const appInput = await screen.findByPlaceholderText(
      "e.g., C:\\Users\\Administrator\\.cc-switch",
    );
    fireEvent.change(appInput, { target: { value: "/custom/app" } });
    fireEvent.click(screen.getByText("Save"));

    await waitFor(() => expect(toastSuccessMock).toHaveBeenCalled());
    await screen.findByText("Restart Required");
    fireEvent.click(screen.getByText("Restart Later"));
    await waitFor(() =>
      expect(screen.queryByText("Restart Required")).not.toBeInTheDocument(),
    );

    expect(getAppConfigDirOverride()).toBe("/custom/app");
  });

  it("allows browsing and resetting directories", async () => {
    renderDialog();

    await waitFor(() =>
      expect(screen.getByTestId("theme-settings")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByText("Advanced"));
    fireEvent.click(screen.getByText("Configuration Directory"));

    const browseButtons = screen.getAllByTitle("Browse Directory");
    const resetButtons = screen.getAllByTitle(
      "Reset to default directory (takes effect after saving)",
    );

    const appInput = (await screen.findByPlaceholderText(
      "e.g., C:\\Users\\Administrator\\.cc-switch",
    )) as HTMLInputElement;
    expect(appInput.value).toBe("/home/mock/.cc-switch");

    fireEvent.click(browseButtons[0]);
    await waitFor(() =>
      expect(appInput.value).toBe("/home/mock/.cc-switch/picked"),
    );

    fireEvent.click(resetButtons[0]);
    await waitFor(() => expect(appInput.value).toBe("/home/mock/.cc-switch"));

    const claudeInput = (await screen.findByPlaceholderText(
      "e.g., /home/<your-username>/.claude",
    )) as HTMLInputElement;
    fireEvent.change(claudeInput, { target: { value: "/custom/claude" } });
    await waitFor(() => expect(claudeInput.value).toBe("/custom/claude"));

    fireEvent.click(browseButtons[1]);
    await waitFor(() =>
      expect(claudeInput.value).toBe("/custom/claude/picked"),
    );

    fireEvent.click(resetButtons[1]);
    await waitFor(() => expect(claudeInput.value).toBe("/home/mock/.claude"));
  });

  it("notifies when export fails", async () => {
    renderDialog();

    await waitFor(() =>
      expect(screen.getByTestId("theme-settings")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByText("Advanced"));
    fireEvent.click(screen.getByText("Data Management"));

    server.use(
      http.post("http://tauri.local/save_file_dialog", () =>
        HttpResponse.json(null),
      ),
    );
    fireEvent.click(screen.getByText("Export SQL Backup"));

    await waitFor(() => expect(toastErrorMock).toHaveBeenCalled());
    const cancelMessage = toastErrorMock.mock.calls.at(-1)?.[0] as string;
    expect(cancelMessage).toMatch(/Please choose a valid SQL backup file/);

    toastErrorMock.mockClear();

    server.use(
      http.post("http://tauri.local/save_file_dialog", () =>
        HttpResponse.json("/mock/export-settings.json"),
      ),
      http.post("http://tauri.local/export_config_to_file", () =>
        HttpResponse.json({ success: false, message: "disk-full" }),
      ),
    );

    fireEvent.click(screen.getByText("Export SQL Backup"));

    await waitFor(() => expect(toastErrorMock).toHaveBeenCalled());
    const exportMessage = toastErrorMock.mock.calls.at(-1)?.[0] as string;
    expect(exportMessage).toContain("disk-full");
    expect(toastSuccessMock).not.toHaveBeenCalled();
  });
});
