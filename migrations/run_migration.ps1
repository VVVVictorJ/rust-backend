# PostgreSQL 迁移执行脚本
# 使用方法: .\run_migration.ps1

$env:PGPASSWORD = "Stock2025@"

Write-Host "正在执行数据库迁移..." -ForegroundColor Green

# 执行迁移SQL
psql -h localhost -U stock_user -d stock_db -f "$PSScriptRoot\2025010200000_create_job_execution_history\up.sql"

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ 迁移成功完成!" -ForegroundColor Green
    
    Write-Host "`n验证表创建..." -ForegroundColor Yellow
    $query = "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'job_execution_history';"
    echo $query | psql -h localhost -U stock_user -d stock_db
} else {
    Write-Host "✗ 迁移失败，退出码: $LASTEXITCODE" -ForegroundColor Red
}

# 清除密码环境变量
Remove-Item Env:\PGPASSWORD

