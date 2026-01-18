## Docker 数据库表校验指南

### 1) 进入数据库容器

先查看容器名称（示例命令）：

```bash
docker ps
```

假设数据库容器名为 `postgres`，数据库名为 `stock`, 用户名为 `postgres`：

```bash
docker exec -it postgres psql -U postgres -d stock
```

> 如果你的容器名或数据库/用户不同，请替换为实际值。

### 2) 检查表是否存在

```sql
\dt
```

至少应看到：

```
stock_plate
stock_table
stock_plate_stock_table
```

### 3) 查询表结构

```sql
\d stock_plate
\d stock_table
\d stock_plate_stock_table
```

### 4) 验证索引

```sql
\d stock_plate
\d stock_table
\d stock_plate_stock_table
```

检查索引列表是否包含：

- `idx_stock_plate_name`
- `idx_stock_plate_code`
- `idx_stock_table_code`
- `idx_stock_table_name`
- `idx_stock_plate_stock_table_plate_id`
- `idx_stock_plate_stock_table_stock_table_id`
- `idx_stock_plate_stock_table_unique`

### 5) 简单数据验证（可选）

```sql
select count(*) from stock_plate;
select count(*) from stock_table;
select count(*) from stock_plate_stock_table;
```

### 6) 如果表不存在/索引缺失

在容器内确认迁移是否执行：

```sql
select * from __diesel_schema_migrations;
```

如需重新执行迁移，请在 `rust-backend/migrations` 下按照项目迁移流程操作。
