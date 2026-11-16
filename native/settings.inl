// SPDX-License-Identifier: Apache-2.0
// Native profile settings share the selected tab's viewport. Only native
// control notifications reach Rust. The associated CEF document owns history.
namespace {
struct SettingsView {
  Preferences prefs{-1, -1, -1, -1, -1, -1};
  int scroll = 0, extent = 0;
  HFONT title = nullptr;
};
constexpr int SettingsTitle = 240, AppearanceHeading = 241,
              PlacementLabel = 242, WidthLabel = 243, BrowsingHeading = 244,
              SearchLabel = 245, StartupLabel = 246, DataHeading = 247,
              DataLabel = 248, ProfileLabel = 249, WidthApply = 250;
const COLORREF &settingsBg = surfaceColor;
constexpr int ThemeLabel = 260, ThemeDark = 261, ThemeLight = 262,
              BlockLabel = 263, BlockOn = 264, BlockOff = 265, SiteData = 266,
              ClearData = 267, DownloadsSettings = 268, LanguageSettings = 269,
              AccessibilitySettings = 270, AdvancedSettings = 271,
              ToolsLabel = 272, VersionLabel = 273, SiteDataLabel = 274,
              AccessLabel = 275, BlockingNote = 276;
SettingsView *settingsView(HWND h) {
  return reinterpret_cast<SettingsView *>(GetWindowLongPtrW(h, GWLP_USERDATA));
}
void positionSettings(HWND h) {
  auto s = settingsView(h);
  if (!s)
    return;
  RECT r{};
  GetClientRect(h, &r);
  int dpi = GetDpiForWindow(h), width = MulDiv(r.right, 96, dpi),
      height = MulDiv(r.bottom, 96, dpi);
  int content = std::min(720, std::max(180, width - 48));
  int x = (width - content) / 2, y = 28 - s->scroll;
  bool stacked = content < 520;
  auto move = [&](int id, int at, int top, int w, int ht) {
    place(GetDlgItem(h, id), px(at), px(top), px(w), px(ht));
  };
  auto group = [&](int id) {
    move(id, x, y, content, 24);
    y += 32;
  };
  auto row = [&](int label, int a, int b, bool number = false) {
    int right = stacked ? x : x + content / 2;
    int rw = stacked ? content : content - content / 2;
    move(label, x, y, stacked ? content : content / 2 - 8, stacked ? 24 : 34);
    if (stacked)
      y += 25;
    int first = number ? std::min(100, rw / 2) : (rw - 8) / 2;
    move(a, right, y, first, 33);
    move(b, right + first + 8, y,
         number ? std::min(86, rw - first - 8) : rw - first - 8, 33);
    y += 41;
  };
  move(SettingsTitle, x, y, content, 48);
  y += 51;
  move(ProfileLabel, x, y, content, 25);
  y += 47;
  group(AppearanceHeading);
  row(ThemeLabel, ThemeDark, ThemeLight);
  row(PlacementLabel, TabTop, TabLeft);
  row(WidthLabel, WidthField, WidthApply, true);
  y += 14;
  group(BrowsingHeading);
  row(SearchLabel, ProviderDuck, ProviderGoogle);
  row(StartupLabel, StartupBlank, StartupLast);
  y += 14;
  group(DataHeading);
  row(BlockLabel, BlockOn, BlockOff);
  move(BlockingNote, x, y, content, 24);
  y += 34;
  row(DataLabel, PickImport, OpenPrivacy);
  row(SiteDataLabel, SiteData, ClearData);
  row(ToolsLabel, DownloadsSettings, LanguageSettings);
  row(AccessLabel, AccessibilitySettings, AdvancedSettings);
  move(VersionLabel, x, y, content, 24);
  y += 34;
  s->extent = y + s->scroll + 21;
  int maxScroll = std::max(0, s->extent - height);
  if (s->scroll > maxScroll) {
    s->scroll = maxScroll;
    positionSettings(h);
    return;
  }
  SCROLLINFO info{sizeof(info),  SIF_PAGE | SIF_POS | SIF_RANGE, 0,
                  s->extent - 1, (UINT)std::max(1, height),      s->scroll};
  SetScrollInfo(h, SB_VERT, &info, TRUE);
}
void scrollSettings(HWND h, int amount, bool absolute = false) {
  auto s = settingsView(h);
  if (!s)
    return;
  RECT r{};
  GetClientRect(h, &r);
  int next = std::clamp(
      absolute ? amount : s->scroll + amount, 0,
      std::max(0, s->extent - MulDiv(r.bottom, 96, GetDpiForWindow(h))));
  if (next == s->scroll)
    return;
  s->scroll = next;
  positionSettings(h);
  InvalidateRect(h, nullptr, FALSE);
}
LRESULT CALLBACK preferenceKeys(HWND h, UINT m, WPARAM w, LPARAM l, UINT_PTR,
                                DWORD_PTR) {
  HWND panel = GetParent(h);
  int id = GetDlgCtrlID(h);
  if (m == WM_MOUSEWHEEL) {
    scrollSettings(panel, -GET_WHEEL_DELTA_WPARAM(w) / WHEEL_DELTA * 60);
    return 0;
  }
  if (m == WM_SETFOCUS) {
    RECT r{}, p{};
    GetWindowRect(h, &r);
    GetWindowRect(panel, &p);
    if (r.top < p.top)
      scrollSettings(panel, MulDiv(r.top - p.top, 96, GetDpiForWindow(h)) - 8);
    else if (r.bottom > p.bottom)
      scrollSettings(panel,
                     MulDiv(r.bottom - p.bottom, 96, GetDpiForWindow(h)) + 8);
  }
  if (m == WM_KEYDOWN) {
    if (w == VK_TAB && !(GetKeyState(VK_CONTROL) & 0x8000)) {
      keyboardFocus = true;
      auto next =
          GetNextDlgTabItem(panel, h, (GetKeyState(VK_SHIFT) & 0x8000) != 0);
      if (next) {
        SetFocus(next);
        return 0;
      }
    }
    if (w == VK_RETURN) {
      SendMessageW(panel, WM_COMMAND, id == WidthField ? WidthApply : id, 0);
      return 0;
    }
    if (w == VK_NEXT || w == VK_PRIOR) {
      scrollSettings(panel, w == VK_NEXT ? 240 : -240);
      return 0;
    }
  }
  return controlProc(h, m, w, l, 0, 0);
}
LRESULT CALLBACK settingsProc(HWND h, UINT m, WPARAM w, LPARAM l) {
  auto s = settingsView(h);
  switch (m) {
  case WM_CREATE: {
    s = new SettingsView();
    SetWindowLongPtrW(h, GWLP_USERDATA, (LONG_PTR)s);
    const auto add = [&](const wchar_t *label, int id,
                         bool interactive = false) {
      HWND c = dialogControl(h, interactive ? L"BUTTON" : L"STATIC", label, id,
                             interactive ? (WS_TABSTOP | BS_OWNERDRAW)
                                         : SS_LEFT | SS_CENTERIMAGE,
                             0, 0, 1, 1);
      if (interactive)
        SetWindowSubclass(c, preferenceKeys, 2, 0);
    };
    add(L"Settings", SettingsTitle);
    auto dpi = GetDpiForWindow(h);
    s->title =
        CreateFontW(-MulDiv(37, dpi, 96), 0, 0, 0, FW_NORMAL, 0, 0, 0,
                    DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0, fontFace());
    SendDlgItemMessageW(h, SettingsTitle, WM_SETFONT, (WPARAM)s->title, TRUE);
    add(privateMode ? L"Changes last for this private window."
                    : L"Preferences for this profile",
        ProfileLabel);
    add(L"Appearance", AppearanceHeading);
    add(L"Color mode", ThemeLabel);
    add(L"Dark", ThemeDark, true);
    add(L"Light", ThemeLight, true);
    add(L"Tab placement", PlacementLabel);
    add(L"Top", TabTop, true);
    add(L"Left", TabLeft, true);
    add(L"Sidebar width (92–350 px)", WidthLabel);
    HWND editWidth = dialogControl(
        h, L"EDIT", L"", WidthField,
        WS_TABSTOP | WS_BORDER | ES_NUMBER | ES_CENTER, 0, 0, 1, 1);
    if (accessibility)
      accessibility->SetHwndPropStr(editWidth, OBJID_CLIENT, CHILDID_SELF,
                                    PROPID_ACC_NAME,
                                    L"Sidebar width in pixels");
    SendMessageW(editWidth, EM_SETLIMITTEXT, 3, 0);
    SetWindowSubclass(editWidth, preferenceKeys, 2, 0);
    add(L"Apply", WidthApply, true);
    add(L"Browsing", BrowsingHeading);
    add(L"Search engine", SearchLabel);
    add(L"DuckDuckGo", ProviderDuck, true);
    add(L"Google", ProviderGoogle, true);
    add(L"On startup", StartupLabel);
    add(L"New tab", StartupBlank, true);
    add(L"Reopen last page", StartupLast, true);
    add(L"Privacy and data", DataHeading);
    add(L"Browser data", DataLabel);
    add(L"Native blocking", BlockLabel);
    add(L"On", BlockOn, true);
    add(L"Off", BlockOff, true);
    add(L"Import browser data…", PickImport, true);
    add(L"Reload open pages after changing blocking.", BlockingNote);
    add(L"Stored data", SiteDataLabel);
    add(L"Browser tools", ToolsLabel);
    add(L"More options", AccessLabel);
    add(L"Cookies and site data", SiteData, true);
    add(L"Clear browsing data", ClearData, true);
    add(L"Downloads", DownloadsSettings, true);
    add(L"Languages", LanguageSettings, true);
    add(L"Accessibility", AccessibilitySettings, true);
    add(L"Advanced settings", AdvancedSettings, true);
    add(L"Sisu 0.1.0 preview · Apache 2.0 · © 2026 Sisu contributors",
        VersionLabel);
    if (lab) {
      EnableWindow(GetDlgItem(h, BlockOn), FALSE);
      EnableWindow(GetDlgItem(h, BlockOff), FALSE);
    }

    add(L"Site permissions", OpenPrivacy, true);
    if (privateMode) {
      EnableWindow(GetDlgItem(h, PickImport), FALSE);
      EnableWindow(GetDlgItem(h, StartupLast), FALSE);
    }
    return 0;
  }
  case WM_NCDESTROY:
    if (s) {
      DeleteObject(s->title);
      delete s;
      SetWindowLongPtrW(h, GWLP_USERDATA, 0);
    }
    break;
  case WM_SIZE:
    positionSettings(h);
    return 0;
  case WM_ERASEBKGND: {
    RECT r;
    GetClientRect(h, &r);
    colorRect((HDC)w, r, settingsBg);
    return 1;
  }
  case WM_CTLCOLORSTATIC:
  case WM_CTLCOLOREDIT: {
    auto dc = (HDC)w;
    int id = GetDlgCtrlID((HWND)l);
    auto bg = m == WM_CTLCOLOREDIT ? active : settingsBg;
    SetBkColor(dc, bg);
    SetDCBrushColor(dc, bg);
    bool heading = id == AppearanceHeading || id == BrowsingHeading ||
                   id == DataHeading || id == ProfileLabel;
    SetTextColor(dc, heading ? muted : ink);
    return (LRESULT)GetStockObject(DC_BRUSH);
  }
  case WM_DRAWITEM: {
    auto d = (DRAWITEMSTRUCT *)l;
    int id = d->CtlID;
    auto box = d->rcItem;
    auto dc = d->hDC;
    bool on = (id == TabTop && !s->prefs.vertical) ||
              (id == TabLeft && s->prefs.vertical) ||
              (id == ProviderDuck && !s->prefs.google) ||
              (id == ProviderGoogle && s->prefs.google) ||
              (id == StartupBlank && !s->prefs.reopen) ||
              (id == StartupLast && s->prefs.reopen) ||
              (id == ThemeDark && !s->prefs.light) ||
              (id == ThemeLight && s->prefs.light) ||
              (id == BlockOn && s->prefs.blocked) ||
              (id == BlockOff && !s->prefs.blocked);
    bool enabled = IsWindowEnabled(d->hwndItem);
    colorRect(dc, box, settingsBg);
    rounded(dc, box, on ? ink : (pointerOver(d->hwndItem) ? line : active),
            on ? ink : line, 10);
    wchar_t label[64]{};
    GetWindowTextW(d->hwndItem, label, 64);
    text(dc, label, box, !enabled ? muted : (on ? paper : ink), labelFont,
         DT_CENTER);
    if ((d->itemState & ODS_FOCUS) && keyboardFocus) {
      InflateRect(&box, -4, -4);
      DrawFocusRect(dc, &box);
    }
    return TRUE;
  }
  case WM_COMMAND: {
    if (HIWORD(w) != BN_CLICKED)
      return 0;
    int id = LOWORD(w);
    if (id == ThemeDark || id == ThemeLight) {
      browser_action(43, id == ThemeLight, nullptr);
      return 0;
    }
    if (id == BlockOn || id == BlockOff) {
      browser_action(44, id == BlockOn, nullptr);
      return 0;
    }
    if (id >= SiteData && id <= AdvancedSettings) {
      browser_action(45, id - SiteData, nullptr);
      return 0;
    }
    if (id >= TabTop && id <= StartupLast) {
      browser_action(30 + id - TabTop, 0, nullptr);
      return 0;
    }
    if (id == WidthApply) {
      BOOL valid = FALSE;
      int v = GetDlgItemInt(h, WidthField, &valid, FALSE);
      if (valid && v >= 92 && v <= 350)
        browser_action(36, v, nullptr);
      else {
        MessageBoxW(h, L"Choose a sidebar width between 92 and 350 pixels.",
                    L"Sidebar width", MB_OK);
        SetFocus(GetDlgItem(h, WidthField));
      }
      return 0;
    }
    if (id == PickImport && !privateMode) {
      browser_action(Import, 0, nullptr);
      return 0;
    }
    if (id == OpenPrivacy) {
      browser_action(37, 0, nullptr);
      return 0;
    }
    break;
  }
  case WM_MOUSEWHEEL:
    scrollSettings(h, -GET_WHEEL_DELTA_WPARAM(w) / WHEEL_DELTA * 60);
    return 0;
  case WM_VSCROLL: {
    int code = LOWORD(w);
    if (code == SB_THUMBTRACK || code == SB_THUMBPOSITION) {
      SCROLLINFO si{sizeof(si), SIF_TRACKPOS};
      GetScrollInfo(h, SB_VERT, &si);
      scrollSettings(h, si.nTrackPos, true);
    } else if (code == SB_LINEDOWN || code == SB_LINEUP ||
               code == SB_PAGEDOWN || code == SB_PAGEUP)
      scrollSettings(h, code == SB_LINEDOWN   ? 28
                        : code == SB_LINEUP   ? -28
                        : code == SB_PAGEDOWN ? 220
                                              : -220);
    return 0;
  }
  }
  return DefWindowProcW(h, m, w, l);
}
} // namespace
extern "C" void shell_sync_settings(int id, bool visible, Preferences prefs) {
  auto it = std::find_if(tabs.begin(), tabs.end(),
                         [id](const Tab &t) { return t.id == id; });
  if (it == tabs.end() || (!visible && !it->settingsVisible))
    return;
  if (visible && !it->settings) {
    WNDCLASSW wc{};
    wc.lpfnWndProc = settingsProc;
    wc.hInstance = GetModuleHandleW(nullptr);
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.lpszClassName = L"BrowserSettingsPage";
    RegisterClassW(&wc);
    RECT r;
    GetClientRect(it->page, &r);
    it->settings = CreateWindowExW(
        WS_EX_CONTROLPARENT, wc.lpszClassName, L"Settings",
        WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_VSCROLL, 0, 0,
        r.right, r.bottom, it->page, nullptr, wc.hInstance, nullptr);
    if (!it->settings)
      return;
  }
  if (visible) {
    auto s = settingsView(it->settings);
    if (!s)
      return;
    if (s->prefs.width != prefs.width)
      SetDlgItemInt(it->settings, WidthField, prefs.width, FALSE);
    s->prefs = prefs;
    for (int control = ThemeDark; control <= BlockOff; control++)
      if (auto h = GetDlgItem(it->settings, control))
        InvalidateRect(h, nullptr, FALSE);
    for (int i = TabTop; i <= StartupLast; i++)
      InvalidateRect(GetDlgItem(it->settings, i), nullptr, FALSE);
  }
  if (it->settingsVisible != visible) {
    it->settingsVisible = visible;
    for (HWND child = GetWindow(it->page, GW_CHILD); child;
         child = GetWindow(child, GW_HWNDNEXT))
      if (child != it->settings)
        ShowWindow(child, visible ? SW_HIDE : SW_SHOWNA);
    if (visible) {
      SetWindowPos(it->settings, HWND_TOP, 0, 0, 0, 0,
                   SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW | SWP_NOACTIVATE);
      if (current == id)
        SetFocus(GetDlgItem(it->settings, TabTop));
    } else
      ShowWindow(it->settings, SW_HIDE);
  }
}
extern "C" void shell_focus_settings(int id) {
  for (const auto &t : tabs)
    if (t.id == id && t.settingsVisible) {
      SetFocus(GetDlgItem(t.settings, TabTop));
      break;
    }
}
