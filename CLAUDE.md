# CLAUDE.md

## Project Overview

**WorkItemFlow (Wif)** is a Windows desktop application for managing work items sourced from email, files, and manual entry. It provides AI-powered analysis and hybrid search (keyword FTS5 + semantic vector search via DuckDB).

The application is Chinese-localized (UI strings, error messages, and AI prompts use Chinese).

### Tech Stack

- **Backend**: Rust (workspace in `src-tauri/`)
- **Frontend**: React + TypeScript + Tailwind CSS v4 (Vite)
- **Desktop shell**: Tauri v2
- **Database**: SQLite (主数据库) + DuckDB (向量搜索)
- **Package manager**: npm (frontend), Cargo (backend)

### Architecture

```
Frontend (React + Tailwind CSS)
  ↕ Tauri IPC (invoke / events)
src-tauri/ (Rust workspace)
  ├── wif-app      ← Tauri 入口，命令注册
  └── crates/
      ├── wif-domain  ← 纯领域模型（实体、枚举、仓储 trait）
      ├── wif-data    ← SQLite 数据层（仓储实现、迁移）
      ├── wif-ai      ← LLM 客户端（OpenAI 兼容、提示词、出站策略）
      ├── wif-core    ← 编排层（DI、模块生命周期）
      ├── wif-docs    ← 文档处理（PDF/DOCX/XLSX/PPTX）
      ├── wif-mail    ← 邮件（IMAP/SMTP、OAuth2、IDLE）
      ├── wif-search  ← 混合搜索（FTS5 + DuckDB 向量 RRF 融合）
      └── wif-gis     ← GIS（地图、矢量/栅格、坐标变换）
```

### Module Mapping (C# → Rust)

| C# 模块 | Rust Crate | 说明 |
|---------|-----------|------|
| Wif.Domain | wif-domain | 纯领域模型，Ulid ID |
| Wif.Data | wif-data | SQLite + 手写幂等迁移 |
| Wif.AI | wif-ai | LLM 客户端 + 出站策略 |
| Wif.Core | wif-core | 编排 + DI 组合根 |
| Wif.Docs | wif-docs | 文档导入/导出 |
| Wif.Mail | wif-mail | IMAP/SMTP + OAuth2 |
| Wif.Search | wif-search | FTS5 + DuckDB 向量 RRF |
| Wif.Gis | wif-gis | 地图 + GDAL + 空间分析 |
| Wif.App | wif-app (src-tauri) + React frontend | UI 层 |

### Key Design Decisions (Preserved)

- **IDs**: All entities use `Ulid` (time-sortable, no DB sequence needed)
- **Database**: SQLite with hand-written idempotent migrations
- **Full-text search**: SQLite FTS5 virtual table with triggers
- **Semantic search**: DuckDB vector store, RRF fusion (k=60)
- **AI**: OpenAI-compatible LLM client with egress policy
- **Module system**: Trait-based module lifecycle management
- **Data directory**: `%LOCALAPPDATA%/WifData/`

### TBD Items

- [ ] SQLite ORM: rusqlite vs sea-orm vs sqlx
- [ ] GIS 前端库: MapLibre GL JS / Leaflet / deck.gl
- [ ] 文档处理库（Rust 生态有限，可能需要 WASM 或 sidecar）
- [ ] 邮件库: imap crate + lettre (SMTP)
- [ ] GDAL: gdal crate 成熟度评估

### Build & Run

```bash
# Install dependencies
npm install

# Development (starts both Vite dev server and Tauri)
npm run tauri dev

# Build for production
npm run tauri build

# Rust only (from src-tauri/)
cd src-tauri && cargo build

# Frontend only
npm run dev

# Type check
npm run build
```

### Design Documents

All preserved design docs are in `docs/`:
- `docs/gis-ui-interface-map.md` — GIS UI/命令映射
- `docs/superpowers/platform-capability-matrix.md` — 平台能力矩阵
- `docs/superpowers/plans/` — 实现计划
- `docs/superpowers/specs/` — 设计规格

## gstack

Use the `/browse` skill from gstack for all web browsing. Never use `mcp__claude-in-chrome__*` tools.

### Available skills

- `/office-hours` - Office hours
- `/plan-ceo-review` - CEO review planning
- `/plan-eng-review` - Engineering review planning
- `/plan-design-review` - Design review planning
- `/design-consultation` - Design consultation
- `/design-shotgun` - Design shotgun
- `/design-html` - Design HTML
- `/review` - Code review
- `/ship` - Ship code
- `/land-and-deploy` - Land and deploy
- `/canary` - Canary deployment
- `/benchmark` - Benchmarking
- `/browse` - Web browsing
- `/connect-chrome` - Connect to Chrome
- `/qa` - QA testing
- `/qa-only` - QA only
- `/design-review` - Design review
- `/setup-browser-cookies` - Setup browser cookies
- `/setup-deploy` - Setup deploy
- `/retro` - Retrospective
- `/investigate` - Investigate issues
- `/document-release` - Document release
- `/codex` - Codex
- `/cso` - CSO
- `/autoplan` - Auto planning
- `/careful` - Careful mode
- `/freeze` - Freeze
- `/guard` - Guard
- `/unfreeze` - Unfreeze
- `/gstack-upgrade` - Upgrade gstack
- `/learn` - Learn

## Skill routing

When the user's request matches an available skill, ALWAYS invoke it using the Skill
tool as your FIRST action. Do NOT answer directly, do NOT use other tools first.
The skill has specialized workflows that produce better results than ad-hoc answers.

Key routing rules:
- Product ideas, "is this worth building", brainstorming → invoke office-hours
- Bugs, errors, "why is this broken", 500 errors → invoke investigate
- Ship, deploy, push, create PR → invoke ship
- QA, test the site, find bugs → invoke qa
- Code review, check my diff → invoke review
- Update docs after shipping → invoke document-release
- Weekly retro → invoke retro
- Design system, brand → invoke design-consultation
- Visual audit, design polish → invoke design-review
- Architecture review → invoke plan-eng-review
- Save progress, checkpoint, resume → invoke checkpoint
- Code quality, health check → invoke health
