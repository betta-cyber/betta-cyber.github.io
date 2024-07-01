---
title: Python Trick
abbrlink: 9d5f329
date: 2024-06-26 09:00:00
---

## 一、deepcopy 性能提升

### 1. 循环赋值构造新的对象代替 deepcopy
速度也可以。

### 2. 使用 pickle
```python
data = pickle.loads(pickle.dumps(data))
```

### 3. 使用 json 或者 ujson
```python
data = json.loads(json.dumps(my_dict))
```

### 4. 使用 cpickle
```python
import _pickle as cPickle
utxos = cPickle.loads(cPickle.dumps(cache_utxos))
```

## pytest 相关
```python
pytest ./tests/lib -s
```


## 二、开源项目规范问题


### 1. pre-commit

Git hook对于在提交代码审查之前识别简单问题很有用。我们在每次提交时都运行钩子，以自动指出代码中的问题，例如缺少分号，尾随空白和调试语句。通过在代码审阅之前指出这些问题，代码审阅者可以专注于更改的体系结构，而不会浪费琐碎的样式问题。

```yaml
repos:
  - repo: "https://github.com/pre-commit/pre-commit-hooks"
    rev: v2.3.0
    hooks:
      - id: check-docstring-first
      - id: check-merge-conflict
      - id: trailing-whitespace
      # - id: end-of-file-fixer
      - id: check-yaml
      - id: check-ast
  - repo: https://github.com/pycqa/isort
    rev: 5.12.0
    hooks:
      - id: isort
        args: ["--profile=black"]
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.4
    hooks:
      - id: ruff
      - id: ruff-format
```

### 2. 使用 pyproject

下面是我的一个例子: 在配置文件里面还用 `ruff` 进行代码的检测。也可以用来做python的依赖管理。

```toml
[build-system]
requires = ["setuptools", "setuptools-scm"]
build-backend = "setuptools.build_meta"

[project]
name = "atomicals-electrumx"
dynamic = ["version"]
dependencies = [
  "aiorpcX[ws]>=0.23.0",
  "attrs",
  "plyvel",
  "pylru @ git+https://github.com/atomicals-community/pylru@c9b47f0",
  "aiohttp>=3.3,<4",
  "cbor2",
  "websockets",
  "regex",
  "krock32",
  "merkletools",
  "requests==2.31.0",
  "python-dotenv",
]
requires-python = ">=3.10"
authors = [
  {name = "Electrum developers", email = "atomicals@atomicals.com"},
]
description = "Atomicals ElectrumX Server"
readme = "README.rst"
license = {text = "MIT Licence"}
keywords = ["atomicals", "ElectrumX"]
classifiers = [
  "Development Status :: 5 - Production/Stable",
  "Framework :: AsyncIO",
  "License :: OSI Approved :: MIT License",
  "Operating System :: Unix",
  "Programming Language :: Python :: 3.10",
  "Topic :: Database",
  "Topic :: Internet"
]

[project.optional-dependencies]
cli = []

[project.urls]
Homepage = "https://atomicals.xyz/"
Documentation = "https://docs.atomicals.xyz/"
Repository = "https://github.com/atomicals/atomicals-electrumx.git"
"Bug Tracker" = "https://github.com/atomicals/atomicals-electrumx/issues"
Changelog = "https://github.com/atomicals/atomicals-electrumx/releases"


[tool.ruff]
# Exclude a variety of commonly ignored directories.
exclude = [
    ".bzr",
    ".direnv",
    ".eggs",
    ".git",
    ".git-rewrite",
    ".hg",
    ".ipynb_checkpoints",
    ".mypy_cache",
    ".nox",
    ".pants.d",
    ".pyenv",
    ".pytest_cache",
    ".pytype",
    ".ruff_cache",
    ".svn",
    ".tox",
    ".venv",
    ".vscode",
    "__pypackages__",
    "_build",
    "buck-out",
    "build",
    "dist",
    "node_modules",
    "site-packages",
    "venv",
]

# Same as Black.
line-length = 120
indent-width = 4

target-version = "py310"

[tool.ruff.lint]
# Enable Pyflakes (`F`) and a subset of the pycodestyle (`E`)  codes by default.
# Unlike Flake8, Ruff doesn't enable pycodestyle warnings (`W`) or
# McCabe complexity (`C901`) by default.
select = [
    "E4",  # pycodestyle errors
    "E7",
    "E9",
    "F",  # pyflakes
    "I",  # isort
    "B",  # flake8-bugbear
]
ignore = [
  "E712",
  "F401",
  "F405",
  "F841",
  "B008",
  "B904"
]

# Allow fix for all enabled rules (when `--fix`) is provided.
fixable = ["ALL"]
unfixable = []

# Allow unused variables when underscore-prefixed.
dummy-variable-rgx = "^(_+|(_+[a-zA-Z0-9_]*[a-zA-Z0-9]+?))$"

[tool.ruff.lint.per-file-ignores]
"tests/server/test_daemon.py" = ["B015"]
"tests/lib/test_script2addr.py" = ["E711"]
"tests/lib/test_atomicals_utils.py" = ["B017"]

[tool.ruff.format]
# Like Black, use double quotes for strings.
quote-style = "double"

# Like Black, indent with spaces, rather than tabs.
indent-style = "space"

# Like Black, respect magic trailing commas.
skip-magic-trailing-comma = false

# Like Black, automatically detect the appropriate line ending.
line-ending = "auto"
```

### 3. ruff

ruff是一个用rust构建的全新的python代码分析工具。HTTPX 和 Starlette 在同一天将在用的代码分析工具（flake8、autoflake 和 isort）统一替换成了 Ruff。我在之前也用过isort，flake8等。但是都觉得一般般，不太好用。

集成到 github action 中:
```yml
name: Ruff
on: [push, pull_request]
jobs:
  ruff:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: chartboost/ruff-action@v1
```

## 三、代码编写问题

### 1. 使用 `if is`

```python
a = range(10000)
%timeit -n 100 [i for i in a if i == True]
%timeit -n 100 [i for i in a if i is True]
100 loops, best of 3: 531 µs per loop
100 loops, best of 3: 362 µs per loop
```

参考:
https://www.cnblogs.com/changkuk/p/9978511.html
