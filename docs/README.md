# 文档导航

`docs/` 是公开的项目设计与协作资料入口。文档应以仓库源码、配置和 CI 为依据，不能要求贡献者访问团队内部文档才能理解当前边界。

## 当前文档

- [Roadmap](./roadmap/README.md)：当前范围、已知缺口和后续优先级。
- [Design](./design/README.md)：当前架构、插件模型和前端设计契约。
- [Rules](./rules/README.md)：贡献、验证和文档维护规则。
- [English README](./README-en.md)：英文用户与开发者入口。

## 历史与临时资料

- [Archive](./archive/README.md)：已经被当前实现或新文档取代的资料摘要，不是当前规范。
- `tmp/`：具有时效性的计划、交接或验证记录。临时文档必须遵守 YAML front matter 和 CI 校验规则，过期后应删除或归档。

## 阅读规则

1. 先看本页和 [Design](./design/README.md) 了解代码边界。
2. 需要提交修改时阅读 [Rules](./rules/README.md)。
3. 当前文档与源码冲突时，以源码和可复现的 CI 行为为准，并在同一个变更中修正文档。
4. 不在稳定文档中记录一次性排查过程、未确认的未来设计或完整的生成式 API 快照。
