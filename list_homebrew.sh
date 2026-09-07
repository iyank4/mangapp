#!/bin/zsh

# Menampilkan instalasi Homebrew sebagai tabel interaktif curses.
# Gunakan --plain untuk output teks yang cocok untuk pipe/log.
export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

if ! command -v brew >/dev/null 2>&1; then
  print "⚠️  Homebrew tidak ditemukan."
  exit 0
fi

if ! command -v python3 >/dev/null 2>&1; then
  print "⚠️  python3 tidak ditemukan; diperlukan untuk tampilan curses."
  exit 1
fi

typeset -a casks formulas
typeset tmp_dir cask_db formula_db mode

tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/homebrew-list.XXXXXX") || exit 1
trap 'rm -rf "$tmp_dir"' EXIT

cask_db="$tmp_dir/casks.json"
formula_db="$tmp_dir/formulae.json"
mode="${1:---curses}"

casks=( ${(f)"$(brew list --cask 2>/dev/null)"} )
formulas=( ${(f)"$(brew list --formula 2>/dev/null)"} )

if (( ${#casks[@]} > 0 )); then
  brew info --json=v2 --cask "${casks[@]}" > "$cask_db" 2>/dev/null || print -r -- '{"casks":[]}' > "$cask_db"
else
  print -r -- '{"casks":[]}' > "$cask_db"
fi

if (( ${#formulas[@]} > 0 )); then
  brew info --json=v2 --formula "${formulas[@]}" > "$formula_db" 2>/dev/null || print -r -- '{"formulae":[]}' > "$formula_db"
else
  print -r -- '{"formulae":[]}' > "$formula_db"
fi

python3 /dev/fd/3 "$cask_db" "$formula_db" "$mode" 3<<'PY'
import curses
import json
import shutil
import sys
import textwrap


def load_records(path, key):
    try:
        with open(path, encoding="utf-8") as stream:
            return json.load(stream).get(key, [])
    except (OSError, json.JSONDecodeError):
        return []


def as_lines(value):
    if isinstance(value, list):
        return [str(item) for item in value if item]
    if value:
        return [str(value)]
    return []


def app_paths(record):
    paths = []
    for artifact in record.get("artifacts", []):
        if isinstance(artifact, dict) and "app" in artifact:
            paths.extend(as_lines(artifact["app"]))
    return paths


def related_formulas(record):
    depends_on = record.get("depends_on", {}) or {}
    return as_lines(depends_on.get("formula"))


def make_rows(cask_records, formula_records):
    rows = []
    related = set()
    applications = []
    non_app_casks = []

    for record in cask_records:
        token = record.get("token", "-")
        dependencies = related_formulas(record)
        related.update(dependencies)
        paths = app_paths(record)
        name = ", ".join(as_lines(record.get("name"))) or token
        installed = record.get("installed") or "tidak diketahui"
        detail = {
            "Homepage": record.get("homepage") or "-",
            "Lokasi aplikasi": paths or ["tidak diketahui"],
            "Formula terkait": dependencies or ["tidak ada dependensi formula langsung"],
        }
        row = {
            "section": "Aplikasi" if paths else "Cask",
            "name": name,
            "package": f"{token} (cask)",
            "version": str(installed),
            "description": record.get("desc") or "-",
            "detail": detail,
        }
        (applications if paths else non_app_casks).append(row)

    rows.append({"section": "Aplikasi yang di-install melalui Homebrew"})
    rows.extend(applications)
    if not applications:
        rows.append({
            "section": None,
            "name": "(tidak ada aplikasi GUI dari Homebrew)",
            "package": "",
            "version": "",
            "description": "",
            "detail": {},
        })

    if non_app_casks:
        rows.append({"section": "Paket cask non-aplikasi"})
        rows.extend(non_app_casks)

    rows.append({"section": "Formula tidak berhubungan dengan aplikasi"})
    unrelated_formulas = []
    for record in formula_records:
        token = record.get("full_name") or record.get("name") or "-"
        if token in related or record.get("name") in related:
            continue
        installed_versions = [item.get("version") for item in record.get("installed", []) if item.get("version")]
        unrelated_formulas.append({
            "section": "Formula",
            "name": record.get("name") or token,
            "package": token,
            "version": ", ".join(installed_versions) or "tidak diketahui",
            "description": record.get("desc") or "-",
            "detail": {
                "Homepage": record.get("homepage") or "-",
                "Versi terpasang": installed_versions or ["tidak diketahui"],
            },
        })
    rows.extend(unrelated_formulas)
    if not unrelated_formulas:
        rows.append({
            "section": None,
            "name": "(tidak ada formula yang tidak terkait)",
            "package": "",
            "version": "",
            "description": "",
            "detail": {},
        })
    return rows


def clip(value, width):
    value = str(value or "")
    if width <= 0:
        return ""
    if len(value) <= width:
        return value.ljust(width)
    if width == 1:
        return "…"
    return (value[: width - 1] + "…").ljust(width)


def column_widths(total_width):
    category = 12
    name = 27
    package = 27
    version = 17
    separators = 8
    description = max(12, total_width - category - name - package - version - separators)
    if category + name + package + version + description + separators > total_width:
        description = 12
        package = max(15, total_width - category - name - version - description - separators)
    return category, name, package, version, description


def table_line(row, width, header=False):
    category, name, package, version, description = column_widths(width)
    if header:
        values = ("Kategori", "Nama / Formula", "Paket", "Versi", "Deskripsi")
    else:
        values = (
            row.get("section", ""),
            row.get("name", ""),
            row.get("package", ""),
            row.get("version", ""),
            row.get("description", ""),
        )
    widths = (category, name, package, version, description)
    return "  ".join(clip(value, column) for value, column in zip(values, widths))[:width]


def detail_lines(row, width):
    lines = []
    for label, values in row.get("detail", {}).items():
        values = values if isinstance(values, list) else [values]
        for index, value in enumerate(values):
            prefix = f"{label}: " if index == 0 else " " * (len(label) + 2)
            lines.extend(textwrap.wrap(prefix + str(value), max(20, width - 2)) or [prefix])
    return lines or ["Pilih baris untuk melihat detail."]


def selectable_indices(rows):
    return [index for index, row in enumerate(rows) if "name" in row]


def draw_screen(stdscr, rows):
    # Some macOS terminals do not support changing cursor visibility.
    try:
        curses.curs_set(0)
    except curses.error:
        pass

    # Decode arrow/Home/End/Page keys instead of returning raw escape bytes.
    try:
        stdscr.keypad(True)
    except curses.error:
        pass

    if curses.has_colors():
        curses.start_color()
        curses.use_default_colors()
        curses.init_pair(1, curses.COLOR_CYAN, -1)
        curses.init_pair(2, curses.COLOR_BLACK, curses.COLOR_CYAN)
        curses.init_pair(3, curses.COLOR_YELLOW, -1)

    selected = selectable_indices(rows)
    selected_pos = 0
    scroll = 0

    while True:
        height, width = stdscr.getmaxyx()
        width = max(40, width - 1)
        detail_height = 6
        table_top = 4
        table_bottom = max(table_top + 2, height - detail_height - 2)
        table_height = table_bottom - table_top
        stdscr.erase()

        title = " Homebrew — aplikasi dan paket "
        title_attr = curses.color_pair(1) | curses.A_BOLD if curses.has_colors() else curses.A_BOLD
        stdscr.addnstr(0, 0, title, width, title_attr)
        stdscr.addnstr(1, 0, "↑/↓ atau j/k: navigasi   PgUp/PgDn: halaman   q/Esc: keluar", width)
        count_attr = curses.color_pair(3) if curses.has_colors() else 0
        stdscr.addnstr(2, 0, f"{len(selected)} item utama  |  terminal {width} kolom", width, count_attr)
        stdscr.addnstr(3, 0, table_line({}, width, header=True), width, curses.A_REVERSE)

        if selected:
            current_row = rows[selected[selected_pos]]
        else:
            current_row = {}

        if selected:
            selected_row_index = selected[selected_pos]
            if selected_row_index < scroll:
                scroll = selected_row_index
            elif selected_row_index >= scroll + table_height:
                scroll = selected_row_index - table_height + 1
        visible_indices = list(range(scroll, min(len(rows), scroll + table_height)))

        for offset, row_index in enumerate(visible_indices):
            row = rows[row_index]
            line = table_line(row, width)
            if "name" not in row:
                attr = curses.color_pair(1) | curses.A_BOLD if curses.has_colors() else curses.A_BOLD
            elif selected and row_index == selected[selected_pos]:
                attr = curses.color_pair(2) if curses.has_colors() else curses.A_REVERSE
            else:
                attr = 0
            stdscr.addnstr(table_top + offset, 0, line, width, attr)

        divider = max(0, height - detail_height - 1)
        stdscr.addnstr(divider, 0, "─" * width, width)
        detail_attr = curses.color_pair(3) if curses.has_colors() else curses.A_BOLD
        stdscr.addnstr(divider + 1, 0, " Detail ", width, detail_attr)
        for offset, line in enumerate(detail_lines(current_row, width)):
            if divider + 2 + offset >= height - 1:
                break
            stdscr.addnstr(divider + 2 + offset, 0, line, width)

        stdscr.refresh()
        key = stdscr.getch()
        if key in (ord("q"), ord("Q"), 27):
            return
        if not selected:
            continue
        if key in (curses.KEY_DOWN, ord("j")):
            selected_pos = min(selected_pos + 1, len(selected) - 1)
        elif key in (curses.KEY_UP, ord("k")):
            selected_pos = max(selected_pos - 1, 0)
        elif key in (curses.KEY_NPAGE, ord(" ")):
            selected_pos = min(selected_pos + max(1, table_height), len(selected) - 1)
        elif key == curses.KEY_PPAGE:
            selected_pos = max(selected_pos - max(1, table_height), 0)
        elif key == curses.KEY_HOME:
            selected_pos = 0
        elif key == curses.KEY_END:
            selected_pos = len(selected) - 1


def print_plain(rows):
    width = max(80, min(180, shutil.get_terminal_size((120, 24)).columns))
    print("Homebrew — aplikasi dan paket")
    print("=" * min(width, 120))
    print(f"{len(selectable_indices(rows))} item utama")
    print(table_line({}, width, header=True))
    print("-" * min(width, 120))
    for row in rows:
        if "name" not in row:
            print(f"\n{row['section']}")
            print("-" * min(width, 120))
        else:
            print(table_line(row, width))


def main():
    casks = load_records(sys.argv[1], "casks")
    formulas = load_records(sys.argv[2], "formulae")
    rows = make_rows(casks, formulas)
    if sys.argv[3] == "--plain":
        print_plain(rows)
    else:
        curses.wrapper(lambda stdscr: draw_screen(stdscr, rows))


if __name__ == "__main__":
    main()
PY
