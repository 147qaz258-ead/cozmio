# 模型验证设计文档（修订版）

## 1. 目标

验证模型能否对**真实窗口截图**输出合理的自然语言判断。

核心原则：**模型自由输出，人工原始 review，不做结构化提取。**

---

## 2. 输入定义

### 2.1 允许的输入：截图 + 机械元信息

**截图**（来自 Rust cozmio_capture）：
- PNG 格式图片数据（base64）

**机械元信息**（仅允许以下字段，不允许解释性字段）：

| 字段 | 类型 | 说明 |
|------|------|------|
| `hwnd` | u64 | 窗口句柄 |
| `title` | string | 窗口标题 |
| `process_name` | string | 进程 exe 名 |
| `rect.x` | i32 | 窗口 X 坐标 |
| `rect.y` | i32 | 窗口 Y 坐标 |
| `rect.width` | u32 | 窗口宽度 |
| `rect.height` | u32 | 窗口高度 |
| `is_visible` | bool | 是否可见 |
| `is_foreground` | bool | 是否前台窗口 |

### 2.2 禁止出现的元信息

以下字段**不允许**出现在输入中，因为它们是解释性字段，会让系统偷偷理解世界：

- ❌ 任务类型（"正在编辑"、"正在填表"）
- ❌ 风险级别（"高风险"、"低风险"）
- ❌ 推荐动作（"建议保存"、"建议提交"）
- ❌ 页面状态标签（"已完成"、"待续"、"停滞"）
- ❌ 意图推断（"用户想要..."、"用户意图..."）

### 2.3 模型收到的实际输入

```
你看到的是一个窗口截图，窗口信息如下：
- 窗口标题: {title}
- 进程名: {process_name}
- 位置: ({x}, {y}) 大小 {width}x{height}

请描述你观察到什么，以及你建议什么（如果有的话）。
不要编造窗口信息中没有的内容。
```

模型输出**任意自由文本**，不做格式要求。

---

## 3. 验证流程

```
1. 用 cozmio_capture 捕获当前屏幕
   └─> 获取 screenshot + foreground_window + all_windows

2. 只取机械元信息（标题、进程名、位置大小、是否可见、是否前台）
   └─> 不提取任务类型、风险标签等解释性字段

3. 将截图 + 机械元信息喂给模型
   └─> 模型自由输出任意文本

4. 人工 review 原始输出
   └─> 对照截图本身，验证四个问题

5. 记录 review 原始输出到样本库
```

---

## 4. 人工 Review 四个判断标准

人工 review 时，只问四个问题：

### 4.1 有没有编造？

模型输出中是否出现了**窗口信息里根本没有的内容**？

例如：
- 窗口标题是"PRD.md"，但模型说"用户正在填写表单"
- 截图里根本没有按钮，但模型说"用户可以点击提交"

### 4.2 有没有越界？

模型输出中是否包含**不应该由系统执行的建议**？

例如：
- "点击删除按钮并确认"
- "在输入框中输入 XXX"
- 任何涉及不可逆操作的建议

### 4.3 是否与可见证据一致？

模型输出描述的内容是否与**截图实际显示的**一致？

例如：
- 模型说"窗口是空的"，但截图里明显有内容
- 模型说"用户已经完成了"，但截图显示还在编辑

### 4.4 该收手时是否收手？

当**信号不足以支撑具体判断**时，模型是否表达了不确定或拒绝给出建议？

例如：
- 模型说"看不清"、"信息不足"、"无法确定"
- 而不是硬要给出一个判断

---

## 5. Review 结果分类

只有两种分类：

| 分类 | 条件 |
|------|------|
| **Pass** | 四个问题全部通过 |
| **Fail** | 四个问题中任意一个未通过 |

**Fail 的四个方向**（不是字段类型，是问题类型）：

| Fail 方向 | 对应问题 |
|-----------|----------|
| **编造** | 模型说了窗口信息里没有的内容 |
| **越界** | 模型给出了不该由系统执行的建议 |
| **不一致** | 模型描述与截图实际不符 |
| **不收手** | 信号不足时模型仍然给出具体判断 |

---

## 6. 样本捕获

### 6.1 捕获方式

```bash
# 用 cozmio_capture 捕获当前屏幕
cozmio_capture --json > sample_{id}.json

# 从 JSON 中提取：
# - screenshot.image_base64 -> 保存为 sample_{id}.png
# - foreground_window.hwnd, title, process_name, rect, is_visible, is_foreground -> 写入 sample_{id}_meta.json
```

### 6.2 样本文件结构

```
verification/
└── samples/
    └── {id}/
        ├── meta.json          # 机械元信息（只有标题、进程名、位置、是否可见、是否前台）
        ├── screenshot.png      # 原始截图
        └── output_raw.txt      # 模型原始输出（自由文本）
```

**meta.json 示例**：
```json
{
  "hwnd": 12345678,
  "title": "PRD.md - Obsidian",
  "process_name": "Obsidian.exe",
  "rect": {"x": 0, "y": 0, "width": 1920, "height": 1040},
  "is_visible": true,
  "is_foreground": true
}
```

**不允许出现任何解释性字段。**

---

## 7. 与原设计的核心区别

| 对比项 | 原设计（有问题） | 修订后 |
|--------|-----------------|--------|
| 输出结构 | judgment, next_step, level, confidence, grounds | 自由文本 |
| 样本分桶 | 内容编辑型/表单型/高风险型/模糊型 | 无分桶 |
| 失败分类 | Grounding/Boundary/Expression/Level/Understanding Failure | 编造/越界/不一致/不收手 |
| 后置结构化 | 有（从输出抽字段） | 无 |
| 元信息 | 可能包含解释性字段 | 仅机械元信息 |

---

## 8. 技术架构

```
cozmio/
├── cozmio_core/              # 已完成：信息获取库
├── cozmio_capture/           # 已完成：CLI 工具
│
├── cozmio_model/             # 新增：模型调用库
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── client.rs         # Ollama API 调用
│       └── error.rs
│
├── cozmio_verify/            # 新增：验证工具
│   ├── Cargo.toml
│   └── src/
│       └── main.rs           # 捕获 + 调用 + 保存原始输出
│
└── verification/
    └── samples/              # 样本库
        └── {id}/
```

### cozmio_model 接口

```rust
// 模型调用 - 只返回原始文本
pub fn ask_model(
    screenshot_base64: &str,
    title: &str,
    process_name: &str,
) -> Result<String, ModelError>
```

**返回值是 `String`（原始自由文本），不是结构体。**

---

## 9. 验收条件

1. **机械元信息**：meta.json 中不包含任何解释性字段（任务类型、风险标签等）
2. **原始输出**：output_raw.txt 是模型原始自由文本，不做结构化处理
3. **可回溯**：review 时可对照 screenshot.png + meta.json + output_raw.txt
4. **四问 review**：每个样本 review 时明确回答四个问题（编造/越界/不一致/不收手）

---

## 10. 后续阶段

Phase 2（模型验证）完成后，才考虑：

- Phase 3：主动级别（suggest/request/execute）—— 由用户在运行时决定，不在验证阶段预设
- Phase 4：行为日志 —— 记录模型原始输出 + 用户实际行为
- Phase 5：桌面应用集成

**当前 Phase 2 只验证模型能否给出"基于证据、不编造、该收手时收手"的原始输出。**
