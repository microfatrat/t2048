#!/usr/bin/env python3
"""生成 THIRD-PARTY-LICENSES.md（第三方许可证与版权声明）。

数据来源是仓库根目录的 Cargo.lock，加上本地 Cargo 注册表缓存里各 crate 的
Cargo.toml 与许可证文件。脚本只读取、不修改任何依赖。

用法：
    python3 scripts/gen-third-party-licenses.py            # 写入 THIRD-PARTY-LICENSES.md
    python3 scripts/gen-third-party-licenses.py --check    # 只比较，不写入

注意：要得到**完整**的跨平台清单，需要先执行一次

    cargo fetch --locked

`cargo fetch` 不带 `--target` 时会抓取所有平台（Windows / WASI / Redox / macOS …）
的依赖。若默认注册表缓存目录不可写，可以把 `CARGO_HOME` 指向一个可写目录，
例如 `CARGO_HOME=.cargo-home cargo fetch --locked`（该目录已在 .gitignore 中）。
本机缓存里缺失的 crate 会被列在文件末尾的「未解析组件」一节。
"""

from __future__ import annotations

import argparse
import glob
import os
import re
import sys
import tarfile
from collections import Counter, OrderedDict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LOCKFILE = os.path.join(ROOT, "Cargo.lock")
OUTFILE = os.path.join(ROOT, "THIRD-PARTY-LICENSES.md")

# 本仓库自己的包名，不作为第三方列出。
WORKSPACE_PACKAGES = {"t2048"}

# 直接依赖（与 Cargo.toml 的 [dependencies] 保持一致），单独成节。
DIRECT_DEPENDENCIES = ["crossterm", "rand", "ratatui"]

LICENSE_FILE_RE = re.compile(r"^(licen[cs]e|copying|notice|copyright)", re.IGNORECASE)
# 只认真正的版权声明行：Copyright 后面必须跟 (c)/©/四位年份/大写名称/占位符。
# 这里刻意不用 re.IGNORECASE 匹配整行，否则 [A-Z] 会连小写也算上，
# 把 Apache-2.0 正文里的散文当成版权行，例如
#   "copyright notice that is included in or attached to the work"
#   "copyright license to reproduce, prepare Derivative Works of,"
#   "Copyrights in the Rand project are retained by their contributors."
# 这些都不是需要保留的版权声明，因此只对 "copyright" 这个词本身忽略大小写。
COPYRIGHT_LINE_RE = re.compile(
    r"^(?i:copyright)\s*(?:\(c\)|©|\(C\))?\s*(?:\d{4}|[A-Z]|<)"
)
# 许可证模板里的占位符，不是真实版权人。
COPYRIGHT_PLACEHOLDERS = (
    "<year>",
    "<copyright holders>",
    "<owner>",
    "[yyyy]",
    "<name of copyright owner>",
)

# 许可证全文的识别特征：按顺序取第一个命中的 id。
# 只在许可证文件正文里匹配，避免把 README 里的提及当成许可证。
DETECTORS = [
    ("Apache-2.0", ("Apache License", "Version 2.0, January 2004")),
    ("MPL-2.0", ("Mozilla Public License Version 2.0",)),
    ("CC0-1.0", ("CC0 1.0", "Creative Commons")),
    ("Unicode-3.0", ("UNICODE LICENSE V3",)),
    ("BSL-1.0", ("Boost Software License",)),
    ("bzip2-1.0.6", ("bzip2", "Julian Seward")),
    ("ISC", ("ISC License", "Permission to use, copy, modify, and/or distribute")),
    ("Zlib", ("This software is provided 'as-is'",)),
    ("Unlicense", ("free and unencumbered software released into the public domain",)),
    ("0BSD", ("Zero-Clause BSD", "Permission to use, copy, modify, and/or distribute")),
    ("BSD-3-Clause", ("Redistribution and use in source and binary forms", "Neither the name")),
    ("BSD-2-Clause", ("Redistribution and use in source and binary forms",)),
    ("MIT", ("Permission is hereby granted, free of charge",)),
]

# 按 OR/AND/WITH 以及 Cargo 常见的 "/" 写法拆分 SPDX 表达式。
SPDX_SPLIT_RE = re.compile(r"\s+(?:OR|AND)\s+|/|,", re.IGNORECASE)

# 生成文件里展示许可证全文的顺序（有全文的按此优先级）。
TEXT_ORDER = [
    "MIT",
    "Apache-2.0",
    "ISC",
    "BSD-3-Clause",
    "BSD-2-Clause",
    "Zlib",
    "Unicode-3.0",
    "BSL-1.0",
    "MPL-2.0",
    "bzip2-1.0.6",
    "CC0-1.0",
    "0BSD",
    "Unlicense",
    "MIT-0",
    "LLVM-exception",
]

LICENSE_TITLES = {
    "MIT": "MIT License",
    "Apache-2.0": "Apache License 2.0",
    "ISC": "ISC License",
    "BSD-3-Clause": 'BSD 3-Clause "New" or "Revised" License',
    "BSD-2-Clause": 'BSD 2-Clause "Simplified" License',
    "Zlib": "zlib License",
    "Unicode-3.0": "UNICODE LICENSE V3",
    "BSL-1.0": "Boost Software License 1.0",
    "MPL-2.0": "Mozilla Public License Version 2.0",
    "bzip2-1.0.6": "bzip2 License 1.0.6",
    "CC0-1.0": "Creative Commons Zero v1.0 Universal",
    "0BSD": "Zero-Clause BSD",
    "Unlicense": "The Unlicense",
    "MIT-0": "MIT No Attribution",
    "LLVM-exception": "LLVM Exceptions to the Apache 2.0 License",
    "GPL-2.0-only": "GNU General Public License v2.0",
}


def cargo_home():
    """定位 Cargo 主目录，顺序：环境变量 → 仓库内 .cargo-home → ~/.cargo。

    仓库内的 .cargo-home 用于默认缓存目录不可写时抓取全平台依赖，
    它已在 .gitignore 中；不存在时回退到标准的 ~/.cargo。
    """
    env = os.environ.get("CARGO_HOME")
    if env:
        return env
    local = os.path.join(ROOT, ".cargo-home")
    if os.path.isdir(local):
        return local
    return os.path.join(os.path.expanduser("~"), ".cargo")


def find_registry():
    """返回 (src_dir, cache_dir)，两者都可能不存在。"""
    home = cargo_home()
    src = sorted(glob.glob(os.path.join(home, "registry", "src", "*")))
    cache = sorted(glob.glob(os.path.join(home, "registry", "cache", "*")))
    return (src[0] if src else None, cache[0] if cache else None)


def parse_lock(path):
    """从 Cargo.lock 里取出 (name, version) 列表，保持文件中的顺序。"""
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    pkgs = re.findall(r'\[\[package\]\]\nname = "([^"]+)"\nversion = "([^"]+)"', text)
    if not pkgs:
        raise SystemExit(f"无法从 {path} 解析出任何包")
    return pkgs


def read_manifest(src_dir, cache_dir, name, version):
    """返回 (Cargo.toml 文本, {许可证文件名: 内容})；本地缓存缺失时返回 None。"""
    crate = f"{name}-{version}"
    directory = os.path.join(src_dir, crate) if src_dir else None
    if directory and os.path.isfile(os.path.join(directory, "Cargo.toml")):
        manifest = _read_text(os.path.join(directory, "Cargo.toml"))
        files = {}
        for entry in sorted(os.listdir(directory)):
            if LICENSE_FILE_RE.match(entry) and os.path.isfile(os.path.join(directory, entry)):
                files[entry] = _read_text(os.path.join(directory, entry))
        return manifest, files

    archive = os.path.join(cache_dir or "", f"{crate}.crate")
    if cache_dir and os.path.isfile(archive):
        try:
            with tarfile.open(archive, "r:gz") as tar:
                manifest = None
                files = {}
                for member in tar.getmembers():
                    if not member.isfile():
                        continue
                    base = os.path.basename(member.name)
                    # 只取 crate 根目录下的文件
                    if member.name.count("/") != 1:
                        continue
                    if base == "Cargo.toml":
                        manifest = _read_member(tar, member)
                    elif LICENSE_FILE_RE.match(base):
                        files[base] = _read_member(tar, member)
                if manifest is not None:
                    return manifest, files
        except (tarfile.TarError, OSError):
            pass
    return None


def _read_text(path):
    with open(path, encoding="utf-8", errors="replace") as fh:
        return fh.read()


def _read_member(tar, member):
    handle = tar.extractfile(member)
    if handle is None:
        return ""
    # 统一换行符，和 _read_text 的文本模式读取保持一致：.crate 包里保留的
    # CRLF 会让同一份许可证在「从 src 读」和「从 cache 读」时生成不同结果。
    text = handle.read().decode("utf-8", errors="replace")
    return text.replace("\r\n", "\n").replace("\r", "\n")


def manifest_license(manifest):
    """返回 Cargo.toml 里的 license / license-file 声明。"""
    if manifest is None:
        return None, None
    match = re.search(r'^license\s*=\s*"([^"]+)"', manifest, re.MULTILINE)
    if match:
        return match.group(1), None
    match = re.search(r'^license-file\s*=\s*"([^"]+)"', manifest, re.MULTILINE)
    if match:
        return None, match.group(1)
    return None, None


def spdx_ids(expression):
    """把 SPDX 表达式拆成单个 id 集合（保留 WITH 后面的例外名）。"""
    ids = set()
    for part in SPDX_SPLIT_RE.split(expression):
        part = part.strip().strip("()").strip()
        if not part:
            continue
        if part.upper() in {"OR", "AND", "WITH"}:
            continue
        ids.add(part)
    return ids


def detect_license_id(text):
    for license_id, needles in DETECTORS:
        if all(needle in text for needle in needles):
            return license_id
    return None


def collect_copyright(text, limit=6):
    """从许可证文件里提取需要保留的 Copyright 行。"""
    lines = []
    seen = set()
    for raw in text.splitlines():
        line = raw.strip().rstrip("*/-").strip()
        if not line or len(line) > 300:
            continue
        if not COPYRIGHT_LINE_RE.match(line):
            continue
        # 全大写的是标题（如 "COPYRIGHT AND PERMISSION NOTICE"），不是署名。
        if line.isupper():
            continue
        lowered = line.lower()
        if any(marker in lowered for marker in COPYRIGHT_PLACEHOLDERS):
            continue
        if line in seen:
            continue
        seen.add(line)
        lines.append(line)
        if len(lines) >= limit:
            break
    return lines


def build(lock_entries, src_dir, cache_dir):
    resolved = []      # dict: name, version, license, copyright, files
    unresolved = []
    for name, version in lock_entries:
        if name in WORKSPACE_PACKAGES:
            continue
        loaded = read_manifest(src_dir, cache_dir, name, version)
        if loaded is None:
            unresolved.append((name, version))
            continue
        manifest, files = loaded
        expression, license_file = manifest_license(manifest)
        if expression is None and license_file is None:
            expression = "(未声明)"
        record = {
            "name": name,
            "version": version,
            "expression": expression or f"见 {license_file}",
            "copyright": [],
        }
        for filename in sorted(files):
            record["copyright"].extend(collect_copyright(files[filename]))
        # 去重并保持顺序
        record["copyright"] = list(OrderedDict.fromkeys(record["copyright"]))
        record["files"] = files
        resolved.append(record)
    return resolved, unresolved


def pick_license_texts(resolved):
    """为每个出现的 SPDX id 挑一份许可证全文。"""
    texts = {}
    for record in resolved:
        for filename, content in record["files"].items():
            license_id = detect_license_id(content)
            if license_id and license_id not in texts:
                texts[license_id] = (content, record["name"], record["version"], filename)
    return texts


def md_table(rows, headers):
    out = ["| " + " | ".join(headers) + " |", "|" + "|".join([" --- "] * len(headers)) + "|"]
    for row in rows:
        out.append("| " + " | ".join(cell.replace("|", "\\|") for cell in row) + " |")
    return "\n".join(out)


def render(resolved, unresolved, texts):
    lines = []
    add = lines.append

    add("# 第三方许可证与版权声明")
    add("")
    add("本项目 **t2048**（Cargo 包名 `t2048`）以 MIT 许可证发布，"
        "详见仓库根目录的 [`LICENSE`](LICENSE)。")
    add("")
    add("t2048 编译成**单个可执行文件**分发：crates.io 上的源码包，以及 "
        "[GitHub Releases](https://github.com/microfatrat/t2048/releases) 里的 "
        "Linux / Windows / macOS 预编译二进制。运行时没有额外依赖，但可执行文件里"
        "**静态链接**了下列第三方组件，MIT / Apache-2.0 / BSD 等许可证都要求在再分发时"
        "保留它们的版权声明与许可证全文。")
    add("")
    add("> **本文件由脚本自动生成，请勿手工编辑。**")
    add("> 生成命令：`python3 scripts/gen-third-party-licenses.py`")
    add("> 数据来源：`Cargo.lock` 与本地 Cargo 注册表缓存。")
    total = len(resolved) + len(unresolved) + len(WORKSPACE_PACKAGES)
    coverage = f"> 覆盖范围：`Cargo.lock` 中 {total} 个包，其中 {len(resolved)} 个已解析许可证"
    if unresolved:
        coverage += f"，{len(unresolved)} 个未包含在本机缓存中（见文末「未解析组件」）"
    add(coverage + "。")
    add("")

    section = 0

    def heading(title):
        nonlocal section
        section += 1
        add(f"## {section}. {title}")
        add("")

    heading("本项目自身")
    add("- 许可证：MIT")
    add("- 版权：Copyright (c) 2026 microfatrat")
    add("- 全文见 [`LICENSE`](LICENSE)")
    add("")

    heading("直接依赖")
    by_name = {}
    for record in resolved:
        by_name.setdefault(record["name"], record)
    rows = []
    for name in DIRECT_DEPENDENCIES:
        record = by_name.get(name)
        if record:
            rows.append((f"`{name}`", record["version"], f"`{record['expression']}`"))
        else:
            rows.append((f"`{name}`", "—", "未解析"))
    add(md_table(rows, ["组件", "版本", "许可证"]))
    add("")
    heading("许可证全文")
    add("下列许可证文本按 SPDX 标识符去重，覆盖本次生成时能解析到的全部组件。"
        "当组件采用「A OR B」这类多许可表达式时，本项目选择其中限制最少的一项"
        "（通常是 MIT 或 Apache-2.0）。")
    add("")
    license_section = section
    license_counts = Counter()
    for record in resolved:
        for license_id in spdx_ids(record["expression"]):
            license_counts[license_id] += 1
    if license_counts:
        summary = "、".join(f"`{license_id}`（{count}）"
                            for license_id, count in license_counts.most_common())
        add(f"各组件声明的许可证标识符（按组件数）：{summary}。")
    else:
        add("本次生成没有解析到任何许可证全文。")
    add("")
    ordered = [license_id for license_id in TEXT_ORDER if license_id in texts]
    ordered += sorted(license_id for license_id in texts if license_id not in TEXT_ORDER)
    for index, license_id in enumerate(ordered, start=1):
        content, source_name, source_version, filename = texts[license_id]
        title = LICENSE_TITLES.get(license_id, license_id)
        add(f"### {license_section}.{index} {title}（`{license_id}`）")
        add("")
        add(f"<!-- 取自 {source_name} {source_version} 的 {filename} -->")
        add("")
        add("```text")
        add(content.strip())
        add("```")
        add("")

    heading("全部第三方组件清单")
    add(f"共 {len(resolved)} 个组件。许可证列是该 crate 在 `Cargo.toml` 中声明的 SPDX 表达式原文。")
    add("")
    rows = [(str(i), f"`{r['name']}`", r["version"], f"`{r['expression']}`")
            for i, r in enumerate(sorted(resolved, key=lambda r: (r["name"].lower(), r["version"])), start=1)]
    add(md_table(rows, ["#", "组件", "版本", "许可证"]))
    add("")

    heading("版权声明明细")
    add("以下版权行摘自各组件自带的许可证文件。MIT / BSD / ISC / zlib 等许可证都要求"
        "在分发时保留这些声明。")
    add("")
    for record in sorted(resolved, key=lambda r: (r["name"].lower(), r["version"])):
        if not record["copyright"]:
            continue
        add(f"- **{record['name']} {record['version']}** — " + "；".join(record["copyright"]))
    add("")

    if unresolved:
        heading("未解析组件")
        add("下列 crate 出现在 `Cargo.lock` 中，但本机 Cargo 缓存没有它们的清单文件，"
            "因此无法自动提取许可证与版权声明。它们通常是 Windows / WASI / Redox "
            "平台专属组件（例如 Windows 版 crossterm 依赖 `winapi` 与 `windows-sys`）；"
            "在对应平台上发布之前，请先 `cargo fetch`（不带 `--target`）再重新生成本文件，"
            "以获得完整清单。")
        add("")
        for name, version in sorted(unresolved, key=lambda item: (item[0].lower(), item[1])):
            add(f"- `{name}` {version}")
        add("")

    heading("重新生成")
    add("```bash")
    add("# 拉取所有平台的依赖；不带 --target 时 cargo 会抓取全部目标三元组")
    add("cargo fetch --locked")
    add("# 重新生成")
    add("python3 scripts/gen-third-party-licenses.py")
    add("```")
    add("")
    add("`--check` 只比较结果不写入，CI 用它确认清单没有随 `Cargo.lock` 过期。"
        "生成结果取决于本机 Cargo 缓存里存在哪些平台的 crate，因此 `--check` 要在与生成时"
        "相同的缓存状态下运行（两者都执行过完整的 `cargo fetch --locked`）。"
        "注册表缓存目录不可写时，可把 `CARGO_HOME` 指向一个可写目录再执行上面的命令。")
    add("")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="生成 THIRD-PARTY-LICENSES.md")
    parser.add_argument("--check", action="store_true", help="只比较，不写入文件")
    args = parser.parse_args()

    src_dir, cache_dir = find_registry()
    if not src_dir and not cache_dir:
        raise SystemExit("找不到 Cargo 注册表缓存，请先执行 cargo fetch")

    lock_entries = parse_lock(LOCKFILE)
    resolved, unresolved = build(lock_entries, src_dir, cache_dir)
    texts = pick_license_texts(resolved)
    document = render(resolved, unresolved, texts)
    if not document.endswith("\n"):
        document += "\n"

    if args.check:
        if not os.path.isfile(OUTFILE):
            print("THIRD-PARTY-LICENSES.md 不存在", file=sys.stderr)
            return 1
        with open(OUTFILE, encoding="utf-8") as fh:
            current = fh.read()
        if current != document:
            print("THIRD-PARTY-LICENSES.md 与 Cargo.lock 不同步，请重新生成", file=sys.stderr)
            return 1
        print("THIRD-PARTY-LICENSES.md 是最新的")
        return 0

    with open(OUTFILE, "w", encoding="utf-8") as fh:
        fh.write(document)
    print(f"已写入 {os.path.relpath(OUTFILE, ROOT)}"
          f"（{len(resolved)} 个组件已解析，{len(unresolved)} 个未解析，"
          f"{len(texts)} 种许可证全文）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
