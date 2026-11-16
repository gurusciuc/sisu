// Included in the window translation unit to share its native typography and
// palette.
struct Preferences {
  int vertical, width, google, reopen, light, blocked;
};
struct DialogContent {
  int kind;
  std::wstring query;
  const wchar_t *const *names = nullptr;
  int count = 0, selection = -1;
};
enum DialogIds {
  TabTop = 200,
  TabLeft,
  ProviderDuck,
  ProviderGoogle,
  StartupBlank,
  StartupLast,
  WidthField,
  PickImport,
  OpenPrivacy,
  OpenLibrary,
  LibraryList,
  RemoveRow,
  FindField
};
HWND dialogControl(HWND h, const wchar_t *klass, const wchar_t *title, int id,
                   DWORD style, int x, int y, int w, int height) {
  HWND c = CreateWindowExW(
      0, klass, title, WS_CHILD | WS_VISIBLE | style, px(x), px(y), px(w),
      px(height), h, (HMENU)(INT_PTR)id, GetModuleHandleW(nullptr), nullptr);
  SendMessageW(c, WM_SETFONT, (WPARAM)labelFont, TRUE);
  return c;
}
void dialogButton(HWND h, const wchar_t *title, int id, int x, int y,
                  int width) {
  dialogControl(h, L"BUTTON", title, id, WS_TABSTOP | BS_OWNERDRAW, x, y, width,
                34);
}
void heading(HWND h, const wchar_t *name, int y) {
  dialogControl(h, L"STATIC", name, 0, SS_LEFT, 24, y, 400, 22);
}
void makePair(HWND h, const wchar_t *label, int y, int id, const wchar_t *a,
              const wchar_t *b) {
  heading(h, label, y);
  dialogButton(h, a, id, 24, y + 24, 180);
  dialogButton(h, b, id + 1, 210, y + 24, 180);
}
INT_PTR CALLBACK dialogProc(HWND h, UINT m, WPARAM w, LPARAM l) {
  auto data = (DialogContent *)GetWindowLongPtrW(h, DWLP_USER);
  if (m == WM_INITDIALOG) {
    data = (DialogContent *)l;
    SetWindowLongPtrW(h, DWLP_USER, l);
    const wchar_t *title = data->kind == 46          ? L"Import browser data"
                           : data->kind == Bookmarks ? L"Bookmarks"
                                                     : L"Find on page";
    SetWindowTextW(h, title);
    BOOL dark = !lightMode;
    DwmSetWindowAttribute(h, 20, &dark, sizeof(dark));
    DwmSetWindowAttribute(h, 35, &paper, sizeof(paper));
    DwmSetWindowAttribute(h, 36, &ink, sizeof(ink));
    RECT owner;
    GetWindowRect(window, &owner);
    int width = px(436), height = px(data->kind == 46          ? 420
                                     : data->kind == Bookmarks ? 388
                                                               : 185);
    SetWindowPos(h, nullptr, (owner.left + owner.right - width) / 2,
                 (owner.top + owner.bottom - height) / 2, width, height,
                 SWP_NOZORDER);
    if (data->kind == 46) {
      heading(h, L"Choose what to import", 18);
      dialogButton(h, L"Bookmarks · HTML export", PickImport, 24, 58, 366);
      dialogButton(h, L"Cookies · JSON export", 281, 24, 101, 366);
      dialogButton(h, L"Browser import tools", 282, 24, 144, 366);
      dialogButton(h, L"Password import tools", 283, 24, 187, 366);
      dialogControl(
          h, L"STATIC",
          L"Cookie files contain login sessions. Existing matching cookies are "
          L"replaced. Encrypted browser databases and partitioned-cookie "
          L"imports need separate migration support.",
          0, SS_LEFT, 24, 240, 366, 65);
      dialogButton(h, L"Cancel", IDCANCEL, 288, 325, 102);
    } else if (data->kind == Bookmarks) {
      heading(h, data->count ? L"Saved pages" : L"No saved pages yet", 18);
      auto list = dialogControl(h, L"LISTBOX", L"Saved pages", LibraryList,
                                WS_TABSTOP | WS_VSCROLL | LBS_NOTIFY |
                                    LBS_NOINTEGRALHEIGHT | WS_BORDER,
                                24, 46, 366, 211);
      for (int i = 0; i < data->count; i++)
        SendMessageW(list, LB_ADDSTRING, 0, (LPARAM)data->names[i]);
      if (data->count)
        SendMessageW(list, LB_SETCURSEL, 0, 0);
      dialogButton(h, L"Remove", RemoveRow, 24, 278, 92);
      dialogButton(h, L"Close", IDCANCEL, 201, 278, 86);
      dialogButton(h, L"Open", IDOK, 297, 278, 93);
      EnableWindow(GetDlgItem(h, RemoveRow), data->count != 0);
      EnableWindow(GetDlgItem(h, IDOK), data->count != 0);
    } else {
      heading(h, L"Find in this page", 15);
      auto e = dialogControl(h, L"EDIT", data->query.c_str(), FindField,
                             WS_TABSTOP | ES_AUTOHSCROLL | WS_BORDER, 24, 40,
                             366, 28);
      SendMessageW(e, EM_SETLIMITTEXT, 512, 0);
      SendMessageW(e, EM_SETSEL, 0, -1);
      dialogButton(h, L"Cancel", IDCANCEL, 201, 86, 86);
      dialogButton(h, L"Find", IDOK, 297, 86, 93);
    }
    return TRUE;
  }
  if (!data)
    return FALSE;
  switch (m) {
  case WM_CTLCOLORDLG:
  case WM_CTLCOLORSTATIC:
  case WM_CTLCOLORLISTBOX:
  case WM_CTLCOLOREDIT: {
    auto dc = (HDC)w;
    auto c = (m == WM_CTLCOLORLISTBOX || m == WM_CTLCOLOREDIT) ? active : paper;
    SetTextColor(dc, ink);
    SetBkColor(dc, c);
    SetDCBrushColor(dc, c);
    return (INT_PTR)GetStockObject(DC_BRUSH);
  }
  case WM_DRAWITEM: {
    auto d = (DRAWITEMSTRUCT *)l;
    auto r = d->rcItem;
    int id = d->CtlID;
    auto dc = d->hDC;
    bool selected = false;
    bool primary = id == IDOK;
    colorRect(dc, r, paper);
    rounded(dc, r, primary ? ink : (selected ? hoverFill : active),
            selected ? muted : line, 12);
    wchar_t label[96]{};
    GetWindowTextW(d->hwndItem, label, 96);
    text(dc, label, r,
         primary ? paper : (IsWindowEnabled(d->hwndItem) ? ink : muted),
         labelFont, DT_CENTER);
    if ((d->itemState & ODS_FOCUS) && !(d->itemState & ODS_NOFOCUSRECT)) {
      InflateRect(&r, -4, -4);
      DrawFocusRect(dc, &r);
    }
    return TRUE;
  }
  case WM_COMMAND: {
    int id = LOWORD(w);
    if (data->kind == 46 &&
        (id == PickImport || id == 281 || id == 282 || id == 283)) {
      EndDialog(h, id == PickImport ? 1 : id == 281 ? 2 : id == 282 ? 3 : 4);
      return TRUE;
    }
    if (id == IDCANCEL) {
      EndDialog(h, 0);
      return TRUE;
    }
    if (id == IDOK || id == RemoveRow ||
        (id == LibraryList && HIWORD(w) == LBN_DBLCLK)) {
      if (data->kind == Bookmarks) {
        data->selection =
            (int)SendDlgItemMessageW(h, LibraryList, LB_GETCURSEL, 0, 0);
        if (data->selection < 0)
          return TRUE;
      } else {
        wchar_t q[513]{};
        GetDlgItemTextW(h, FindField, q, 513);
        data->query = q;
      }
      EndDialog(h, id == RemoveRow ? 2 : 1);
      return TRUE;
    }
    break;
  }
  case WM_CLOSE:
    EndDialog(h, 0);
    return TRUE;
  }
  return FALSE;
}
int showDialog(DialogContent &data) {
  struct Template {
    DLGTEMPLATE dlg;
    WORD menu = 0, klass = 0, title = 0;
  } dialog{};
  dialog.dlg.style = WS_POPUP | WS_CAPTION | WS_SYSMENU | DS_MODALFRAME;
  dialog.dlg.cx = 275;
  dialog.dlg.cy = 300;
  return (int)DialogBoxIndirectParamW(GetModuleHandleW(nullptr), &dialog.dlg,
                                      window, dialogProc, (LPARAM)&data);
}
extern "C" int shell_library(const wchar_t *const *names, int count,
                             int *selection) {
  DialogContent data{Bookmarks};
  data.names = names;
  data.count = count;
  int result = showDialog(data);
  *selection = data.selection;
  return result;
}
extern "C" int shell_find(wchar_t *dest) {
  DialogContent data{Find};
  data.query = dest;
  if (showDialog(data) != 1)
    return 0;
  wcscpy_s(dest, 513, data.query.c_str());
  return 1;
}
extern "C" int shell_import_kind() {
  DialogContent data{46};
  return showDialog(data);
}
extern "C" bool shell_pick_file(wchar_t *dest, int count, int kind) {
  OPENFILENAMEW dlg{sizeof(dlg)};
  dlg.hwndOwner = GetActiveWindow();
  dlg.lpstrFile = dest;
  dlg.nMaxFile = count;
  dlg.lpstrTitle =
      kind == 1
          ? L"Import cookies from a JSON export"
          : L"Import a bookmarks HTML export from Brave, Chrome, or Firefox";
  dlg.lpstrFilter = kind == 1
                        ? L"Cookies JSON (*.json)\0*.json\0\0"
                        : L"Bookmarks HTML (*.html; *.htm)\0*.html;*.htm\0\0";
  dlg.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;
  return GetOpenFileNameW(&dlg) != 0;
}
extern "C" void shell_notice(const wchar_t *title, const wchar_t *message) {
  MessageBoxW(GetActiveWindow(), message, title, MB_OK | MB_ICONINFORMATION);
}
extern "C" void shell_width(int width) { gutter = std::clamp(width, 92, 350); }
