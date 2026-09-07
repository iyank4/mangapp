#!/bin/sh

# Mac App Store TUI. Dapat dijalankan dari bash maupun zsh.
# Data vendor diambil dari field `By` pada `mas info <id>`.
export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

if ! command -v mas >/dev/null 2>&1; then
  printf '%s\n' '⚠️  mas tidak ditemukan. Install dengan: brew install mas' >&2
  exit 0
fi

if ! command -v python3 >/dev/null 2>&1; then
  printf '%s\n' '⚠️  python3 tidak ditemukan.' >&2
  exit 1
fi

exec python3 -c "$(cat <<'PY'
import curses
import locale
import os
import re
import subprocess
import sys
import unicodedata
from dataclasses import dataclass


HEADERS = ("VENDOR", "NAMA APLIKASI", "VERSI", "ID APLIKASI")
MAS_COMMAND = os.environ.get("MAS_BIN", "mas")
LIST_PATTERN = re.compile(r"^\s*(\d+)\s+(.+)\s+\(([^()]*)\)\s*$")
TABLE_TOP = 3


@dataclass(frozen=True)
class App:
    vendor: str
    name: str
    version: str
    app_id: str


def run_command(args, timeout):
    try:
        return subprocess.run(
            args,
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None


def get_vendor(app_id):
    result = run_command([MAS_COMMAND, "info", app_id], timeout=10)
    if result is None:
        return "-"

    for line in result.stdout.splitlines():
        if not line.startswith("By "):
            continue
        vendor = re.sub(r"^By\s+", "", line).strip()
        vendor = re.sub(r"^▁+\s*", "", vendor).strip()
        return vendor or "-"
    return "-"


def fetch_apps():
    result = run_command([MAS_COMMAND, "list"], timeout=30)
    if result is None:
        raise RuntimeError("gagal menjalankan mas list atau proses timeout")
    if result.returncode != 0:
        detail = result.stderr.strip() or "tidak ada detail error"
        raise RuntimeError(f"mas list gagal: {detail}")

    apps = []
    for line in result.stdout.splitlines():
        match = LIST_PATTERN.match(line)
        if match is None:
            continue
        app_id, name, version = match.groups()
        apps.append(App(get_vendor(app_id), name.rstrip(), version.strip(), app_id))

    apps.sort(key=lambda app: (app.vendor.casefold(), app.name.casefold()))
    return apps


def print_plain(apps):
    print("\t".join(HEADERS))
    for app in apps:
        print("\t".join((app.vendor, app.name, app.version, app.app_id)))


def display_width(value):
    width = 0
    for char in value:
        if unicodedata.combining(char):
            continue
        width += 2 if unicodedata.east_asian_width(char) in "WFA" else 1
    return width


def clip(value, width):
    if width <= 0:
        return ""
    if display_width(value) <= width:
        return value

    result = []
    used = 0
    for char in value:
        char_width = 0 if unicodedata.combining(char) else (
            2 if unicodedata.east_asian_width(char) in "WFA" else 1
        )
        if used + char_width > max(0, width - 1):
            break
        result.append(char)
        used += char_width
    return "".join(result) + "…"


def column_widths(apps, terminal_width):
    values = [HEADERS] + [
        (app.vendor, app.name, app.version, app.app_id) for app in apps
    ]
    natural = [max(display_width(row[index]) for row in values) for index in range(4)]
    caps = (32, 36, 16, 12)
    minimum = [8, 12, 5, 10]
    widths = [min(natural[index], caps[index]) for index in range(4)]
    available = max(0, terminal_width - 2)

    while sum(widths) + 3 * (len(widths) - 1) > available:
        candidates = [
            index for index in range(4) if widths[index] > minimum[index]
        ]
        if not candidates:
            break
        index = max(candidates, key=lambda item: widths[item] - minimum[item])
        widths[index] -= 1
    return widths


def safe_add(window, y, x, text, attribute=0):
    height, width = window.getmaxyx()
    if y < 0 or y >= height or x >= width:
        return
    try:
        window.addnstr(y, max(0, x), text, max(0, width - max(0, x) - 1), attribute)
    except curses.error:
        pass


def change_selection(selected, delta, total):
    if total == 0:
        return 0
    return max(0, min(total - 1, selected + delta))


def draw(window, apps, selected, top):
    height, width = window.getmaxyx()
    window.erase()

    if height < 8 or width < 44:
        safe_add(window, 0, 0, "Mac App Store")
        safe_add(window, 2, 2, "Perbesar terminal minimal ke 44x8.", curses.A_BOLD)
        safe_add(window, height - 1, 0, "q/Esc keluar")
        window.refresh()
        return 1

    title = " Mac App Store Applications — sorted: vendor, name "
    safe_add(window, 0, 0, title.ljust(width), curses.A_REVERSE | curses.A_BOLD)
    safe_add(window, 1, 1, f"{len(apps)} aplikasi dari Mac App Store")

    widths = column_widths(apps, width)
    table_bottom = height - 3
    visible = max(1, table_bottom - TABLE_TOP)
    row_format = "   ".join("{:<%d}" % column_width for column_width in widths)

    header = row_format.format(*(clip(HEADERS[index], widths[index]) for index in range(4)))
    safe_add(window, TABLE_TOP, 1, header, curses.A_BOLD | curses.A_UNDERLINE)

    for offset, app_index in enumerate(range(top, min(len(apps), top + visible))):
        app = apps[app_index]
        row = (app.vendor, app.name, app.version, app.app_id)
        line = row_format.format(*(clip(row[index], widths[index]) for index in range(4)))
        attribute = curses.A_REVERSE if app_index == selected else 0
        safe_add(window, TABLE_TOP + 1 + offset, 1, line, attribute)

    status = f" {selected + 1 if apps else 0}/{len(apps)} "
    footer_y = height - 2
    safe_add(window, footer_y, 0, "─" * max(1, width - 1), curses.A_DIM)
    safe_add(window, footer_y, 1, status, curses.A_BOLD)
    help_text = "↑/↓ j/k pilih | PgUp/PgDn halaman | Home/End | r refresh | q/Esc keluar"
    safe_add(window, height - 1, 1, help_text, curses.A_DIM)
    window.refresh()
    return visible


def interactive(window, apps):
    try:
        locale.setlocale(locale.LC_ALL, "")
    except locale.Error:
        pass

    window.keypad(True)
    window.timeout(-1)
    try:
        curses.curs_set(0)
    except curses.error:
        pass
    try:
        curses.mousemask(curses.ALL_MOUSE_EVENTS)
    except curses.error:
        pass

    selected = 0
    top = 0
    message = ""

    while True:
        visible = draw(window, apps, selected, top)
        if message:
            safe_add(window, 2, 1, message, curses.A_BOLD)
            window.refresh()
            message = ""

        key = window.getch()
        if key in (ord("q"), ord("Q"), 27, 3):
            return
        if key == curses.KEY_RESIZE:
            continue
        if key in (curses.KEY_DOWN, ord("j")):
            selected = change_selection(selected, 1, len(apps))
        elif key in (curses.KEY_UP, ord("k")):
            selected = change_selection(selected, -1, len(apps))
        elif key == curses.KEY_NPAGE:
            selected = change_selection(selected, visible, len(apps))
        elif key == curses.KEY_PPAGE:
            selected = change_selection(selected, -visible, len(apps))
        elif key == curses.KEY_HOME:
            selected = 0
        elif key == curses.KEY_END:
            selected = max(0, len(apps) - 1)
        elif key in (ord("r"), ord("R")):
            try:
                apps = fetch_apps()
                selected = min(selected, max(0, len(apps) - 1))
                top = 0
                message = "Data diperbarui."
            except RuntimeError as error:
                message = str(error)
        elif key == curses.KEY_MOUSE:
            try:
                _, mouse_y, _, _, button_state = curses.getmouse()
                button4 = getattr(curses, "BUTTON4_PRESSED", 0)
                button5 = getattr(curses, "BUTTON5_PRESSED", 0)
                if button4 and button_state & button4:
                    selected = change_selection(selected, -1, len(apps))
                elif button5 and button_state & button5:
                    selected = change_selection(selected, 1, len(apps))
                elif TABLE_TOP + 1 <= mouse_y < TABLE_TOP + 1 + visible:
                    selected = min(len(apps) - 1, top + mouse_y - TABLE_TOP - 1)
            except curses.error:
                pass

        if selected < top:
            top = selected
        elif selected >= top + visible:
            top = selected - visible + 1


def main(arguments):
    try:
        locale.setlocale(locale.LC_ALL, "")
    except locale.Error:
        pass

    try:
        apps = fetch_apps()
    except RuntimeError as error:
        print(f"⚠️  {error}", file=sys.stderr)
        return 1

    plain_requested = "--plain" in arguments
    terminal = os.environ.get("TERM", "")
    interactive_terminal = sys.stdin.isatty() and sys.stdout.isatty()
    if plain_requested or not interactive_terminal or terminal in ("", "dumb", "unknown"):
        print_plain(apps)
        return 0

    try:
        curses.wrapper(lambda window: interactive(window, apps))
    except curses.error as error:
        print(f"⚠️  Terminal curses tidak tersedia: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
PY
)" "$@"
