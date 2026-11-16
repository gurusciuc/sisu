// Native Win32 controls for the Windows feasibility build. CEF owns every page
// HWND.
#define UNICODE
#define _UNICODE
#define NOMINMAX
#include <windows.h>

#include <commctrl.h>
#include <commdlg.h>
#include <uxtheme.h>
#pragma comment(lib, "comdlg32.lib")
#pragma comment(lib, "uxtheme.lib")
#include <imm.h>
#include <initguid.h>
#include <oleacc.h>
#include <windowsx.h>
#pragma comment(lib, "oleacc.lib")
#pragma comment(lib, "ole32.lib")
#pragma comment(lib, "oleaut32.lib")
#pragma comment(lib, "uuid.lib")
#pragma comment(lib, "imm32.lib")
#include <dwmapi.h>
#pragma comment(lib, "gdi32.lib")
#include <algorithm>
#include <string>
#include <vector>
#pragma comment(lib, "comctl32.lib")
#pragma comment(lib, "dwmapi.lib")

extern "C" void browser_action(int command, int tab, const wchar_t *text);
enum Command {
  Back = 1,
  Forward,
  Reload,
  NewTab,
  CloseTab,
  Placement,
  Address,
  SelectTab,
  Protection,
  CloseAllTabs,
  More,
  AllTabs,
  ResizeStrip,
  FocusAddress,
  Cycle,
  Manager,
  Reference,
  Settings,
  PrivateWindow,
  Bookmarks,
  AddBookmark,
  Import,
  History,
  Downloads,
  Find,
  ZoomIn,
  ZoomOut,
  ZoomReset,
  FindNext
};
namespace {
const COLORREF ink = RGB(241, 242, 243), muted = RGB(167, 170, 176),
               paper = RGB(22, 23, 25), band = RGB(14, 15, 17),
               active = RGB(38, 39, 42), accent = RGB(150, 169, 196),
               line = RGB(61, 63, 68);
struct Tab {
  int id;
  HWND button, page;
  std::wstring title;
  HWND settings = nullptr;
  bool settingsVisible = false;
};
std::vector<Tab> tabs;
HWND window, edit, viewport, buttons[18]{};
HWND retiringPages;
HFONT font, labelFont, symbolFont, smallSymbol;
HWND hovered = nullptr;
bool keyboardFocus = false;
int presented = -1;
int pending = -1;
int current = -1, gutter = 216, addressX, addressY, addressW, scrollIndex = 0;
bool left = false, resizing = false, loading = false, blocking = true,
     lab = false, privateMode = false;
std::wstring activeUrl;
IAccPropServices *accessibility = nullptr;
int px(int n) { return MulDiv(n, window ? GetDpiForWindow(window) : 96, 96); }
void colorRect(HDC dc, RECT box, COLORREF color) {
  SetDCBrushColor(dc, color);
  FillRect(dc, &box, (HBRUSH)GetStockObject(DC_BRUSH));
}
void rounded(HDC dc, RECT box, COLORREF fill, COLORREF border, int radius) {
  auto pen = CreatePen(PS_SOLID, 1, border);
  auto old = SelectObject(dc, pen);
  auto brush = SelectObject(dc, GetStockObject(DC_BRUSH));
  SetDCBrushColor(dc, fill);
  RoundRect(dc, box.left, box.top, box.right, box.bottom, px(radius),
            px(radius));
  SelectObject(dc, brush);
  SelectObject(dc, old);
  DeleteObject(pen);
}
wchar_t glyph(int id) {
  switch (id) {
  case Back:
    return 0xe72b;
  case Forward:
    return 0xe72a;
  case Reload:
    return loading ? 0xe711 : 0xe72c;
  case NewTab:
    return 0xe710;
  case CloseTab:
    return 0xe711;
  case Placement:
    return 0xe90c;
  case AllTabs:
    return 0xe70d;
  case More:
    return 0xe712;
  case Protection:
    return 0xe83d;
  case Settings:
    return 0xe713;
  case PrivateWindow:
    return 0xe72e;
  case Bookmarks:
  case AddBookmark:
    return 0xe734;
  case Import:
    return 0xe8b5;
  case History:
    return 0xe81c;
  case Downloads:
    return 0xe896;
  case Find:
    return 0xe721;
  case Manager:
    return 0xea86;
  default:
    return 0xe7c3;
  }
}
void icon(HDC dc, int id, int x, int y, COLORREF c) {
  auto old = SelectObject(dc, id == CloseTab ? smallSymbol : symbolFont);
  const wchar_t ch = glyph(id);
  RECT box{x - px(13), y - px(13), x + px(13), y + px(13)};
  SetTextColor(dc, c);
  SetBkMode(dc, TRANSPARENT);
  DrawTextW(dc, &ch, 1, &box,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
  SelectObject(dc, old);
}
void text(HDC dc, std::wstring const &str, RECT box, COLORREF c, HFONT f,
          UINT align = DT_LEFT) {
  SelectObject(dc, f);
  SetTextColor(dc, c);
  SetBkMode(dc, TRANSPARENT);
  DrawTextW(dc, str.c_str(), -1, &box,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX | align);
}
const wchar_t *fontFace() {
  static const wchar_t *face = []() {
    auto dc = GetDC(nullptr);
    for (auto family : {L"Helvetica Neue", L"Helvetica", L"Arial"}) {
      LOGFONTW lf{};
      lf.lfCharSet = DEFAULT_CHARSET;
      wcscpy_s(lf.lfFaceName, family);
      bool exists = false;
      EnumFontFamiliesExW(
          dc, &lf,
          [](const LOGFONTW *, const TEXTMETRICW *, DWORD, LPARAM data) -> int {
            *reinterpret_cast<bool *>(data) = true;
            return 0;
          },
          reinterpret_cast<LPARAM>(&exists), 0);
      if (exists) {
        ReleaseDC(nullptr, dc);
        return family;
      }
    }
    ReleaseDC(nullptr, dc);
    return L"Arial";
  }();
  return face;
}
void setFonts() {
  auto make = [](int height, int weight) {
    return CreateFontW(-px(height), 0, 0, 0, weight, 0, 0, 0, DEFAULT_CHARSET,
                       0, 0, CLEARTYPE_QUALITY, 0, fontFace());
  };
  auto next = make(15, FW_NORMAL), nextSmall = make(14, FW_NORMAL);
  if (edit)
    SendMessageW(edit, WM_SETFONT, (WPARAM)next, 1);
  if (font)
    DeleteObject(font);
  if (labelFont)
    DeleteObject(labelFont);
  if (symbolFont)
    DeleteObject(symbolFont);
  if (smallSymbol)
    DeleteObject(smallSymbol);
  auto symbols = [](int size) {
    return CreateFontW(-px(size), 0, 0, 0, FW_NORMAL, 0, 0, 0, DEFAULT_CHARSET,
                       0, 0, ANTIALIASED_QUALITY, 0, L"Segoe Fluent Icons");
  };
  symbolFont = symbols(16);
  smallSymbol = symbols(11);
  font = next;
  labelFont = nextSmall;
}
void place(HWND w, int x, int y, int width, int height) {
  SetWindowPos(w, nullptr, x, y, std::max(1, width), std::max(1, height),
               SWP_NOZORDER | SWP_NOACTIVATE);
}
void sizeSettings(HWND panel);
void layout() {
  if (!window || !edit)
    return;
  RECT r{};
  GetClientRect(window, &r);
  const int top = left ? 0 : px(47), nav = px(52),
            side = left ? std::min(px(gutter), int(r.right) / 2) : 0;
  const int leftHeader = px(48);
  const int maxTabs =
      left ? std::max(1, (int(r.bottom) - leftHeader - px(98)) / px(37))
           : std::max(1, (int(r.right) - px(145)) / px(112));
  scrollIndex =
      std::clamp(scrollIndex, 0, std::max(0, (int)tabs.size() - maxTabs));
  const int tabWidth =
      std::min(px(204), (int(r.right) - px(145)) /
                            std::max(1, std::min(maxTabs, (int)tabs.size())));
  int shown = 0;
  bool closeVisible = false;
  for (int i = 0; i < (int)tabs.size(); ++i) {
    bool show = i >= scrollIndex && i < scrollIndex + maxTabs;
    if (show) {
      if (left)
        place(tabs[i].button, px(8), leftHeader + shown * px(37), side - px(18),
              px(33));
      else
        place(tabs[i].button, px(12) + shown * tabWidth, px(7),
              tabWidth - px(4), px(34));
      if (tabs[i].id == current) {
        int x = left ? side - px(36) : px(12) + (shown + 1) * tabWidth - px(34);
        int y = left ? leftHeader + shown * px(37) + px(5) : px(12);
        place(buttons[CloseTab], x, y, px(24), px(23));
        SetWindowPos(buttons[CloseTab], HWND_TOP, 0, 0, 0, 0,
                     SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE);
        closeVisible = true;
      }
      ++shown;
    }
    ShowWindow(tabs[i].button, show ? SW_SHOWNA : SW_HIDE);
  }
  ShowWindow(buttons[CloseTab], closeVisible ? SW_SHOWNA : SW_HIDE);
  place(buttons[Placement], left ? px(8) : r.right - px(45), px(8), px(34),
        px(32));
  int plusX = left ? px(9) : px(12) + shown * tabWidth;
  int plusY = left ? leftHeader + shown * px(37) : px(8);
  place(buttons[NewTab], plusX, plusY, px(32), px(32));
  place(buttons[AllTabs], left ? side - px(45) : r.right - px(82), px(8),
        px(29), px(32));
  ShowWindow(buttons[AllTabs],
             (int)tabs.size() > maxTabs ? SW_SHOWNA : SW_HIDE);
  for (int i = 0; i < 3; i++)
    place(buttons[Back + i], side + px(10 + 34 * i), top + px(9), px(31),
          px(32));
  const int protectWidth = r.right - side > px(630) ? px(124) : px(36);
  place(buttons[More], r.right - px(43), top + px(9), px(32), px(32));
  place(buttons[Protection], r.right - px(50) - protectWidth, top + px(9),
        protectWidth, px(32));
  addressX = side + px(122);
  addressY = top + px(8);
  addressW = std::max(px(55), int(r.right) - px(61) - protectWidth - addressX);
  place(edit, addressX + px(12), addressY + px(6), addressW - px(24), px(22));
  place(viewport, side, top + nav, r.right - side, r.bottom - top - nav);
  RECT v{};
  GetClientRect(viewport, &v);
  for (auto &t : tabs) {
    if (t.id != current && t.id != presented)
      continue;
    place(t.page, 0, 0, v.right, v.bottom);
    for (HWND child = GetWindow(t.page, GW_CHILD); child;
         child = GetWindow(child, GW_HWNDNEXT))
      place(child, 0, 0, v.right, v.bottom);
    if (t.id == presented && !IsWindowVisible(t.page))
      SetWindowPos(t.page, HWND_TOP, 0, 0, 0, 0,
                   SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
  }
  for (auto &t : tabs)
    if (t.id != presented && IsWindowVisible(t.page))
      ShowWindow(t.page, SW_HIDE);
  InvalidateRect(window, nullptr, FALSE);
}
LRESULT CALLBACK controlProc(HWND h, UINT m, WPARAM w, LPARAM l, UINT_PTR,
                             DWORD_PTR) {
  if (m == WM_MOUSEMOVE && hovered != h) {
    hovered = h;
    TRACKMOUSEEVENT track{sizeof(track), TME_LEAVE, h, 0};
    TrackMouseEvent(&track);
    InvalidateRect(h, nullptr, FALSE);
  }
  if (m == WM_MOUSELEAVE && hovered == h) {
    hovered = nullptr;
    InvalidateRect(h, nullptr, FALSE);
  }
  if (m == WM_LBUTTONDOWN)
    keyboardFocus = false;
  if (m == WM_KEYDOWN) {
    keyboardFocus = true;
    bool ctrl = (GetKeyState(VK_CONTROL) & 0x8000) != 0;
    int command = 0;
    if (ctrl) {
      switch (w) {
      case VK_OEM_COMMA:
        command = Settings;
        break;
      case 'D':
        command = AddBookmark;
        break;
      case 'F':
        command = Find;
        break;
      case 'H':
        command = History;
        break;
      case 'J':
        command = Downloads;
        break;
      case 'N':
        if (GetKeyState(VK_SHIFT) & 0x8000)
          command = PrivateWindow;
        break;
      case 'L':
        command = FocusAddress;
        break;
      case 'T':
        command = NewTab;
        break;
      case 'W':
        command = CloseTab;
        break;
      case 'R':
        command = Reload;
        break;
      case VK_TAB:
        command = Cycle;
        break;
      }
    }
    if (h == edit && w == VK_RETURN) {
      HIMC im = ImmGetContext(h);
      bool composing =
          im && ImmGetCompositionStringW(im, GCS_COMPSTR, nullptr, 0) > 0;
      if (im)
        ImmReleaseContext(h, im);
      if (!composing) {
        std::wstring s(GetWindowTextLengthW(h) + 1, 0);
        GetWindowTextW(h, s.data(), (int)s.size());
        browser_action(Address, 0, s.c_str());
        return 0;
      }
    }
    if (command) {
      PostMessageW(window, WM_COMMAND, command, 0);
      return 0;
    }
    if (w == VK_TAB && !ctrl) {
      HWND next =
          GetNextDlgTabItem(window, h, (GetKeyState(VK_SHIFT) & 0x8000) != 0);
      if (next) {
        SetFocus(next);
        return 0;
      }
    }
  }
  if (h == edit && m == WM_CHAR && w == VK_RETURN)
    return 0;
  if (h == edit && (m == WM_SETFOCUS || m == WM_KILLFOCUS))
    InvalidateRect(window, nullptr, FALSE);
  return DefSubclassProc(h, m, w, l);
}
HWND button(int id, const wchar_t *name) {
  auto b = CreateWindowExW(
      0, L"BUTTON", name,
      WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_CLIPSIBLINGS | BS_OWNERDRAW, 0, 0,
      1, 1, window, (HMENU)(INT_PTR)id, GetModuleHandleW(nullptr), nullptr);
  if (accessibility) {
    VARIANT role{};
    role.vt = VT_I4;
    role.lVal = id >= 100 ? ROLE_SYSTEM_PAGETAB : ROLE_SYSTEM_PUSHBUTTON;
    accessibility->SetHwndProp(b, OBJID_CLIENT, CHILDID_SELF, PROPID_ACC_ROLE,
                               role);
  }
  SetWindowSubclass(b, controlProc, 1, 0);
  return b;
}
struct MenuRow {
  int id;
  std::wstring label, key;
};
void popup(int id) {
  HMENU m = CreatePopupMenu();
  std::vector<MenuRow> rows;
  if (id == AllTabs) {
    for (size_t i = 0; i < tabs.size(); i++)
      rows.push_back({100 + (int)i, tabs[i].title, L""});
  } else
    rows = {{NewTab, L"New tab", L"Ctrl+T"},
            {PrivateWindow, L"New private window", L"Ctrl+Shift+N"},
            {0, L"", L""},
            {Bookmarks, L"Bookmarks", L""},
            {AddBookmark, L"Bookmark this page", L"Ctrl+D"},
            {Import, L"Import bookmarks…", L""},
            {History, L"History", L"Ctrl+H"},
            {Downloads, L"Downloads", L"Ctrl+J"},
            {0, L"", L""},
            {Find, L"Find on page…", L"Ctrl+F"},
            {Settings, L"Settings", L"Ctrl+,"},
            {Manager, L"Extensions", L""}};
  auto bg = CreateSolidBrush(paper);
  MENUINFO mi{sizeof(mi)};
  mi.fMask = MIM_BACKGROUND;
  mi.hbrBack = bg;
  SetMenuInfo(m, &mi);
  for (auto &row : rows) {
    MENUITEMINFOW info{sizeof(info)};
    info.fMask = MIIM_FTYPE | MIIM_DATA | MIIM_ID | MIIM_STRING;
    info.fType = MFT_OWNERDRAW | (row.id ? 0 : MFT_SEPARATOR);
    info.wID = row.id;
    info.dwItemData = (ULONG_PTR)&row;
    info.dwTypeData = row.label.data();
    InsertMenuItemW(m, GetMenuItemCount(m), TRUE, &info);
  }
  RECT rect{};
  GetWindowRect(buttons[id], &rect);
  const int selected =
      TrackPopupMenu(m, TPM_RETURNCMD | TPM_RIGHTALIGN | TPM_RIGHTBUTTON,
                     rect.right, rect.bottom + px(5), 0, window, nullptr);
  DestroyMenu(m);
  DeleteObject(bg);
  if (selected >= 100 && selected - 100 < (int)tabs.size())
    browser_action(SelectTab, tabs[selected - 100].id, nullptr);
  else if (selected)
    browser_action(selected, 0, nullptr);
}
void drawMenu(DRAWITEMSTRUCT *d) {
  auto &row = *reinterpret_cast<MenuRow *>(d->itemData);
  auto r = d->rcItem;
  HDC dc = d->hDC;
  colorRect(dc, r, paper);
  if (!row.id) {
    RECT rule{r.left + px(13), (r.top + r.bottom) / 2, r.right - px(13),
              (r.top + r.bottom) / 2 + 1};
    colorRect(dc, rule, line);
    return;
  }
  if (d->itemState & ODS_SELECTED) {
    auto hover = r;
    InflateRect(&hover, -px(4), -px(2));
    rounded(dc, hover, active, active, 8);
  }
  icon(dc, row.id, r.left + px(22), (r.top + r.bottom) / 2, muted);
  RECT title{r.left + px(42), r.top, r.right - px(82), r.bottom};
  text(dc, row.label, title, ink, labelFont);
  RECT key{r.right - px(100), r.top, r.right - px(14), r.bottom};
  text(dc, row.key, key, muted, labelFont, DT_RIGHT);
}
LRESULT CALLBACK proc(HWND h, UINT m, WPARAM w, LPARAM l) {
  switch (m) {
  case WM_APP + 31:
    if (pending >= 0 && (int)w == pending && pending == current) {
      presented = current;
      pending = -1;
      KillTimer(h, 77);
      layout();
    }
    return 0;
  case WM_APP + 32:
    if (IsWindow((HWND)l))
      DestroyWindow((HWND)l);
    return 0;
  case WM_TIMER:
    if (w == 77) {
      PostMessageW(h, WM_APP + 31, pending, 0);
      KillTimer(h, 77);
      return 0;
    }
    break;
  case WM_SIZE:
    layout();
    return 0;
  case WM_DPICHANGED: {
    auto r = (RECT *)l;
    SetWindowPos(h, nullptr, r->left, r->top, r->right - r->left,
                 r->bottom - r->top, SWP_NOZORDER);
    setFonts();
    layout();
    return 0;
  }
  case WM_GETMINMAXINFO: {
    auto v = (MINMAXINFO *)l;
    v->ptMinTrackSize = {px(610), px(350)};
    return 0;
  }
  case WM_CLOSE:
    browser_action(CloseAllTabs, 0, nullptr);
    return 0;
  case WM_COMMAND: {
    int id = LOWORD(w);
    if (id >= 100 && HIWORD(w) == BN_CLICKED) {
      for (auto &t : tabs)
        if (t.button == (HWND)l) {
          browser_action(SelectTab, t.id, nullptr);
          break;
        }
      return 0;
    }
    if (id == Address)
      return 0;
    if (id == More || id == AllTabs) {
      popup(id);
      return 0;
    }
    if (id == FocusAddress) {
      SetFocus(edit);
      SendMessageW(edit, EM_SETSEL, 0, -1);
      return 0;
    }
    if (id > 0 && id <= FindNext) {
      browser_action(id, 0, nullptr);
      return 0;
    }
    break;
  }
  case WM_SETCURSOR: {
    POINT p;
    GetCursorPos(&p);
    ScreenToClient(h, &p);
    if (left && abs(p.x - px(gutter)) <= px(5)) {
      SetCursor(LoadCursorW(nullptr, IDC_SIZEWE));
      return TRUE;
    }
    break;
  }
  case WM_LBUTTONDOWN:
    if (left && abs(GET_X_LPARAM(l) - px(gutter)) <= px(5)) {
      resizing = true;
      SetCapture(h);
      return 0;
    }
    break;
  case WM_MOUSEMOVE:
    if (resizing) {
      gutter =
          std::clamp(MulDiv(GET_X_LPARAM(l), 96, GetDpiForWindow(h)), 92, 350);
      layout();
      return 0;
    }
    break;
  case WM_LBUTTONUP:
    if (resizing) {
      resizing = false;
      ReleaseCapture();
      browser_action(ResizeStrip, gutter, nullptr);
      return 0;
    }
    break;
  case WM_CAPTURECHANGED:
    resizing = false;
    break;
  case WM_ERASEBKGND:
    return 1;
  case WM_PAINT: {
    PAINTSTRUCT ps;
    auto dc = BeginPaint(h, &ps);
    RECT r;
    GetClientRect(h, &r);
    colorRect(dc, r, paper);
    RECT tb{0, 0, left ? std::min(px(gutter), int(r.right) / 2) : r.right,
            left ? r.bottom : px(47)};
    colorRect(dc, tb, band);
    RECT field{addressX, addressY, addressX + addressW, addressY + px(35)};
    rounded(dc, field, active, GetFocus() == edit ? accent : line, 9);
    EndPaint(h, &ps);
    return 0;
  }
  case WM_CTLCOLORSTATIC:
    SetDCBrushColor((HDC)w, RGB(16, 17, 19));
    return (LRESULT)GetStockObject(DC_BRUSH);
  case WM_CTLCOLOREDIT:
    SetBkColor((HDC)w, active);
    SetTextColor((HDC)w, ink);
    SetDCBrushColor((HDC)w, active);
    return (LRESULT)GetStockObject(DC_BRUSH);
  case WM_MEASUREITEM: {
    auto m = (MEASUREITEMSTRUCT *)l;
    if (m->CtlType == ODT_MENU) {
      auto row = (MenuRow *)m->itemData;
      m->itemHeight = px(row && row->id ? 33 : 9);
      m->itemWidth = px(265);
      return TRUE;
    }
    break;
  }
  case WM_DRAWITEM: {
    if (((DRAWITEMSTRUCT *)l)->CtlType == ODT_MENU) {
      drawMenu((DRAWITEMSTRUCT *)l);
      return TRUE;
    }
    auto d = (DRAWITEMSTRUCT *)l;
    int id = d->CtlID;
    RECT r = d->rcItem;
    HDC dc = d->hDC;
    bool tab = id >= 100,
         strip = tab || id == NewTab || id == Placement || id == AllTabs;
    colorRect(dc, r, id == CloseTab ? active : (strip ? band : paper));
    if (tab) {
      auto it = std::find_if(tabs.begin(), tabs.end(),
                             [&](auto &t) { return t.button == d->hwndItem; });
      if (it == tabs.end())
        return TRUE;
      bool on = it->id == current;
      if (on)
        rounded(dc, r, active, line, 10);
      icon(dc, 0, px(19), r.bottom / 2, on ? ink : muted);
      RECT label{px(35), 0, r.right - px(on ? 34 : 6), r.bottom};
      text(dc, it->title, label, on ? ink : muted, labelFont);
    } else if (id == Protection) {
      if (hovered == d->hwndItem)
        rounded(dc, r, active, active, 10);
      icon(dc, id, px(18), r.bottom / 2, blocking ? RGB(167, 197, 174) : muted);
      if (r.right > px(45)) {
        RECT label{px(33), 0, r.right - px(3), r.bottom};
        text(dc,
             activeUrl == L"browser://settings" ? L"Local page"
             : lab                              ? L"Extension lab"
             : blocking                         ? L"Blocking on"
                                                : L"Paused",
             label, ink, labelFont);
      }
    } else {
      if (hovered == d->hwndItem || (d->itemState & ODS_SELECTED))
        rounded(dc, r, RGB(48, 50, 54), RGB(48, 50, 54), 8);
      icon(dc, id, r.right / 2, r.bottom / 2,
           (d->itemState & ODS_DISABLED) ? line : muted);
    }
    if ((d->itemState & ODS_FOCUS) && keyboardFocus) {
      InflateRect(&r, -2, -2);
      auto pen = CreatePen(PS_SOLID, px(2), accent);
      auto old = SelectObject(dc, pen);
      auto b = SelectObject(dc, GetStockObject(NULL_BRUSH));
      RoundRect(dc, r.left, r.top, r.right, r.bottom, px(8), px(8));
      SelectObject(dc, b);
      SelectObject(dc, old);
      DeleteObject(pen);
    }
    return TRUE;
  }
  }
  return DefWindowProcW(h, m, w, l);
}
} // namespace
#include "dialogs.inl"
#include "settings.inl"
extern "C" HWND shell_open(bool vertical, int width, bool priv,
                           bool extension) {
  left = vertical;
  gutter = std::clamp(width, 92, 350);
  privateMode = priv;
  lab = extension;
  auto module = GetModuleHandleW(nullptr);
  WNDCLASSW wc{};
  wc.hInstance = module;
  wc.lpfnWndProc = proc;
  wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
  wc.lpszClassName = L"DesktopBrowserFrame";
  RegisterClassW(&wc);
  window = CreateWindowExW(WS_EX_DLGMODALFRAME, wc.lpszClassName,
                           priv ? L"Private browsing" : L"Desktop Browser",
                           WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN, CW_USEDEFAULT,
                           CW_USEDEFAULT, 1120, 780, nullptr, nullptr, module,
                           nullptr);
  if (!window)
    return nullptr;
  // CEF posts WM_CLOSE to the root ancestor when a hosted Chrome window
  // finishes tearing down. Retired pages get their own root for that message.
  wc.lpszClassName = L"DesktopBrowserRetirementRoot";
  wc.lpfnWndProc = [](HWND h, UINT msg, WPARAM w, LPARAM l) -> LRESULT {
    return msg == WM_CLOSE ? 0 : DefWindowProcW(h, msg, w, l);
  };
  RegisterClassW(&wc);
  retiringPages = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                                  wc.lpszClassName, L"", WS_POPUP, 0, 0, 1, 1,
                                  nullptr, nullptr, module, nullptr);
  if (!retiringPages) {
    DestroyWindow(window);
    window = nullptr;
    return nullptr;
  }
  BOOL dark = TRUE;
  DwmSetWindowAttribute(window, 20, &dark, sizeof(dark));
  COLORREF caption = band;
  DwmSetWindowAttribute(window, 35, &caption, sizeof(caption));
  DwmSetWindowAttribute(window, 36, &ink, sizeof(ink));
  CoCreateInstance(CLSID_AccPropServices, nullptr, CLSCTX_INPROC_SERVER,
                   IID_PPV_ARGS(&accessibility));
  buttons[Back] = button(Back, L"Back");
  buttons[Forward] = button(Forward, L"Forward");
  buttons[Reload] = button(Reload, L"Reload");
  buttons[CloseTab] = button(CloseTab, L"Close active tab");
  buttons[NewTab] = button(NewTab, L"New tab");
  buttons[Placement] = button(Placement, L"Change tab placement");
  buttons[AllTabs] = button(AllTabs, L"All tabs");
  buttons[More] = button(More, L"Browser menu");
  buttons[Protection] = button(Protection, L"Site protection");
  edit = CreateWindowExW(0, L"EDIT", nullptr,
                         WS_VISIBLE | WS_CHILD | WS_TABSTOP | ES_AUTOHSCROLL, 0,
                         0, 1, 1, window, (HMENU)Address, module, nullptr);
  if (accessibility)
    accessibility->SetHwndPropStr(edit, OBJID_CLIENT, CHILDID_SELF,
                                  PROPID_ACC_NAME, L"Address and search");
  SetWindowSubclass(edit, controlProc, 1, 0);
  SendMessageW(edit, EM_SETLIMITTEXT, 8192, 0);
  SendMessageW(edit, EM_SETCUEBANNER, TRUE,
               (LPARAM)L"Search or enter an address");
  viewport = CreateWindowExW(0, L"STATIC", nullptr,
                             WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN, 0, 0, 1,
                             1, window, nullptr, module, nullptr);
  setFonts();
  layout();
  ShowWindow(window, SW_SHOW);
  return viewport;
}
extern "C" HWND shell_prepare_page() {
  RECT r{};
  GetClientRect(viewport, &r);
  return CreateWindowExW(
      0, L"STATIC", nullptr, WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS, 0, 0,
      r.right, r.bottom, viewport, nullptr, GetModuleHandleW(nullptr), nullptr);
}
extern "C" void shell_discard_stage(HWND stage) { DestroyWindow(stage); }
extern "C" void shell_ready(int id) {
  if (id == pending)
    PostMessageW(window, WM_APP + 31, id, 0);
}
extern "C" void shell_add(int id, HWND page, bool localHome) {
  if (localHome && presented != -1) {
    pending = id;
    SetTimer(window, 77, 220, nullptr);
  } else {
    presented = id;
    pending = -1;
  }
  tabs.push_back({id, button(100, L"New tab"), page, L"New tab"});
  current = id;
  scrollIndex = (int)tabs.size() - 1;
  layout();
}
extern "C" void shell_remove(int id) {
  auto it = std::find_if(tabs.begin(), tabs.end(),
                         [&](auto &t) { return t.id == id; });
  if (it == tabs.end())
    return;
  ShowWindow(it->page, SW_HIDE);
  SetParent(it->page, retiringPages);
  PostMessageW(window, WM_APP + 32, 0, (LPARAM)it->page);
  DestroyWindow(it->button);
  tabs.erase(it);
  layout();
}
extern "C" void shell_select(int id) {
  presented = id;
  pending = -1;
  KillTimer(window, 77);
  activeUrl.assign(1, L'\1');
  current = id;
  auto it = std::find_if(tabs.begin(), tabs.end(),
                         [&](auto &t) { return t.id == id; });
  if (it != tabs.end() && !IsWindowVisible(it->button))
    scrollIndex = (int)(it - tabs.begin());
  layout();
  for (auto &t : tabs)
    InvalidateRect(t.button, nullptr, TRUE);
}
extern "C" void shell_title(int id, const wchar_t *title) {
  for (auto &t : tabs)
    if (t.id == id) {
      t.title = *title ? title : L"New tab";
      SetWindowTextW(t.button, t.title.c_str());
      InvalidateRect(t.button, nullptr, FALSE);
    }
}
extern "C" void shell_status(const wchar_t *url, bool back, bool forward,
                             bool busy, bool enabled) {
  if (!window)
    return;
  if (activeUrl != url) {
    activeUrl = url;
    SetWindowTextW(edit, url);
  }
  loading = busy;
  blocking = enabled;
  EnableWindow(buttons[Protection], activeUrl != L"browser://settings");
  EnableWindow(buttons[Back], back);
  EnableWindow(buttons[Forward], forward);
  SetWindowTextW(buttons[Reload], busy ? L"Stop" : L"Reload");
  SetWindowTextW(buttons[Protection], activeUrl == L"browser://settings"
                                          ? L"Local settings page"
                                      : lab     ? L"Extension lab"
                                      : enabled ? L"Pause site blocking"
                                                : L"Resume site blocking");
  InvalidateRect(buttons[Reload], nullptr, FALSE);
  InvalidateRect(buttons[Protection], nullptr, FALSE);
}
extern "C" void shell_layout(bool vertical) {
  left = vertical;
  layout();
}
extern "C" void shell_post(int command) {
  if (window)
    PostMessageW(window, WM_COMMAND, command, 0);
}
extern "C" void shell_destroy() {
  if (!window)
    return;
  DestroyWindow(window);
  DeleteObject(symbolFont);
  DeleteObject(smallSymbol);
  DestroyWindow(retiringPages);
  retiringPages = nullptr;
  if (accessibility) {
    accessibility->Release();
    accessibility = nullptr;
  }
  window = nullptr;
  edit = nullptr;
  if (font)
    DeleteObject(font);
  if (labelFont)
    DeleteObject(labelFont);
}
