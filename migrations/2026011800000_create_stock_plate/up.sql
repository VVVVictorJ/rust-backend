create table stock_plate (
  id serial primary key,
  plate_code varchar(10) not null,  
  name varchar(255) not null,
  created_at timestamp not null default now(),
  updated_at timestamp not null default now()
)

create table stock_table(
  id serial primary key,
  stock_code varchar(10) not null,
  stock_name varchar(50) not null,
  created_at timestamp not null default now(),
  updated_at timestamp not null default now()
)

create table stock_plate_stock_table (
  plate_id int not null references stock_plate(id) on delete cascade,
  stock_table_id int not null references stock_table(id) on delete cascade,
  created_at timestamp not null default now(),
  updated_at timestamp not null default now()
)

comment on table stock_plate is '股票板块表';
comment on table stock_table is '股票表';
comment on table stock_plate_stock_table is '股票板块股票表';

comment on column stock_plate.id is '股票板块ID';
comment on column stock_plate.plate_code is '股票板块代码';
comment on column stock_plate.name is '股票板块名称';
comment on column stock_table.id is '股票表ID';
comment on column stock_table.stock_code is '股票代码';
comment on column stock_table.stock_name is '股票名称';
comment on column stock_plate_stock_table.plate_id is '股票板块ID';
comment on column stock_plate_stock_table.stock_table_id is '股票表ID';

-- 索引建议（按需启用）
create unique index if not exists idx_stock_plate_name on stock_plate(name);
create unique index if not exists idx_stock_plate_code on stock_plate(plate_code);
create unique index if not exists idx_stock_table_code on stock_table(stock_code);
create index if not exists idx_stock_table_name on stock_table(stock_name);
create index if not exists idx_stock_plate_stock_table_plate_id on stock_plate_stock_table(plate_id);
create index if not exists idx_stock_plate_stock_table_stock_table_id on stock_plate_stock_table(stock_table_id);
create unique index if not exists idx_stock_plate_stock_table_unique on stock_plate_stock_table(plate_id, stock_table_id);
