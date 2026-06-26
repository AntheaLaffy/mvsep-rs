# MVSep API 端点文档（按功能分类）

> 来源: https://mvsep.com/zh/full_api
> 更新时间: 2026-06-27

---

## 📌 核心要点

### 🔑 获取模型信息的 API（动态获取，无需硬编码）

```
GET https://mvsep.com/api/app/algorithms
```

**说明**: 此 API 返回所有可用的分离算法及其参数选项，包括：
- `render_id` - 分离类型 ID（sep_type）
- `name` - 算法名称
- `algorithm_fields` - 附加参数（add_opt1, add_opt2, add_opt3）及其选项
- `price_coefficient` - 积分消耗系数
- `orientation` - 用户权限要求

**查询参数**:
| 字段 | 类型 | 说明 |
|------|------|------|
| scopes | 字符串 | 可选，过滤模型类型：<br>- `single_upload` - 单文件上传（默认）<br>- `no_upload` - 无需上传<br>- `matchering_upload` - 音频匹配 |

**响应示例结构**:
```json
[
  {
    "render_id": 26,
    "name": "Ensemble (vocals, instrum)",
    "algorithm_group": { "name": "人声/伴奏分离" },
    "algorithm_fields": [
      {
        "name": "add_opt1",
        "text": "Output files",
        "options": {"0": "Standard set", "1": "Include intermediate results"},
        "default_key": "0"
      },
      {
        "name": "add_opt2",
        "text": "Model Type",
        "options": {...},
        "default_key": "7"
      }
    ],
    "price_coefficient": 1.0,
    "orientation": 0
  }
]
```

---

## ⚡ 一、操作类 API（执行动作）

### 1. 创建分离任务
**POST** `https://mvsep.com/api/separation/create`

**功能**: 上传音频文件并创建分离任务

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| audiofile | 二进制 | ✅ | 音频文件（MP3, WAV, FLAC, OGG, WEBM, MP4A, AAC）<br>最大: 免费50MB / 付费200MB |
| api_token | 字符串 | ✅ | 用户 API 密钥 |
| sep_type | 整数 | ✅ | 分离类型 ID（从 /api/app/algorithms 获取） |
| add_opt1 | 整数/字符串 | ❌ | 附加选项 1（根据算法不同） |
| add_opt2 | 整数/字符串 | ❌ | 附加选项 2 |
| output_format | 整数 | ❌ | 输出格式：0=MP3, 1=WAV(16bit), 2=WAV(24bit), 3=WAV(32bit float), 4=WAV(32bit), 5=FLAC |
| is_demo | 整数 | ❌ | 是否演示模式：0=否, 1=是 |

**Curl 示例**:
```bash
curl --location --request POST 'https://mvsep.com/api/separation/create' \
  --form 'audiofile=@"/path/to/file.mp3"' \
  --form 'api_token="<您的 API 令牌>"' \
  --form 'sep_type="26"' \
  --form 'add_opt1="0"' \
  --form 'add_opt2="7"' \
  --form 'output_format="1"' \
  --form 'is_demo="0"'
```

**响应**:
```json
{
  "success": true,
  "data": {
    "link": "https://mvsep.com/api/separation/get?hash=xxx",
    "hash": "20230327071601-xxx.mp3"
  }
}
```

**错误代码**:
- `400` - 参数无效或缺失
- `401` - api_token 无效

---

### 2. 取消分离任务
**POST** `https://mvsep.com/api/separation/cancel`

**功能**: 取消未开始处理的任务并退回积分

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| api_token | 字符串 | ✅ | 用户 API 密钥 |
| hash | 字符串 | ✅ | 任务哈希值（创建任务时返回） |

**Curl 示例**:
```bash
curl --location --request POST 'https://mvsep.com/api/separation/cancel' \
  --form 'api_token="<您的 API 令牌>"' \
  --form 'hash="<任务 hash>"'
```

---

### 3. 用户登录
**POST** `https://mvsep.com/api/app/login`

**功能**: 验证用户身份并获取 API Token

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| email | 字符串 | ✅ | 用户邮箱 |
| password | 字符串 | ✅ | 用户密码 |

**Curl 示例**:
```bash
curl --location --request POST 'https://mvsep.com/api/app/login' \
  --form 'email="user@example.com"' \
  --form 'password="your_password"'
```

**响应**:
```json
{
  "success": true,
  "data": {
    "name": "用户名",
    "email": "user@example.com",
    "api_token": "xxxxx",          // ← 保存这个！后续API调用需要
    "premium_minutes": 100,         // 剩余积分
    "premium_enabled": 1,           // 是否允许使用积分
    "long_filenames_enabled": 0     // 文件名格式
  }
}
```

---

### 4. 用户注册
**POST** `https://mvsep.com/api/app/register`

**功能**: 注册新用户

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| name | 字符串 | ✅ | 用户名 |
| email | 字符串 | ✅ | 邮箱 |
| password | 字符串 | ✅ | 密码 |
| password_confirmation | 字符串 | ✅ | 确认密码 |

**Curl 示例**:
```bash
curl --location --request POST 'https://mvsep.com/api/app/register' \
  --form 'name="username"' \
  --form 'email="user@example.com"' \
  --form 'password="SecurePass123!"' \
  --form 'password_confirmation="SecurePass123!"'
```

---

### 5. 启用高级功能（使用积分）
**POST** `https://mvsep.com/api/app/enable_premium`

**功能**: 允许任务消耗积分以获得更快处理速度

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| api_token | 字符串 | ✅ | 用户 API 密钥 |

---

### 6. 禁用高级功能
**POST** `https://mvsep.com/api/app/disable_premium`

**功能**: 禁止任务消耗积分

**请求参数**: 同上

---

### 7. 启用长文件名
**POST** `https://mvsep.com/api/app/enable_long_filenames`

**功能**: 输出文件名包含更多信息（算法名、选项等）

**请求参数**: 同上

---

### 8. 禁用长文件名
**POST** `https://mvsep.com/api/app/disable_long_filenames`

**功能**: 使用简短文件名

**请求参数**: 同上

---

### 9. 质量检查 - 创建条目
**POST** `https://mvsep.com/api/quality_checker/add`

**功能**: 提交算法到质量检查排行榜

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| api_token | 字符串 | ✅ | API 密钥 |
| zipfile | 二进制 | ✅ | 待处理的 ZIP 文件 |
| algo_name | 字符串 | ✅ | 算法名称 |
| main_text | 字符串 | ✅ | 算法描述 |
| dataset_type | 字符串 | ❌ | 数据集类型 (0-12) |
| ensemble | 整数 | ❌ | 是否集成模型: 0=否, 1=是 |
| password | 字符串 | ✅ | 删除密码 |

---

### 10. 质量检查 - 删除条目
**POST** `https://mvsep.com/api/quality_checker/delete`

**功能**: 删除质量检查条目

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | 整数 | ✅ | 条目 ID |
| password | 字符串 | ✅ | 删除密码 |

---

## 📊 二、数据查询类 API（获取信息）

### 1. 获取分离结果
**GET** `https://mvsep.com/api/separation/get`

**功能**: 查询任务状态和获取下载链接

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| hash | 字符串 | ✅ | 任务哈希值 |
| mirror | 整数 | ❌ | 使用镜像下载: 0=否, 1=是（需 api_token + 1积分） |
| api_token | 字符串 | 条件 | mirror=1 时必填 |

**Curl 示例**:
```bash
curl --location --request GET 'https://mvsep.com/api/separation/get?hash=20230327071601-xxx.mp3'
```

**响应 - 状态值**:
- `not_found` - 任务无效
- `waiting` - 排队中
- `processing` - 处理中
- `done` - ✅ 完成（包含下载链接）
- `failed` - 处理失败
- `distributing` - 大文件分发中
- `merging` - 合并中

**完成时响应示例**:
```json
{
  "success": true,
  "status": "done",
  "data": {
    "algorithm": "BS Roformer",
    "algorithm_description": "...",
    "output_format": "wav",
    "input_file": { "link": "...", "size": 10240000 },
    "files": [
      { "name": "vocals.wav", "link": "...", "size": 5000000 },
      { "name": "instrumental.wav", "link": "...", "size": 5200000 }
    ],
    "date": "2023-03-27 07:20:00"
  }
}
```

**排队中响应示例**:
```json
{
  "success": true,
  "status": "waiting",
  "data": {
    "queue_count": 15,
    "current_order": 3,
    "message": "您在队列中排第 3 位"
  }
}
```

---

### 2. 获取远程任务结果
**GET** `https://mvsep.com/api/separation/get-remote`

**功能**: 查询远程提交的任务状态

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| hash | 字符串 | ✅ | 远程任务哈希值 |

**响应**: 结构同上，但 `done` 时返回新的 `hash` 用于获取实际结果

---

### 3. 获取分离类型（算法列表）
**GET** `https://mvsep.com/api/app/algorithms`

**功能**: 获取所有可用算法及参数定义（详见上方📌核心要点）

**重要**: 这是构建前端 UI 的基础 API，必须先调用此接口才能知道有哪些算法和参数

---

### 4. 获取用户信息
**GET** `https://mvsep.com/api/app/user`

**功能**: 获取当前用户信息和账户状态

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| api_token | 字符串 | ✅ | 用户 API 密钥 |

**响应**:
```json
{
  "success": true,
  "data": {
    "name": "用户名",
    "email": "user@example.com",
    "api_token": "xxxxx",
    "premium_minutes": 100,
    "premium_enabled": 1,
    "long_filenames_enabled": 0,
    "current_queue": null  // 当前正在处理的任务
  }
}
```

---

### 5. 获取分离历史
**GET** `https://mvsep.com/api/app/separation_history`

**功能**: 获取用户的任务历史记录

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| api_token | 字符串 | ✅ | 用户 API 密钥 |
| start | 整数 | ❌ | 起始偏移量（默认0，最新在前） |
| limit | 整数 | ❌ | 返回数量（默认10，最大20） |

---

### 6. 获取站点队列状态
**GET** `https://mvsep.com/api/app/queue`

**功能**: 查看当前服务器负载和排队情况

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| api_token | 字符串 | ❌ | 提供后可查看个人套餐队列信息 |

**响应**:
```json
{
  "queue": {
    "in_process": 25,
    "premium": 5,
    "registered": 150,
    "unregistered": 800
  },
  "plan": {
    "plan": "free",
    "queue": 800
  }
}
```

---

### 7. 获取新闻
**GET** `https://mvsep.com/api/app/news`

**功能**: 获取 MVSEP 最新动态

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| lang | 字符串 | ❌ | 语言代码: en, ru, zh 等 |
| start | 整数 | ❌ | 起始偏移量 |
| limit | 整数 | ❌ | 返回数量（默认10，最大20） |

---

### 8. 获取演示分离列表
**GET** `https://mvsep.com/api/app/demo`

**功能**: 获取官方提供的演示样本

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| start | 整数 | ❌ | 起始偏移量 |
| limit | 整数 | ❌ | 返回数量 |
| algorithm_id | 整数 | ❌ | 按算法ID过滤 |
| options[FIELD] | 混合 | ❌ | 按选项过滤（需先调用 algorithms API 获取字段名） |
| additional_options | 字符串 | ❌ | 原始选项JSON过滤（不推荐） |

**Curl 示例**:
```bash
# 获取所有演示
curl 'https://mvsep.com/api/app/demo?start=0&limit=10'

# 过滤特定算法的演示
curl 'https://mvsep.com/api/app/demo?algorithm_id=26&options[add_opt2]=7&start=0&limit=10'
```

---

### 9. 质量检查 - 获取队列
**GET** `https://mvsep.com/api/quality_checker/queue`

**功能**: 获取质量检查队列中的条目

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| start | 整数 | ❌ | 起始偏移量 |
| limit | 整数 | ❌ | 返回数量 |
| algorithm_id | 整数 | ❌ | 按算法ID过滤 |
| options[FIELD] | 混合 | ❌ | 按选项过滤 |

---

### 10. 质量检查 - 获取排行榜
**GET** `https://mvsep.com/api/quality_checker/leaderboard`

**功能**: 获取算法性能排行榜

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| dataset_type | 字符串 | ❌ | 数据集类型 (0-12)<br>0=Synth, 1=Multi, 2=Piano, 3=Lead/Back Vocals, 4=Guitar... |
| start | 整数 | ❌ | 起始偏移量 |
| limit | 整数 | ❌ | 返回数量 |
| algo_name_filter | 字符串 | ❌ | 按算法名搜索 |
| sort | 字符串 | ❌ | 排序字段（从响应 sortables 获取） |

---

### 11. 质量检查 - 获取条目详情
**GET** `https://mvsep.com/api/quality_checker/entry`

**功能**: 获取单个质量检查条目的详细信息

**请求参数**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | 整数 | ✅ | 条目 ID |

---

## 🔄 典型工作流程

### 流程1: 基础音频分离

```
1. [登录] POST /api/app/login → 获取 api_token
       ↓
2. [可选] GET /api/app/algorithms → 获取可用算法列表（构建UI）
       ↓
3. [创建任务] POST /api/separation/create → 上传音频 + 选择算法 → 获得 hash
       ↓
4. [轮询结果] GET /api/separation/get?hash=xxx → 检查 status
       ↓ (等待 status == "done")
5. [下载] 从响应 data.files[].link 下载分离后的音频文件
```

### 流程2: 动态构建算法选择器

```
1. GET /api/app/algorithms → 获取完整算法列表
       ↓
2. 解析响应，提取每个算法的：
   - render_id (sep_type)
   - name (显示名称)
   - algorithm_fields[] (参数定义)
     - name (参数键: add_opt1/2/3)
     - text (显示标签)
     - options (可选值映射)
     - default_key (默认值)
   - price_coefficient (积分成本)
   - orientation (权限要求)
       ↓
3. 动态渲染 UI 表单（下拉框等）
       ↓
4. 用户选择后，收集参数调用 POST /api/separation/create
```

### 流程3: 远程任务处理

```
适用于已存储在服务器的音频或URL引用的场景

1. POST /api/separation/create (特殊模式) → 获得远程 hash
       ↓
2. GET /api/separation/get-remote?hash=xxx → 轮询状态
       ↓ (status == "done" 时返回新 hash)
3. GET /api/separation/get?hash=<新hash> → 获取实际结果
```

---

## 📋 完整 API 端点清单

### 操作类（10个）
| # | 方法 | 端点 | 功能 | 认证 |
|---|------|------|------|------|
| 1 | POST | `/api/separation/create` | 创建分离任务 | ✅ Token |
| 2 | POST | `/api/separation/cancel` | 取消任务 | ✅ Token |
| 3 | POST | `/api/app/login` | 用户登录 | ❌ |
| 4 | POST | `/api/app/register` | 用户注册 | ❌ |
| 5 | POST | `/api/app/enable_premium` | 启用积分 | ✅ Token |
| 6 | POST | `/api/app/disable_premium` | 禁用积分 | ✅ Token |
| 7 | POST | `/api/app/enable_long_filenames` | 启用长文件名 | ✅ Token |
| 8 | POST | `/api/app/disable_long_filenames` | 禁用长文件名 | ✅ Token |
| 9 | POST | `/api/quality_checker/add` | 提交质量检查 | ✅ Token |
| 10 | POST | `/api/quality_checker/delete` | 删除条目 | ❌ Password |

### 数据查询类（11个）
| # | 方法 | 端点 | 功能 | 认证 |
|---|------|------|------|------|
| 1 | GET | `/api/separation/get` | 获取任务结果 | ❌ |
| 2 | GET | `/api/separation/get-remote` | 获取远程任务结果 | ❌ |
| 3 | GET | `/api/app/algorithms` | **获取算法列表** ⭐ | ❌ |
| 4 | GET | `/api/app/user` | 获取用户信息 | ✅ Token |
| 5 | GET | `/api/app/separation_history` | 获取历史记录 | ✅ Token |
| 6 | GET | `/api/app/queue` | 获取队列状态 | ❌ |
| 7 | GET | `/api/app/news` | 获取新闻 | ❌ |
| 8 | GET | `/api/app/demo` | 获取演示列表 | ❌ |
| 9 | GET | `/api/quality_checker/queue` | 获取QC队列 | ❌ |
| 10 | GET | `/api/quality_checker/leaderboard` | 获取排行榜 | ❌ |
| 11 | GET | `/api/quality_checker/entry` | 获取QC条目详情 | ❌ |

---

## ⚠️ 错误代码汇总

| HTTP Code | 含义 | 触发场景 |
|-----------|------|----------|
| **200** | 成功 | 请求正常完成 |
| **400** | 请求无效 | 参数错误/缺失、表单验证失败、凭据错误 |
| **401** | 未授权 | api_token 无效或未知 |

---

## 💡 最佳实践建议

1. **缓存算法列表**: `/api/app/algorithms` 结果应缓存（变更频率低），避免每次都请求
2. **轮询间隔**: 建议每 5-10 秒轮询一次任务状态，避免过于频繁
3. **错误重试**: 遇到 `waiting`/`processing` 状态时正常轮询，遇到 `failed` 时检查原因后决定是否重试
4. **积分管理**: 调用创建任务前先检查 `/api/app/user` 确认积分余额
5. **文件大小**: 注意免费用户 50MB 限制，付费用户 200MB 限制
6. **输出格式**: WAV 格式质量更高但文件更大，MP3 适合快速预览

---

*本文档基于 curl 直接抓取的原始 HTML 整理，确保信息准确无误*
