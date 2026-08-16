create table export_button_config (
  id serial primary key,
  page_key varchar(50) not null,
  name varchar(50) not null,
  plate_codes jsonb not null default '[]',
  sort_order int not null default 0,
  created_at timestamp not null default now(),
  updated_at timestamp not null default now()
)

comment on table export_button_config is '导出按钮配置表';
comment on column export_button_config.id is '配置ID';
comment on column export_button_config.page_key is '所属页面标识，如 trade-date-query / track-data';
comment on column export_button_config.name is '按钮显示名称';
comment on column export_button_config.plate_codes is '绑定的板块代码数组';
comment on column export_button_config.sort_order is '排序';
