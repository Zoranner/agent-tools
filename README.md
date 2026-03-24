# Agent Tools

面向文档编辑智能体的工具套件，提供文件系统操作、内容搜索、网络获取、文档分析、版本控制和跨会话记忆能力。

## 接口规范

### 工具定义格式

每个工具使用 JSON Schema 描述，兼容 OpenAI Function Calling 格式，可直接适配主流 LLM 供应商（Anthropic、OpenAI、Google 等）。

```json
{
  "name": "tool_name",
  "description": "工具功能描述",
  "parameters": {
    "type": "object",
    "properties": {
      "param1": {
        "type": "string",
        "description": "参数说明"
      },
      "param2": {
        "type": "number",
        "description": "参数说明"
      }
    },
    "required": ["param1"]
  }
}
```

### 返回值格式

所有工具返回统一的 JSON 结构。

**成功**

```json
{
  "success": true,
  "data": {}
}
```

**失败**

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述"
  }
}
```

### 错误码

| 错误码 | 说明 |
|--------|------|
| `FILE_NOT_FOUND` | 文件或目录不存在 |
| `PERMISSION_DENIED` | 无读写权限 |
| `FILE_ALREADY_EXISTS` | 目标文件已存在 |
| `DIRECTORY_NOT_EMPTY` | 目录非空，无法删除 |
| `PATTERN_NOT_UNIQUE` | `edit_file` 的 `old_text` 在文件中匹配到多处 |
| `PATTERN_NOT_FOUND` | `edit_file` 的 `old_text` 未找到 |
| `INVALID_PATH` | 路径格式不合法 |
| `NETWORK_ERROR` | 网络请求失败 |
| `GIT_ERROR` | Git 操作失败 |
| `MEMORY_KEY_NOT_FOUND` | 记忆条目不存在 |

---

## 工具列表

### 文件系统

#### `read_file`

读取文件内容。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |
| `offset` | `number` | 否 | 起始行号（从 1 开始） |
| `limit` | `number` | 否 | 读取行数 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `content` | `string` | 文件内容 |
| `total_lines` | `number` | 文件总行数 |

---

#### `write_file`

写入文件，文件不存在时自动创建。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |
| `content` | `string` | 是 | 写入内容 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 文件的绝对路径 |

---

#### `edit_file`

精确替换文件中的某段文本，要求 `old_text` 在文件中唯一。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |
| `old_text` | `string` | 是 | 待替换的原始文本 |
| `new_text` | `string` | 是 | 替换后的新文本 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 文件的绝对路径 |

---

#### `create_directory`

创建目录，支持递归创建多级目录。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 目录路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 目录的绝对路径 |

---

#### `list_directory`

列出目录内容。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 目录路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `entries` | `Entry[]` | 条目列表 |
| `entries[].name` | `string` | 文件或目录名 |
| `entries[].type` | `"file" \| "directory"` | 类型 |
| `entries[].size` | `number` | 文件大小（字节），目录为 0 |

---

#### `delete_file`

删除文件。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `string` | 被删除文件的绝对路径 |

---

#### `move_file`

移动或重命名文件。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `source` | `string` | 是 | 源文件路径 |
| `destination` | `string` | 是 | 目标路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `source` | `string` | 源文件的绝对路径 |
| `destination` | `string` | 目标文件的绝对路径 |

---

#### `copy_file`

复制文件。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `source` | `string` | 是 | 源文件路径 |
| `destination` | `string` | 是 | 目标路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `source` | `string` | 源文件的绝对路径 |
| `destination` | `string` | 目标文件的绝对路径 |

---

### 搜索

#### `grep_search`

按关键词或正则表达式搜索文件内容。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `pattern` | `string` | 是 | 搜索关键词或正则表达式 |
| `path` | `string` | 否 | 搜索范围，默认当前目录 |
| `glob` | `string` | 否 | 文件名过滤，如 `**/*.md` |
| `ignore_case` | `boolean` | 否 | 是否忽略大小写，默认 `false` |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `matches` | `Match[]` | 匹配列表 |
| `matches[].file` | `string` | 文件路径 |
| `matches[].line` | `number` | 行号 |
| `matches[].content` | `string` | 匹配行内容 |

---

#### `glob_search`

按文件名模式匹配文件。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `pattern` | `string` | 是 | Glob 模式，如 `**/*.md` |
| `path` | `string` | 否 | 搜索根目录，默认当前目录 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `files` | `string[]` | 匹配的文件路径列表 |

---

### 网络

#### `web_search`

搜索网络，返回相关资料摘要。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | `string` | 是 | 搜索关键词 |
| `limit` | `number` | 否 | 返回结果数量，默认 5 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `results` | `Result[]` | 搜索结果列表 |
| `results[].title` | `string` | 页面标题 |
| `results[].url` | `string` | 页面 URL |
| `results[].snippet` | `string` | 内容摘要 |

---

#### `web_fetch`

抓取指定网页内容并转为 Markdown。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `url` | `string` | 是 | 网页 URL |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `content` | `string` | 转换后的 Markdown 内容 |
| `title` | `string` | 页面标题 |
| `url` | `string` | 实际访问的 URL（含重定向） |

---

### 文档

#### `extract_toc`

提取文档的目录结构。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `toc` | `TocItem[]` | 目录列表 |
| `toc[].level` | `number` | 标题层级（1-6） |
| `toc[].title` | `string` | 标题文本 |
| `toc[].line` | `number` | 所在行号 |

---

#### `count_words`

统计文档字数、段落数、标题数。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 是 | 文件路径 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `characters` | `number` | 字符数（不含空格） |
| `words` | `number` | 词数 |
| `paragraphs` | `number` | 段落数 |
| `headings` | `number` | 标题数 |
| `lines` | `number` | 总行数 |

---

### 版本控制

#### `git_status`

查看工作区文件变更状态。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 否 | 仓库路径，默认当前目录 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `changes` | `Change[]` | 变更列表 |
| `changes[].file` | `string` | 文件路径 |
| `changes[].status` | `"added" \| "modified" \| "deleted" \| "untracked"` | 变更类型 |

---

#### `git_diff`

查看文件修改差异。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 否 | 仓库或文件路径，默认当前目录 |
| `staged` | `boolean` | 否 | 是否查看暂存区差异，默认 `false` |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `diff` | `string` | diff 文本内容 |

---

#### `git_commit`

暂存并提交文档变更。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `message` | `string` | 是 | 提交信息 |
| `files` | `string[]` | 否 | 指定暂存的文件列表，默认暂存全部变更 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `hash` | `string` | 提交的 commit hash |
| `message` | `string` | 提交信息 |

---

#### `git_log`

查看提交历史记录。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | `string` | 否 | 仓库或文件路径，默认当前目录 |
| `limit` | `number` | 否 | 返回条数，默认 10 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `commits` | `Commit[]` | 提交列表 |
| `commits[].hash` | `string` | commit hash |
| `commits[].message` | `string` | 提交信息 |
| `commits[].author` | `string` | 作者 |
| `commits[].date` | `string` | 提交时间（ISO 8601） |

---

### 记忆

#### `memory_write`

存储一条记忆条目。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 记忆标识符 |
| `content` | `string` | 是 | 记忆内容 |
| `tags` | `string[]` | 否 | 标签，用于分类和搜索 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 记忆标识符 |

---

#### `memory_read`

读取指定记忆条目。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `key` | `string` | 是 | 记忆标识符 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 记忆标识符 |
| `content` | `string` | 记忆内容 |
| `tags` | `string[]` | 标签列表 |
| `created_at` | `string` | 创建时间（ISO 8601） |
| `updated_at` | `string` | 更新时间（ISO 8601） |

---

#### `memory_search`

按关键词搜索历史记忆。

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `query` | `string` | 是 | 搜索关键词 |
| `tags` | `string[]` | 否 | 按标签过滤 |
| `limit` | `number` | 否 | 返回数量，默认 10 |

**返回**

| 字段 | 类型 | 说明 |
|------|------|------|
| `results` | `MemoryItem[]` | 匹配的记忆列表 |
| `results[].key` | `string` | 记忆标识符 |
| `results[].content` | `string` | 记忆内容 |
| `results[].tags` | `string[]` | 标签列表 |
